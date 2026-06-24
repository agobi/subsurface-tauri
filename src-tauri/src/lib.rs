// AI-generated (Claude)
#[cfg(desktop)]
mod menu;
mod cloud;
mod ssrf_git;
mod types;

use std::sync::Mutex;
use tauri_plugin_store::StoreExt;
use types::{Dive, OpenResult, RecentEntry};

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

#[tauri::command]
async fn startup_logbook(
    app: tauri::AppHandle,
    dives_state: tauri::State<'_, Mutex<Vec<Dive>>>,
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

    let (full_dives, logbook) = parsed.into_summary();
    *dives_state.lock().map_err(|e| e.to_string())? = full_dives;

    #[cfg(desktop)]
    menu::rebuild(&app, &recents).map_err(|e| e.to_string())?;

    Ok(OpenResult { logbook, display_name, recents })
}

#[tauri::command]
async fn open_logbook(
    app: tauri::AppHandle,
    root: String,
    dives_state: tauri::State<'_, Mutex<Vec<Dive>>>,
) -> Result<OpenResult, String> {
    let path = std::path::PathBuf::from(&root);
    validate_logbook_path(&path)?;

    // Parse first — nothing is persisted until we know the logbook is readable.
    let path_clone = path.clone();
    let parsed = tauri::async_runtime::spawn_blocking(move || crate::ssrf_git::parse_logbook(&path_clone))
        .await
        .map_err(|e| e.to_string())??;

    let (full_dives, logbook) = parsed.into_summary();
    *dives_state.lock().map_err(|e| e.to_string())? = full_dives;

    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let display_name = path_basename(&path);
    store.set("logbookPath", serde_json::json!(root));
    store.save().map_err(|e| e.to_string())?;
    let recents = update_recents(&store, RecentEntry::Local { path: root })?;

    #[cfg(desktop)]
    menu::rebuild(&app, &recents).map_err(|e| e.to_string())?;

    Ok(OpenResult { logbook, display_name, recents })
}

#[tauri::command]
async fn new_logbook(
    app: tauri::AppHandle,
    root: String,
    dives_state: tauri::State<'_, Mutex<Vec<Dive>>>,
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

    let (full_dives, logbook) = parsed.into_summary();
    *dives_state.lock().map_err(|e| e.to_string())? = full_dives;

    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let display_name = path_basename(&path);
    store.set("logbookPath", serde_json::json!(root));
    store.save().map_err(|e| e.to_string())?;
    let recents = update_recents(&store, RecentEntry::Local { path: root })?;

    #[cfg(desktop)]
    menu::rebuild(&app, &recents).map_err(|e| e.to_string())?;

    Ok(OpenResult { logbook, display_name, recents })
}

#[tauri::command]
async fn get_dive(
    number: i32,
    dives_state: tauri::State<'_, Mutex<Vec<Dive>>>,
) -> Result<Dive, String> {
    let guard = dives_state.lock().map_err(|e| e.to_string())?;
    guard
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .manage(Mutex::new(Vec::<Dive>::new()))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            #[cfg(desktop)]
            {
                let m = menu::build(app.handle(), &[])?;
                app.set_menu(m)?;
            }
            Ok(())
        });

    #[cfg(desktop)]
    let builder = builder.on_menu_event(menu::handle_event);

    builder
        .invoke_handler(tauri::generate_handler![
            startup_logbook,
            open_logbook,
            new_logbook,
            get_dive,
            get_recents,
            cloud::get_cloud_credentials,
            cloud::open_cloud_logbook,
            cloud::open_recent_cloud_logbook,
            cloud::sync_cloud_logbook,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
