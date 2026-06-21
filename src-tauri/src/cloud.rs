// AI-generated (Claude)
use keyring::Entry;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use crate::types::{Logbook, OpenResult, RecentEntry};

pub(crate) const CLOUD_BASE: &str = "https://ssrf-cloud-eu.subsurface-divelog.org";

// Default to the EU server, matching Qt's default (cloud_base_url in pref.cpp).
const KEYRING_SERVICE: &str = "subsurface-tauri";

fn cloud_remote_url(email: &str) -> String {
    format!("{CLOUD_BASE}/git/{email}")
}

// The Subsurface cloud server uses the email address as the git branch name.
fn cloud_branch(email: &str) -> &str {
    email
}

/// Calls the Subsurface cloud /storage REST endpoint to validate credentials.
/// This also initializes the git repo on the server for new accounts.
/// Returns Ok(()) for [OK] or [VERIFIED], Err with a human-readable message otherwise.
async fn rest_authenticate(email: &str, password: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let body = format!("{email} {password}");
    let resp = client
        .post(format!("{CLOUD_BASE}/storage"))
        .header("Content-Type", "text/plain")
        .header("Accept", "text/xml, text/plain")
        .body(body)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                "Could not reach Subsurface Cloud. Check your connection.".to_string()
            } else {
                e.to_string()
            }
        })?;
    let text = resp.text().await.map_err(|e| e.to_string())?;
    match text.trim() {
        "[OK]" | "[VERIFIED]" => Ok(()),
        "[VERIFY]" => Err("Email not yet verified. Check your inbox for a PIN.".to_string()),
        "Invalid PIN" => Err("Invalid PIN.".to_string()),
        other => Err(format!("Cloud authentication failed: {other}")),
    }
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
    log::warn!("git2 error (class={:?}): {}", e.class(), e.message());
    map_error_message(e.message())
}

fn make_fetch_opts<'a>(email: &'a str, password: &'a str) -> git2::FetchOptions<'a> {
    let mut callbacks = git2::RemoteCallbacks::new();
    // Only provide credentials once — replaying the same credentials triggers libgit2's
    // "too many redirects or authentication replays" error (mirrors Qt's exceeded_auth_attempts).
    let mut attempt = 0u32;
    callbacks.credentials(move |_, _, _| {
        attempt += 1;
        if attempt > 1 {
            return Err(git2::Error::from_str("credentials rejected by server"));
        }
        git2::Cred::userpass_plaintext(email, password)
    });
    let mut opts = git2::FetchOptions::new();
    opts.remote_callbacks(callbacks);
    opts
}

fn clone_or_fetch(
    url: &str,
    branch: &str,
    cache_dir: &std::path::Path,
    email: &str,
    password: &str,
) -> Result<(), String> {
    if cache_dir.is_dir() {
        let repo = git2::Repository::open(cache_dir).map_err(|e| e.to_string())?;
        // Always sync the remote URL — the cache may have been written by an older version
        // of this code that stored a different URL format (e.g. with [branch] brackets).
        repo.remote_set_url("origin", url).map_err(|e| e.to_string())?;
        let mut remote = repo.find_remote("origin").map_err(|e| e.to_string())?;
        let mut opts = make_fetch_opts(email, password);
        // Fetch using the configured refspec (maps refs/heads/* → refs/remotes/origin/*).
        remote
            .fetch(&[] as &[&str], Some(&mut opts), None)
            .map_err(map_git_error)?;
        let refname = format!("refs/remotes/origin/{branch}");
        let remote_ref = repo
            .find_reference(&refname)
            .map_err(|_| format!("Remote branch '{branch}' not found after fetch — is the account empty?"))?;
        let resolved = remote_ref.resolve().map_err(|e| e.to_string())?;
        let oid = resolved
            .target()
            .ok_or_else(|| "Unborn remote branch".to_string())?;
        let obj = repo.find_object(oid, None).map_err(|e| e.to_string())?;
        repo.reset(&obj, git2::ResetType::Hard, None)
            .map_err(|e| e.to_string())?;
    } else {
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(make_fetch_opts(email, password));
        builder.branch(branch);
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
) -> Result<OpenResult, String> {
    // Qt lowercases the email before all cloud operations (preferences_cloud.cpp::syncSettings).
    let email = email.to_lowercase();

    // Step 1: Validate credentials via REST (also initialises git repo for new accounts).
    rest_authenticate(&email, &password).await?;

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let cache_dir = data_dir.join("cloud").join(&email);
    let url = cloud_remote_url(&email);
    let branch = cloud_branch(&email).to_owned();

    let display_name = email.clone();
    let email_for_creds = email.clone();
    let password_for_creds = password.clone();
    let cache_dir_for_parse = cache_dir.clone();

    // Step 2: Attempt clone/fetch — credentials are NOT saved until this succeeds
    tauri::async_runtime::spawn_blocking(move || {
        clone_or_fetch(&url, &branch, &cache_dir, &email, &password)
    })
    .await
    .map_err(|e| e.to_string())??;

    // Step 3: Save credentials, persist logbookPath, update recents, and parse
    let app_clone = app.clone();
    let cloud_url = CLOUD_BASE.to_owned();
    let (logbook, recents) = tauri::async_runtime::spawn_blocking(move || -> Result<(Logbook, Vec<RecentEntry>), String> {
        let store = app_clone.store("settings.json").map_err(|e| e.to_string())?;
        store.set("cloudEmail", serde_json::json!(email_for_creds));
        store.set("logbookPath", serde_json::json!(cache_dir_for_parse.to_string_lossy().as_ref()));
        store.save().map_err(|e| e.to_string())?;
        let entry = Entry::new(KEYRING_SERVICE, &email_for_creds).map_err(|e| e.to_string())?;
        entry.set_password(&password_for_creds).map_err(|e| e.to_string())?;
        let recents = crate::update_recents(
            &store,
            RecentEntry::Cloud { email: email_for_creds.clone(), url: cloud_url },
        )?;
        let logbook = crate::ssrf_git::parse_logbook(&cache_dir_for_parse)?;
        Ok((logbook, recents))
    })
    .await
    .map_err(|e| e.to_string())??;

    #[cfg(desktop)]
    crate::menu::rebuild(&app, &recents).map_err(|e| e.to_string())?;

    Ok(OpenResult { logbook, display_name, recents })
}

#[tauri::command]
pub async fn sync_cloud_logbook(app: tauri::AppHandle) -> Result<OpenResult, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let email = store
        .get("cloudEmail")
        .and_then(|v| v.as_str().map(str::to_owned))
        .ok_or_else(|| "No cloud logbook configured.".to_string())?;

    // Read current recents without modifying them — sync doesn't change the entry.
    let recents: Vec<RecentEntry> = store
        .get("recents")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let entry = Entry::new(KEYRING_SERVICE, &email).map_err(|e| e.to_string())?;
    let password = entry.get_password().map_err(|_| {
        "No saved credentials found. Please open the cloud logbook again.".to_string()
    })?;

    let display_name = email.clone();
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let cache_dir = data_dir.join("cloud").join(&email);
    let url = cloud_remote_url(&email);
    let branch = cloud_branch(&email).to_owned();

    let logbook = tauri::async_runtime::spawn_blocking(move || {
        clone_or_fetch(&url, &branch, &cache_dir, &email, &password)?;
        crate::ssrf_git::parse_logbook(&cache_dir)
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(OpenResult { logbook, display_name, recents })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_remote_url_format() {
        assert_eq!(
            cloud_remote_url("user@example.com"),
            "https://ssrf-cloud-eu.subsurface-divelog.org/git/user@example.com"
        );
    }

    #[test]
    fn cloud_branch_is_email() {
        assert_eq!(cloud_branch("user@example.com"), "user@example.com");
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
