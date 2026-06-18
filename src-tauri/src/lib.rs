// AI-generated (Claude)
mod types;
mod ssrf_git;

use tauri_plugin_store::StoreExt;
use types::Logbook;

#[tauri::command]
async fn startup_logbook(app: tauri::AppHandle) -> Result<Logbook, String> {
    use tauri::Manager;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let saved_path = store
        .get("logbookPath")
        .and_then(|v| v.as_str().map(str::to_owned))
        .and_then(|s| {
            let p = std::path::PathBuf::from(&s);
            if p.is_dir() { Some(p) } else { None }
        });

    let root = match saved_path {
        Some(p) => p,
        None => {
            let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            let default = data_dir.join("logbook");
            std::fs::create_dir_all(&default).map_err(|e| e.to_string())?;
            store.set("logbookPath", serde_json::json!(default.to_string_lossy().as_ref()));
            store.save().map_err(|e| e.to_string())?;
            default
        }
    };

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
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("logbookPath", serde_json::json!(root));
    store.save().map_err(|e| e.to_string())?;
    let path = std::path::PathBuf::from(root);
    tauri::async_runtime::spawn_blocking(move || crate::ssrf_git::parse_logbook(&path))
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            startup_logbook,
            open_logbook,
            new_logbook,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
