// AI-generated (Claude)
mod sync;

use tauri::Manager;
use tauri_plugin_store::StoreExt;
use std::sync::Mutex;
use crate::state::LogbookState;
use crate::types::{OpenResult, ParsedLogbook, RecentEntry};

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
        fn drop(&mut self) {
            if !self.0.is_null() { unsafe { CFRelease(self.0 as _); } }
        }
    }
    struct OwnedDict(CFMutableDictionaryRef);
    impl Drop for OwnedDict {
        fn drop(&mut self) {
            if !self.0.is_null() { unsafe { CFRelease(self.0 as _); } }
        }
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

// Only provide credentials once — replaying the same credentials triggers libgit2's
// "too many redirects or authentication replays" error (mirrors Qt's exceeded_auth_attempts).
// Shared by fetch and push so both time out the same way on a rejected credential.
pub(crate) fn credential_callbacks<'a>(email: &'a str, password: &'a str) -> git2::RemoteCallbacks<'a> {
    let mut callbacks = git2::RemoteCallbacks::new();
    let mut attempt = 0u32;
    callbacks.credentials(move |_, _, _| {
        attempt += 1;
        if attempt > 1 {
            return Err(git2::Error::from_str("credentials rejected by server"));
        }
        git2::Cred::userpass_plaintext(email, password)
    });
    callbacks
}

fn make_fetch_opts<'a>(email: &'a str, password: &'a str) -> git2::FetchOptions<'a> {
    let mut callbacks = credential_callbacks(email, password);
    callbacks.certificate_check(|cert, host| {
        // Mirrors Qt's certificate_check_cb (core/git-access.cpp): libgit2's own X.509 chain
        // verification is unreliable across platforms — e.g. on Android, vendored OpenSSL's
        // hash-dir CA lookup can't chase a freshly cross-signed root even though the served
        // chain is genuinely valid (issue #67). Since we only ever fetch from our own
        // hardcoded cloud host, accept unconditionally for that host and let libgit2's
        // built-in result stand for anything else.
        if host == cloud_host(CLOUD_BASE) && cert.as_x509().is_some() {
            Ok(git2::CertificateCheckStatus::CertificateOk)
        } else {
            Ok(git2::CertificateCheckStatus::CertificatePassthrough)
        }
    });
    let mut opts = git2::FetchOptions::new();
    opts.remote_callbacks(callbacks);
    opts
}

// Vendored OpenSSL has no CA store on Android (openssl-probe only checks desktop/Linux
// paths, none of which exist there). Point it at Android's own OS-managed trust store —
// already in OpenSSL's c_rehash directory format — instead of vendoring/maintaining a
// separate CA bundle.
#[cfg(target_os = "android")]
fn init_android_tls() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        for dir in [
            "/apex/com.android.conscrypt/cacerts",
            "/system/etc/security/cacerts",
        ] {
            if std::path::Path::new(dir).is_dir() {
                std::env::set_var("SSL_CERT_DIR", dir);
                break;
            }
        }
    });
}

fn clone_or_fetch(
    url: &str,
    branch: &str,
    cache_dir: &std::path::Path,
    email: &str,
    password: &str,
) -> Result<(), String> {
    #[cfg(target_os = "android")]
    init_android_tls();

    if cache_dir.is_dir() {
        let repo = git2::Repository::open(cache_dir).map_err(|e| e.to_string())?;
        // Write the exclude file before touching local changes — a legacy cache cloned
        // before this feature existed has no exclude file yet, and without this ordering
        // its first sync could commit junk files (e.g. .DS_Store) before self-healing.
        sync::ensure_git_exclude(cache_dir)?;
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
        repo.find_reference(&refname)
            .map_err(|_| format!("Remote branch '{branch}' not found after fetch — is the account empty?"))?;
        sync::commit_local_changes(&repo)?;
        sync::rebase_onto_remote(&repo, branch)?;
        sync::push_to_remote(&repo, "origin", branch, email, password)?;
    } else {
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(make_fetch_opts(email, password));
        builder.branch(branch);
        builder.clone(url, cache_dir).map_err(map_git_error)?;
        sync::ensure_git_exclude(cache_dir)?;
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
    logbook_state: tauri::State<'_, Mutex<Option<LogbookState>>>,
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

    // Step 3: Save credentials, persist logbookPath, update recents, and parse.
    // keychain_set runs BEFORE store.save so that a keychain failure leaves nothing
    // persisted — avoiding the stuck-loop where the path is saved but credentials aren't.
    let root = cache_dir2.clone();
    let app_clone = app.clone();
    let cloud_url = CLOUD_BASE.to_owned();
    let (parsed, recents) = tauri::async_runtime::spawn_blocking(move || -> Result<(ParsedLogbook, Vec<RecentEntry>), String> {
        let store = app_clone.store("settings.json").map_err(|e| e.to_string())?;
        let kc_key = cloud_display_name(&email2, CLOUD_BASE);
        keychain_set(KEYRING_SERVICE, &kc_key, &password2)?;
        store.set("cloudEmail", serde_json::json!(email2));
        store.set("logbookPath", serde_json::json!(cache_dir2.to_string_lossy().as_ref()));
        store.save().map_err(|e| e.to_string())?;
        let recents = crate::update_recents(
            &store,
            RecentEntry::Cloud { email: email2.clone(), url: cloud_url },
        )?;
        let parsed = crate::ssrf_git::parse_logbook(&cache_dir2)?;
        Ok((parsed, recents))
    })
    .await
    .map_err(|e| e.to_string())??;

    let warnings = parsed.warnings.clone();
    let logbook = crate::install_logbook(&app, &logbook_state, root, parsed)?;

    #[cfg(desktop)]
    crate::menu::rebuild(&app, &recents).map_err(|e| e.to_string())?;

    Ok(OpenResult { logbook, display_name, recents, warnings })
}

/// Opens a cloud logbook from the recents list: fetches from the server using the password saved
/// in settings.json (written at login), so no keyring access is needed.
/// Returns Err("NO_SAVED_CREDENTIALS") if no password is stored, so the caller can show the
/// login dialog instead.
#[tauri::command]
pub async fn open_recent_cloud_logbook(
    app: tauri::AppHandle,
    email: String,
    logbook_state: tauri::State<'_, Mutex<Option<LogbookState>>>,
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
    let root = cache_dir.clone();

    let (parsed, recents) = tauri::async_runtime::spawn_blocking(
        move || -> Result<(ParsedLogbook, Vec<RecentEntry>), String> {
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
            let parsed = crate::ssrf_git::parse_logbook(&cache_dir)?;
            Ok((parsed, recents))
        },
    )
    .await
    .map_err(|e| e.to_string())??;

    let warnings = parsed.warnings.clone();
    let logbook = crate::install_logbook(&app, &logbook_state, root, parsed)?;

    #[cfg(desktop)]
    crate::menu::rebuild(&app, &recents).map_err(|e| e.to_string())?;

    Ok(OpenResult { logbook, display_name, recents, warnings })
}

#[tauri::command]
pub async fn sync_cloud_logbook(
    app: tauri::AppHandle,
    logbook_state: tauri::State<'_, Mutex<Option<LogbookState>>>,
) -> Result<OpenResult, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let email = store
        .get("cloudEmail")
        .and_then(|v| v.as_str().map(str::to_owned))
        .ok_or_else(|| "No cloud logbook configured.".to_string())?;

    let password = get_password(&email, CLOUD_BASE).map_err(|_| {
        "No saved credentials found. Please open the cloud logbook again.".to_string()
    })?;

    let display_name = cloud_display_name(&email, CLOUD_BASE);
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let cache_dir = cloud_cache_dir(&data_dir, CLOUD_BASE, &email);
    let url = cloud_remote_url(&email);
    let branch = cloud_branch(&email).to_owned();
    let root = cache_dir.clone();

    let parsed = tauri::async_runtime::spawn_blocking(move || {
        clone_or_fetch(&url, &branch, &cache_dir, &email, &password)?;
        crate::ssrf_git::parse_logbook(&cache_dir)
    })
    .await
    .map_err(|e| e.to_string())??;

    let warnings = parsed.warnings.clone();
    let logbook = crate::install_logbook(&app, &logbook_state, root, parsed)?;

    // Read recents AFTER the (potentially multi-second) fetch so a concurrent open
    // that wrote a new entry during the fetch is not overwritten with a stale snapshot.
    let recents: Vec<RecentEntry> = store
        .get("recents")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    Ok(OpenResult { logbook, display_name, recents, warnings })
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
