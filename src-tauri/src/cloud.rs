// AI-generated (Claude)
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use crate::types::{Logbook, OpenResult, RecentEntry};

pub(crate) const CLOUD_BASE: &str = "https://ssrf-cloud-eu.subsurface-divelog.org";

// Default to the EU server, matching Qt's default (cloud_base_url in pref.cpp).
const KEYRING_SERVICE: &str = "subsurface-tauri";

// On macOS use the DataProtection keychain (bound to device login, no separate unlock
// prompt). On other platforms fall back to the keyring crate.

/// macOS DataProtection keychain — uses SecItem* APIs with kSecUseDataProtectionKeychain.
#[cfg(target_os = "macos")]
mod macos_keychain {
    use std::ffi::c_void;
    use core_foundation_sys::{
        base::{CFRelease, CFTypeRef, OSStatus},
        data::CFDataRef,
        dictionary::{
            CFDictionaryCreateMutable, CFDictionarySetValue, CFMutableDictionaryRef,
            kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks,
        },
        string::{CFStringCreateWithCString, CFStringRef, kCFStringEncodingUTF8},
    };
    use security_framework_sys::{
        base::errSecSuccess,
        item::{
            kSecAttrAccount, kSecAttrService, kSecClass, kSecClassGenericPassword,
            kSecMatchLimit, kSecReturnData, kSecValueData,
        },
        keychain_item::{SecItemAdd, SecItemCopyMatching, SecItemDelete},
    };

    // These CF/Security constants are not exposed by the sys crates at this version.
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFBooleanTrue: CFStringRef; // actually CFBooleanRef, cast to opaque
    }
    #[link(name = "Security", kind = "framework")]
    extern "C" {
        static kSecUseDataProtectionKeychain: CFStringRef;
        static kSecMatchLimitOne: CFStringRef;
    }

    // RAII wrappers to ensure CF objects are released.
    struct OwnedStr(CFStringRef);
    impl Drop for OwnedStr {
        fn drop(&mut self) { unsafe { CFRelease(self.0 as _); } }
    }
    struct OwnedDict(CFMutableDictionaryRef);
    impl Drop for OwnedDict {
        fn drop(&mut self) { unsafe { CFRelease(self.0 as _); } }
    }

    fn cf_str(s: &str) -> OwnedStr {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0);
        OwnedStr(unsafe {
            CFStringCreateWithCString(std::ptr::null(), bytes.as_ptr() as _, kCFStringEncodingUTF8)
        })
    }

    fn new_dict() -> OwnedDict {
        OwnedDict(unsafe {
            CFDictionaryCreateMutable(
                std::ptr::null(),
                0,
                &kCFTypeDictionaryKeyCallBacks,
                &kCFTypeDictionaryValueCallBacks,
            )
        })
    }

    fn set(dict: &OwnedDict, key: *const c_void, value: *const c_void) {
        unsafe { CFDictionarySetValue(dict.0, key, value); }
    }

    fn base_query(dict: &OwnedDict, svc: &OwnedStr, acc: &OwnedStr) {
        unsafe {
            set(dict, kSecClass as _, kSecClassGenericPassword as _);
            set(dict, kSecAttrService as _, svc.0 as _);
            set(dict, kSecAttrAccount as _, acc.0 as _);
            set(dict, kSecUseDataProtectionKeychain as _, kCFBooleanTrue as _);
        }
    }

    pub fn keychain_set(service: &str, account: &str, password: &str) -> Result<(), String> {
        let svc = cf_str(service);
        let acc = cf_str(account);

        // Remove any existing item first (ignore not-found).
        let del = new_dict();
        base_query(&del, &svc, &acc);
        unsafe { SecItemDelete(del.0 as _); }

        // Build add dict: class + attrs + password data.
        let pw_str = cf_str(password);
        let data_ref: CFDataRef = unsafe {
            core_foundation_sys::data::CFDataCreate(
                std::ptr::null(),
                password.as_ptr(),
                password.len() as _,
            )
        };
        let add = new_dict();
        base_query(&add, &svc, &acc);
        unsafe {
            set(&add, kSecValueData as _, data_ref as _);
            CFRelease(data_ref as _);
        }
        drop(pw_str); // only needed if used; suppress warning

        let status: OSStatus = unsafe { SecItemAdd(add.0 as _, std::ptr::null_mut()) };
        if status == errSecSuccess {
            Ok(())
        } else if status == -34018 {
            // errSecMissingEntitlement: keychain-access-groups entitlement absent (dev builds).
            Err("MISSING_ENTITLEMENT".to_string())
        } else {
            Err(format!("SecItemAdd: {status}"))
        }
    }

    pub fn keychain_get(service: &str, account: &str) -> Result<String, String> {
        let svc = cf_str(service);
        let acc = cf_str(account);
        let q = new_dict();
        base_query(&q, &svc, &acc);
        unsafe {
            set(&q, kSecReturnData as _, kCFBooleanTrue as _);
            set(&q, kSecMatchLimit as _, kSecMatchLimitOne as _);
        }

        let mut result: CFTypeRef = std::ptr::null();
        let status: OSStatus = unsafe { SecItemCopyMatching(q.0 as _, &mut result) };
        if status == -34018 {
            // errSecMissingEntitlement: entitlement absent, caller may try login keychain.
            return Err("MISSING_ENTITLEMENT".to_string());
        }
        if status != errSecSuccess || result.is_null() {
            return Err("NO_SAVED_CREDENTIALS".to_string());
        }
        let data_ref = result as CFDataRef;
        let len = unsafe { core_foundation_sys::data::CFDataGetLength(data_ref) } as usize;
        let ptr = unsafe { core_foundation_sys::data::CFDataGetBytePtr(data_ref) };
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();
        unsafe { CFRelease(result); }
        String::from_utf8(bytes).map_err(|e| e.to_string())
    }
}

// On macOS, use only the DataProtection keychain (auto-unlocks with login, never prompts).
// If the keychain-access-groups entitlement is absent (dev builds), credentials are kept
// in the session cache only — they won't survive a restart, but no system dialog appears.

#[cfg(target_os = "macos")]
fn keychain_set(service: &str, account: &str, password: &str) -> Result<(), String> {
    match macos_keychain::keychain_set(service, account, password) {
        Ok(()) => {
            log::info!("keychain_set: stored (account={account})");
            Ok(())
        }
        Err(e) if e == "MISSING_ENTITLEMENT" => {
            log::warn!("keychain_set: entitlement absent, credentials not persisted (account={account})");
            Ok(()) // non-fatal: credentials won't survive restart in dev builds
        }
        Err(e) => {
            log::error!("keychain_set: {e} (account={account})");
            Err(e)
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn keychain_set(service: &str, account: &str, password: &str) -> Result<(), String> {
    use keyring::Entry;
    Entry::new(service, account)
        .map_err(|e| e.to_string())?
        .set_password(password)
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
fn keychain_get(service: &str, account: &str) -> Result<String, String> {
    match macos_keychain::keychain_get(service, account) {
        Ok(pw) => {
            log::info!("keychain_get: found (account={account})");
            Ok(pw)
        }
        Err(e) => {
            log::warn!("keychain_get: {e} (account={account})");
            Err("NO_SAVED_CREDENTIALS".to_string())
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn keychain_get(service: &str, account: &str) -> Result<String, String> {
    use keyring::Entry;
    Entry::new(service, account)
        .map_err(|e| e.to_string())?
        .get_password()
        .map_err(|_| "NO_SAVED_CREDENTIALS".to_string())
}

fn get_password(email: &str, url: &str) -> Result<String, String> {
    keychain_get(KEYRING_SERVICE, &cloud_display_name(email, url))
}

fn cloud_remote_url(email: &str) -> String {
    format!("{CLOUD_BASE}/git/{email}")
}

// The Subsurface cloud server uses the email address as the git branch name.
fn cloud_branch(email: &str) -> &str {
    email
}

fn cloud_host(url: &str) -> &str {
    url.trim_start_matches("https://").trim_start_matches("http://")
}

pub(crate) fn cloud_display_name(email: &str, url: &str) -> String {
    format!("{email}@{}", cloud_host(url))
}

pub(crate) fn cloud_cache_dir(
    data_dir: &std::path::Path,
    url: &str,
    email: &str,
) -> std::path::PathBuf {
    data_dir.join("cloud").join(cloud_host(url)).join(email)
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
    let cache_dir = cloud_cache_dir(&data_dir, CLOUD_BASE, &email);
    let url = cloud_remote_url(&email);
    let branch = cloud_branch(&email).to_owned();

    let display_name = cloud_display_name(&email, CLOUD_BASE);
    let email2 = email.clone();
    let password2 = password.clone();
    let cache_dir2 = cache_dir.clone();

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
        store.set("cloudEmail", serde_json::json!(email2));
        store.set("logbookPath", serde_json::json!(cache_dir2.to_string_lossy().as_ref()));
        store.save().map_err(|e| e.to_string())?;
        let kc_key = cloud_display_name(&email2, CLOUD_BASE);
        keychain_set(KEYRING_SERVICE, &kc_key, &password2)?;
        let recents = crate::update_recents(
            &store,
            RecentEntry::Cloud { email: email2.clone(), url: cloud_url },
        )?;
        let logbook = crate::ssrf_git::parse_logbook(&cache_dir2)?;
        Ok((logbook, recents))
    })
    .await
    .map_err(|e| e.to_string())??;

    #[cfg(desktop)]
    crate::menu::rebuild(&app, &recents).map_err(|e| e.to_string())?;

    Ok(OpenResult { logbook, display_name, recents })
}

/// Opens a cloud logbook from the recents list: fetches from the server using the password saved
/// in settings.json (written at login), so no keyring access is needed.
/// Returns Err("NO_SAVED_CREDENTIALS") if no password is stored, so the caller can show the
/// login dialog instead.
#[tauri::command]
pub async fn open_recent_cloud_logbook(
    app: tauri::AppHandle,
    email: String,
) -> Result<OpenResult, String> {
    use tauri::Manager;

    let email = email.to_lowercase();
    let password = get_password(&email, CLOUD_BASE)?;
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let cache_dir = cloud_cache_dir(&data_dir, CLOUD_BASE, &email);
    let url = cloud_remote_url(&email);
    let branch = cloud_branch(&email).to_owned();
    let display_name = cloud_display_name(&email, CLOUD_BASE);
    let email_clone = email.clone();
    let cloud_url = CLOUD_BASE.to_owned();
    let app_clone = app.clone();

    let (logbook, recents) = tauri::async_runtime::spawn_blocking(
        move || -> Result<(Logbook, Vec<RecentEntry>), String> {
            clone_or_fetch(&url, &branch, &cache_dir, &email_clone, &password)?;
            let store = app_clone.store("settings.json").map_err(|e| e.to_string())?;
            store.set("cloudEmail", serde_json::json!(email_clone));
            store.set(
                "logbookPath",
                serde_json::json!(cache_dir.to_string_lossy().as_ref()),
            );
            store.save().map_err(|e| e.to_string())?;
            let recents = crate::update_recents(
                &store,
                RecentEntry::Cloud {
                    email: email_clone.clone(),
                    url: cloud_url,
                },
            )?;
            let logbook = crate::ssrf_git::parse_logbook(&cache_dir)?;
            Ok((logbook, recents))
        },
    )
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

    let password = get_password(&email, CLOUD_BASE).map_err(|_| {
        "No saved credentials found. Please open the cloud logbook again.".to_string()
    })?;

    let display_name = cloud_display_name(&email, CLOUD_BASE);
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let cache_dir = cloud_cache_dir(&data_dir, CLOUD_BASE, &email);
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
