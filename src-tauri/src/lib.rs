// AI-generated (Claude)
#[cfg(desktop)]
mod menu;
mod cloud;
mod dc;
mod ssrf_git;
mod state;
mod types;

use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tauri_plugin_store::StoreExt;
use state::LogbookState;
use types::{OpenResult, RecentEntry};

fn validate_logbook_path(path: &std::path::Path) -> Result<(), String> {
    use std::path::Component;
    if !path.is_absolute() {
        return Err("logbook path must be absolute".to_string());
    }
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err("logbook path must not contain '..'".to_string());
    }
    Ok(())
}

fn path_basename(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_owned()
}

pub(crate) fn update_recents(
    store: &tauri_plugin_store::Store<impl tauri::Runtime>,
    entry: RecentEntry,
) -> Result<Vec<RecentEntry>, String> {
    let mut recents: Vec<RecentEntry> = store
        .get("recents")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    recents.retain(|e| match (e, &entry) {
        (RecentEntry::Local { path: p1 }, RecentEntry::Local { path: p2 }) => p1 != p2,
        (
            RecentEntry::Cloud { email: e1, url: u1 },
            RecentEntry::Cloud { email: e2, url: u2 },
        ) => e1 != e2 || u1 != u2,
        _ => true,
    });
    recents.insert(0, entry);

    store.set("recents", serde_json::to_value(&recents).map_err(|e| e.to_string())?);
    store.save().map_err(|e| e.to_string())?;
    Ok(recents)
}

fn install_logbook(
    app: &tauri::AppHandle,
    logbook_state: &tauri::State<'_, Mutex<Option<LogbookState>>>,
    root: std::path::PathBuf,
    parsed: types::ParsedLogbook,
) -> Result<types::Logbook, String> {
    // Refuse to swap logbooks out from under a dive-computer download that's
    // still buffered awaiting review/commit — commit_dc_download snapshots
    // the logbook root at download-start time, so switching here would write
    // the buffered dives to one logbook while the fingerprint update lands
    // in whichever logbook ends up open.
    {
        use tauri::Manager;
        let pending = app.state::<dc::commands::PendingDownloadState>();
        if pending.lock().map_err(|e| e.to_string())?.is_some() {
            return Err("A dive computer download is waiting for review — save or discard it before switching logbooks.".to_string());
        }
    }

    let state = LogbookState {
        root,
        dives: parsed.dives,
        trips: parsed.trips,
        sites: parsed.sites,
        settings: parsed.settings,
    };
    let logbook = state.to_logbook();
    *logbook_state.lock().map_err(|e| e.to_string())? = Some(state);
    Ok(logbook)
}

#[tauri::command]
async fn startup_logbook(
    app: tauri::AppHandle,
    logbook_state: tauri::State<'_, Mutex<Option<LogbookState>>>,
) -> Result<OpenResult, String> {
    use tauri::Manager;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    let saved_str = store
        .get("logbookPath")
        .and_then(|v| v.as_str().map(str::to_owned));

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let default = data_dir.join("logbook");

    let (root, created_new, is_cloud) = tauri::async_runtime::spawn_blocking(
        move || -> Result<(std::path::PathBuf, bool, bool), String> {
            if let Some(s) = saved_str {
                let p = std::path::PathBuf::from(&s);
                if p.is_dir() {
                    let is_cloud = p.join(".git").is_dir();
                    return Ok((p, false, is_cloud));
                }
            }
            std::fs::create_dir_all(&default).map_err(|e| e.to_string())?;
            Ok((default, true, false))
        },
    )
    .await
    .map_err(|e| e.to_string())??;

    if created_new {
        store.set("logbookPath", serde_json::json!(root.to_string_lossy().as_ref()));
        store.save().map_err(|e| e.to_string())?;
    }

    let (entry, display_name) = if is_cloud {
        let email = store
            .get("cloudEmail")
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        let url = cloud::CLOUD_BASE.to_owned();
        let name = cloud::cloud_display_name(&email, &url);
        (RecentEntry::Cloud { email, url }, name)
    } else {
        let path = root.to_string_lossy().to_string();
        let name = path_basename(&root);
        (RecentEntry::Local { path }, name)
    };

    let recents = update_recents(&store, entry)?;

    let root_clone = root.clone();
    let parsed = tauri::async_runtime::spawn_blocking(move || crate::ssrf_git::parse_logbook(&root_clone))
        .await
        .map_err(|e| e.to_string())??;

    let warnings = parsed.warnings.clone();
    let logbook = install_logbook(&app, &logbook_state, root, parsed)?;

    #[cfg(desktop)]
    menu::rebuild(&app, &recents).map_err(|e| e.to_string())?;

    Ok(OpenResult { logbook, display_name, recents, warnings })
}

#[tauri::command]
async fn open_logbook(
    app: tauri::AppHandle,
    root: String,
    logbook_state: tauri::State<'_, Mutex<Option<LogbookState>>>,
) -> Result<OpenResult, String> {
    let path = std::path::PathBuf::from(&root);
    validate_logbook_path(&path)?;

    // Parse first — nothing is persisted until we know the logbook is readable.
    let path_clone = path.clone();
    let parsed = tauri::async_runtime::spawn_blocking(move || crate::ssrf_git::parse_logbook(&path_clone))
        .await
        .map_err(|e| e.to_string())??;

    let warnings = parsed.warnings.clone();
    let logbook = install_logbook(&app, &logbook_state, path.clone(), parsed)?;

    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let display_name = path_basename(&path);
    store.set("logbookPath", serde_json::json!(root));
    store.save().map_err(|e| e.to_string())?;
    let recents = update_recents(&store, RecentEntry::Local { path: root })?;

    #[cfg(desktop)]
    menu::rebuild(&app, &recents).map_err(|e| e.to_string())?;

    Ok(OpenResult { logbook, display_name, recents, warnings })
}

#[tauri::command]
async fn new_logbook(
    app: tauri::AppHandle,
    root: String,
    logbook_state: tauri::State<'_, Mutex<Option<LogbookState>>>,
) -> Result<OpenResult, String> {
    let path = std::path::PathBuf::from(&root);
    validate_logbook_path(&path)?;

    // Create dir and parse first — nothing is persisted until we know the path is good.
    let path_clone = path.clone();
    let parsed = tauri::async_runtime::spawn_blocking(move || {
        std::fs::create_dir_all(&path_clone).map_err(|e| e.to_string())?;
        crate::ssrf_git::parse_logbook(&path_clone)
    })
    .await
    .map_err(|e| e.to_string())??;

    let warnings = parsed.warnings.clone();
    let logbook = install_logbook(&app, &logbook_state, path.clone(), parsed)?;

    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let display_name = path_basename(&path);
    store.set("logbookPath", serde_json::json!(root));
    store.save().map_err(|e| e.to_string())?;
    let recents = update_recents(&store, RecentEntry::Local { path: root })?;

    #[cfg(desktop)]
    menu::rebuild(&app, &recents).map_err(|e| e.to_string())?;

    Ok(OpenResult { logbook, display_name, recents, warnings })
}

#[tauri::command]
async fn get_dive(
    number: i32,
    logbook_state: tauri::State<'_, Mutex<Option<LogbookState>>>,
) -> Result<types::Dive, String> {
    let guard = logbook_state.lock().map_err(|e| e.to_string())?;
    let state = guard.as_ref().ok_or_else(|| "no logbook open".to_string())?;
    state.dives
        .iter()
        .find(|d| d.number == number)
        .cloned()
        .ok_or_else(|| format!("dive {} not found", number))
}

#[tauri::command]
async fn get_recents(app: tauri::AppHandle) -> Result<Vec<RecentEntry>, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    Ok(store
        .get("recents")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default())
}

fn default_log_level() -> log::LevelFilter {
    if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Info }
}

#[tauri::command]
fn get_log_level() -> String {
    log::max_level().to_string()
}

#[tauri::command]
fn set_log_level(app: tauri::AppHandle, level: String) -> Result<(), String> {
    let parsed: log::LevelFilter = level
        .parse()
        .map_err(|_| format!("invalid log level: {level}"))?;
    log::set_max_level(parsed);
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("logging", serde_json::json!({ "level": level.to_lowercase() }));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .manage(Mutex::new(None::<LogbookState>))
        .manage(Arc::new(AtomicBool::new(false)))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let level = app
                .store("settings.json")
                .ok()
                .and_then(|s| s.get("logging"))
                .and_then(|v| v.get("level").and_then(|l| l.as_str()).map(str::to_owned))
                .and_then(|s| s.parse::<log::LevelFilter>().ok())
                .unwrap_or_else(default_log_level);
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(level)
                    .build(),
            )?;
            #[cfg(desktop)]
            {
                let m = menu::build(app.handle(), &[])?;
                app.set_menu(m)?;
            }
            Ok(())
        });

    let builder = builder.manage(Mutex::new(None::<dc::commands::PendingDownload>));

    #[cfg(target_os = "android")]
    let builder = builder.plugin(dc_ble::init());

    #[cfg(desktop)]
    let builder = builder.on_menu_event(menu::handle_event);

    builder
        .invoke_handler(tauri::generate_handler![
            startup_logbook,
            open_logbook,
            new_logbook,
            get_dive,
            get_recents,
            get_log_level,
            set_log_level,
            cloud::get_cloud_credentials,
            cloud::open_cloud_logbook,
            cloud::open_recent_cloud_logbook,
            cloud::sync_cloud_logbook,
            dc::commands::list_dc_vendors,
            dc::commands::list_dc_models,
            dc::commands::list_known_devices,
            dc::commands::list_serial_ports,
            dc::commands::scan_ble_devices,
            #[cfg(target_os = "android")]
            dc::commands::open_app_settings,
            dc::commands::start_dc_download,
            dc::commands::commit_dc_download,
            dc::commands::discard_dc_download,
            dc::commands::cancel_dc_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_log_level_matches_build_profile() {
        let expected = if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        };
        assert_eq!(default_log_level(), expected);
    }
}
