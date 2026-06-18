// AI-generated (Claude)
mod menu;
mod ssrf_git;
mod types;

use tauri_plugin_store::StoreExt;
use types::Logbook;

#[tauri::command]
async fn startup_logbook(app: tauri::AppHandle) -> Result<Logbook, String> {
    use tauri::Manager;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    // Retrieve saved path string without touching the filesystem yet.
    let saved_str = store
        .get("logbookPath")
        .and_then(|v| v.as_str().map(str::to_owned));

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let default = data_dir.join("logbook");

    // Resolve the logbook root in a blocking thread: is_dir() and create_dir_all()
    // must not run on the async executor.
    let (root, created_new) = tauri::async_runtime::spawn_blocking(
        move || -> Result<(std::path::PathBuf, bool), String> {
            if let Some(s) = saved_str {
                let p = std::path::PathBuf::from(&s);
                if p.is_dir() {
                    return Ok((p, false));
                }
            }
            std::fs::create_dir_all(&default).map_err(|e| e.to_string())?;
            Ok((default, true))
        },
    )
    .await
    .map_err(|e| e.to_string())??;

    // Persist the new default path; store uses Arc/Mutex so it is safe on the
    // async thread.
    if created_new {
        store.set("logbookPath", serde_json::json!(root.to_string_lossy().as_ref()));
        store.save().map_err(|e| e.to_string())?;
    }

    tauri::async_runtime::spawn_blocking(move || crate::ssrf_git::parse_logbook(&root))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn open_logbook(app: tauri::AppHandle, root: String) -> Result<Logbook, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("logbookPath", serde_json::json!(root));
    store.save().map_err(|e| e.to_string())?;
    let path = std::path::PathBuf::from(root);
    tauri::async_runtime::spawn_blocking(move || crate::ssrf_git::parse_logbook(&path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn new_logbook(app: tauri::AppHandle, root: String) -> Result<Logbook, String> {
    // Store update is non-blocking (Arc/Mutex internals); do it on the async thread.
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("logbookPath", serde_json::json!(root));
    store.save().map_err(|e| e.to_string())?;
    // create_dir_all is blocking; merge it into the same spawn_blocking as parse_logbook.
    let path = std::path::PathBuf::from(root);
    tauri::async_runtime::spawn_blocking(move || {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        crate::ssrf_git::parse_logbook(&path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            let m = menu::build(app.handle())?;
            app.set_menu(m)?;
            Ok(())
        })
        .on_menu_event(menu::handle_event)
        .invoke_handler(tauri::generate_handler![
            startup_logbook,
            open_logbook,
            new_logbook,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
