// AI-generated (Claude)
use keyring::Entry;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use crate::types::Logbook;

const KEYRING_SERVICE: &str = "subsurface-tauri";

fn cloud_url(email: &str) -> String {
    format!(
        "https://cloud.subsurface-divelog.org/git/{email}[{email}]",
        email = email
    )
}

fn map_error_message(msg: &str) -> String {
    if msg.contains("401") || msg.contains("authentication") || msg.contains("credential") {
        "Authentication failed. Check your email and password.".to_string()
    } else if msg.contains("resolve host") || msg.contains("network") || msg.contains("timed out") {
        "Could not reach Subsurface Cloud. Check your connection.".to_string()
    } else {
        msg.to_string()
    }
}

fn map_git_error(e: git2::Error) -> String {
    map_error_message(e.message())
}

fn make_fetch_opts<'a>(email: &'a str, password: &'a str) -> git2::FetchOptions<'a> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |_, _, _| git2::Cred::userpass_plaintext(email, password));
    let mut opts = git2::FetchOptions::new();
    opts.remote_callbacks(callbacks);
    opts
}

fn clone_or_fetch(
    url: &str,
    cache_dir: &std::path::Path,
    email: &str,
    password: &str,
) -> Result<(), String> {
    if cache_dir.is_dir() {
        let repo = git2::Repository::open(cache_dir).map_err(|e| e.to_string())?;
        let mut remote = repo.find_remote("origin").map_err(|e| e.to_string())?;
        let mut opts = make_fetch_opts(email, password);
        remote
            .fetch(&[] as &[&str], Some(&mut opts), None)
            .map_err(map_git_error)?;
        let remote_head = repo
            .find_reference("refs/remotes/origin/HEAD")
            .or_else(|_| repo.find_reference("refs/remotes/origin/master"))
            .map_err(|_| "Cannot determine remote branch after fetch".to_string())?;
        let resolved = remote_head.resolve().map_err(|e| e.to_string())?;
        let oid = resolved
            .target()
            .ok_or_else(|| "Unborn remote HEAD".to_string())?;
        let obj = repo.find_object(oid, None).map_err(|e| e.to_string())?;
        repo.reset(&obj, git2::ResetType::Hard, None)
            .map_err(|e| e.to_string())?;
    } else {
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(make_fetch_opts(email, password));
        builder.clone(url, cache_dir).map_err(map_git_error)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_cloud_credentials(
    app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    Ok(store
        .get("cloudEmail")
        .and_then(|v| v.as_str().map(str::to_owned)))
}

#[tauri::command]
pub async fn open_cloud_logbook(
    app: tauri::AppHandle,
    email: String,
    password: String,
) -> Result<Logbook, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let cache_dir = data_dir.join("cloud").join(&email);
    let url = cloud_url(&email);

    // Clone credentials so we can persist them after a successful clone/fetch
    let email_for_creds = email.clone();
    let password_for_creds = password.clone();
    let cache_dir_for_parse = cache_dir.clone();

    // Step 1: Attempt clone/fetch first — credentials are NOT saved until this succeeds
    tauri::async_runtime::spawn_blocking(move || {
        clone_or_fetch(&url, &cache_dir, &email, &password)
    })
    .await
    .map_err(|e| e.to_string())??;

    // Step 2+3: Save credentials and parse in one blocking call to avoid stalling the async runtime
    let app_clone = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Save email to settings.json (fast in-memory + file flush)
        let store = app_clone.store("settings.json").map_err(|e| e.to_string())?;
        store.set("cloudEmail", serde_json::json!(email_for_creds));
        store.save().map_err(|e| e.to_string())?;
        // Save password to OS keychain (blocks on IPC — must be in spawn_blocking)
        let entry = Entry::new(KEYRING_SERVICE, &email_for_creds).map_err(|e| e.to_string())?;
        entry.set_password(&password_for_creds).map_err(|e| e.to_string())?;
        // Parse logbook
        crate::ssrf_git::parse_logbook(&cache_dir_for_parse)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn sync_cloud_logbook(app: tauri::AppHandle) -> Result<Logbook, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let email = store
        .get("cloudEmail")
        .and_then(|v| v.as_str().map(str::to_owned))
        .ok_or_else(|| "No cloud logbook configured.".to_string())?;

    let entry = Entry::new(KEYRING_SERVICE, &email).map_err(|e| e.to_string())?;
    let password = entry.get_password().map_err(|_| {
        "No saved credentials found. Please open the cloud logbook again.".to_string()
    })?;

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let cache_dir = data_dir.join("cloud").join(&email);
    let url = cloud_url(&email);

    tauri::async_runtime::spawn_blocking(move || {
        clone_or_fetch(&url, &cache_dir, &email, &password)?;
        crate::ssrf_git::parse_logbook(&cache_dir)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_url_format() {
        assert_eq!(
            cloud_url("user@example.com"),
            "https://cloud.subsurface-divelog.org/git/user@example.com[user@example.com]"
        );
    }

    #[test]
    fn map_error_message_auth_401() {
        assert_eq!(
            map_error_message("401 authentication required"),
            "Authentication failed. Check your email and password."
        );
    }

    #[test]
    fn map_error_message_auth_credential() {
        assert_eq!(
            map_error_message("invalid credential"),
            "Authentication failed. Check your email and password."
        );
    }

    #[test]
    fn map_error_message_network() {
        assert_eq!(
            map_error_message("failed to resolve host 'cloud.subsurface-divelog.org'"),
            "Could not reach Subsurface Cloud. Check your connection."
        );
    }

    #[test]
    fn map_error_message_other() {
        assert_eq!(
            map_error_message("repository not found"),
            "repository not found"
        );
    }
}
