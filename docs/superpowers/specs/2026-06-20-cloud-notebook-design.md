# Cloud Notebook — Design Spec

**Date:** 2026-06-20
**Branch:** feat/cloud-notebook (to be created)
**Scope:** Open a Subsurface Cloud logbook (read-only) with OS keychain credential storage and a toolbar sync button.

---

## Goal

Allow users to open their logbook from the Subsurface Cloud (`cloud.subsurface-divelog.org`)
in addition to local directories. Credentials are saved to the OS keychain. A Sync button
in the Toolbar refreshes the data on demand. Read-only for now — no push/write-back.

---

## Out of scope

- Writing / pushing changes back to the cloud
- Supporting non-Subsurface git remotes
- iOS support (Android + desktop only)
- Conflict resolution

---

## Architecture

### New Rust deps (`Cargo.toml`)

```toml
git2 = { version = "0.20", default-features = false, features = ["https"] }
keyring = "3"
```

`git2` provides libgit2 bindings for clone/fetch without requiring a system `git` install.
`keyring` provides cross-platform OS keychain access (macOS Keychain, Windows Credential
Manager, Linux Secret Service).

### New Rust module: `src-tauri/src/cloud.rs`

Three Tauri commands:

**`get_cloud_credentials() -> Result<Option<String>, String>`**
- Reads saved email from `settings.json` (`cloudEmail` key).
- Returns email string if found, `None` if no cloud logbook has been opened before.
- Password is never returned to the frontend — it stays in the keychain.

**`open_cloud_logbook(email, password) -> Result<Logbook, String>`**
- Saves email to `settings.json` (`cloudEmail`).
- Saves password to OS keychain (service: `"subsurface-tauri"`, account: email).
- Resolves local cache path: `app_data_dir/cloud/{email}/`.
- If cache dir does not exist: `git2::Repository::clone()`.
- If cache dir exists: `git2::Repository::open()` + `remote.fetch()` + `repo.checkout_head(CheckoutBuilder::force())`.
- Credentials injected via `RemoteCallbacks::credentials` — never embedded in the URL.
- Calls `ssrf_git::parse_logbook(&cache_path)` and returns the result.
- Cloud URL format: `https://cloud.subsurface-divelog.org/git/{email}[{email}]`

**`sync_cloud_logbook() -> Result<Logbook, String>`**
- Reads email from `settings.json`, password from keychain.
- Same fetch + fast-forward + re-parse as `open_cloud_logbook`, no args needed from frontend.

All three run blocking git/fs work inside `tauri::async_runtime::spawn_blocking`.

### `src-tauri/src/lib.rs`

- Add `mod cloud;`
- Register `cloud::get_cloud_credentials`, `cloud::open_cloud_logbook`, `cloud::sync_cloud_logbook` in `invoke_handler!`.

### `src-tauri/src/menu.rs`

- Add `file-cloud-open` `MenuItem` ("Open Cloud Notebook…") in the File submenu after `file-open`, in both the macOS and non-macOS branches.
- Emit `menu:cloud-open` in `handle_event`.

---

## Frontend

### `src/lib/stores/app.svelte.ts`

New state and methods added to `AppStore`:

```ts
isCloudLogbook = $state(false);

async openCloud(email: string, password: string): Promise<void>
async syncCloud(): Promise<void>
```

`openCloud` invokes `open_cloud_logbook`, sets `logbook`, `selectedDiveId`,
and `isCloudLogbook = true`.

`syncCloud` invokes `sync_cloud_logbook`, updates `logbook`; retains `selectedDiveId`
if the dive still exists, otherwise resets to first dive.

`open()` and `newLogbook()` set `isCloudLogbook = false`.

### `src/lib/components/CloudLoginDialog.svelte` (new)

Modal overlay with:
- Email field (prefilled from `get_cloud_credentials` on mount)
- Password field (`type="password"`)
- Inline error message area (shown on auth/network failure)
- Loading spinner while the Tauri command runs
- Cancel / Open Cloud buttons
- Emits `close` and `success` events to parent

### `src/lib/components/Toolbar.svelte`

- Accept `isCloud` and `onSync` props.
- Show "Sync" button (after the existing divider) when `isCloud` is true.
- Show a brief error banner below the toolbar when sync fails (dismissable).

### `src/App.svelte`

- Import `CloudLoginDialog`.
- `let showCloudDialog = $state(false)`.
- Listen for `menu:cloud-open` → `showCloudDialog = true`.
- On dialog `success`: set window title to `{email} — Subsurface`.
- Pass `isCloud={app.isCloudLogbook}` and `onSync` handler to `Toolbar`.

---

## Data flow

### Open cloud logbook

```
File menu / Android button
  → menu:cloud-open event
  → showCloudDialog = true
  → CloudLoginDialog mounts, calls get_cloud_credentials (prefill email)
  → user enters credentials, clicks Open
  → dialog enters loading state
  → invoke open_cloud_logbook({ email, password })
      → save email to settings.json
      → save password to keychain
      → clone or fetch git repo into app_data_dir/cloud/{email}/
      → ssrf_git::parse_logbook
      → return Logbook
  → AppStore.openCloud() sets logbook + isCloudLogbook = true
  → dialog closes, window title updated
```

### Sync

```
Toolbar Sync button (visible when isCloudLogbook)
  → Toolbar shows loading state
  → invoke sync_cloud_logbook()
      → read email from settings.json, password from keychain
      → fetch + fast-forward + re-parse
      → return Logbook
  → AppStore.syncCloud() updates logbook, retains selected dive if present
  → Toolbar returns to idle (or shows error banner on failure)
```

---

## Error handling

| Situation | Rust maps to | Frontend shows |
|---|---|---|
| Bad credentials | `"Authentication failed. Check your email and password."` | Inline under password field |
| Network unreachable | `"Could not reach Subsurface Cloud. Check your connection."` | Inline in dialog |
| Clone/fetch errors | raw error string | Inline in dialog (open) or dismissable banner (sync) |
| Keychain unavailable | `get_cloud_credentials` returns `Ok(None)` | Email field starts empty — non-fatal |
| Already up to date | success, re-parse runs | Normal logbook update |

---

## Android considerations

Android has no native menu bar. The "Open Cloud Notebook" entry point on Android will be
a UI-level button or prompt (exact placement designed separately). The `CloudLoginDialog`
component and all Tauri commands are platform-agnostic — no Android-specific code needed
in this feature.

---

## Files changed

| File | Change |
|---|---|
| `src-tauri/Cargo.toml` | add `git2`, `keyring` |
| `src-tauri/src/cloud.rs` | new — three Tauri commands |
| `src-tauri/src/lib.rs` | `mod cloud;` + register commands |
| `src-tauri/src/menu.rs` | add "Open Cloud Notebook…" + emit event |
| `src/lib/stores/app.svelte.ts` | `isCloudLogbook`, `openCloud()`, `syncCloud()` |
| `src/lib/components/CloudLoginDialog.svelte` | new — login modal |
| `src/lib/components/Toolbar.svelte` | Sync button + error banner |
| `src/App.svelte` | dialog wiring + menu event listener |
