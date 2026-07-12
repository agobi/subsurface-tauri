// AI-generated (Claude)
use std::sync::Mutex;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};

use crate::types::RecentEntry;

// Holds the five mutually-exclusive View menu items (radio group).
// Wrapped in Mutex so rebuild() can replace handles atomically.
pub struct ViewItems<R: Runtime> {
    all: CheckMenuItem<R>,
    list: CheckMenuItem<R>,
    profile: CheckMenuItem<R>,
    info: CheckMenuItem<R>,
    map: CheckMenuItem<R>,
}

// Holds the current recents list for index-based menu item resolution.
pub struct RecentList(pub Vec<RecentEntry>);

#[derive(Clone, serde::Serialize)]
struct SetPanelsPayload {
    list: bool,
    profile: bool,
    info: bool,
    map: bool,
}

fn build_recent_submenu<R: Runtime>(
    app: &impl Manager<R>,
    recents: &[RecentEntry],
) -> tauri::Result<Submenu<R>> {
    if recents.is_empty() {
        let placeholder =
            MenuItem::with_id(app, "recent-empty", "(No Recent Items)", false, None::<&str>)?;
        return Submenu::with_items(app, "Open Recent", true, &[&placeholder]);
    }
    let items: Vec<MenuItem<R>> = recents
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let label = match entry {
                RecentEntry::Local { path } => path
                    .rsplit(['/', '\\'])
                    .next()
                    .unwrap_or(path)
                    .to_owned(),
                RecentEntry::Cloud { email, url } => {
                    let host = url.trim_start_matches("https://").trim_start_matches("http://");
                    format!("{email}@{host}")
                }
            };
            MenuItem::with_id(app, format!("recent-{i}"), label, true, None::<&str>)
        })
        .collect::<tauri::Result<_>>()?;
    let refs: Vec<&dyn tauri::menu::IsMenuItem<R>> = items.iter().map(|m| m as _).collect();
    Submenu::with_items(app, "Open Recent", true, &refs)
}

fn create_view_items<R: Runtime>(app: &impl Manager<R>) -> tauri::Result<ViewItems<R>> {
    Ok(ViewItems {
        all: CheckMenuItem::with_id(app, "view-all", "All", true, true, Some("cmd+1"))?,
        list: CheckMenuItem::with_id(app, "view-list", "Dive List", true, false, Some("cmd+2"))?,
        profile: CheckMenuItem::with_id(
            app,
            "view-profile",
            "Dive Profile",
            true,
            false,
            Some("cmd+3"),
        )?,
        info: CheckMenuItem::with_id(app, "view-info", "Info", true, false, Some("cmd+4"))?,
        map: CheckMenuItem::with_id(app, "view-map", "Map", true, false, Some("cmd+5"))?,
    })
}

fn assemble_menu<R: Runtime>(
    app: &impl Manager<R>,
    vi: &ViewItems<R>,
    recents: &[RecentEntry],
) -> tauri::Result<Menu<R>> {
    let file_open =
        MenuItem::with_id(app, "file-open", "Open Logbook\u{2026}", true, None::<&str>)?;
    let file_new =
        MenuItem::with_id(app, "file-new", "New Logbook\u{2026}", true, None::<&str>)?;
    let file_cloud_open = MenuItem::with_id(
        app,
        "file-cloud-open",
        "Open Cloud Notebook\u{2026}",
        true,
        None::<&str>,
    )?;
    let file_dc_download = MenuItem::with_id(
        app,
        "file-dc-download",
        "Download from DC\u{2026}",
        true,
        None::<&str>,
    )?;
    let recent_sep = PredefinedMenuItem::separator(app)?;
    let recent_submenu = build_recent_submenu(app, recents)?;

    let view = Submenu::with_items(
        app,
        "View",
        true,
        &[&vi.all, &vi.list, &vi.profile, &vi.info, &vi.map],
    )?;

    #[cfg(target_os = "macos")]
    {
        let file = Submenu::with_items(
            app,
            "File",
            true,
            &[
                &file_open,
                &file_new,
                &file_cloud_open,
                &file_dc_download,
                &recent_sep,
                &recent_submenu,
            ],
        )?;
        let app_menu = Submenu::with_items(
            app,
            "Decco",
            true,
            &[
                &PredefinedMenuItem::about(app, None, None)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, "settings", "Settings\u{2026}", true, Some("cmd+,"))?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::services(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::hide(app, None)?,
                &PredefinedMenuItem::hide_others(app, None)?,
                &PredefinedMenuItem::show_all(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::quit(app, None)?,
            ],
        )?;
        let items: &[&dyn tauri::menu::IsMenuItem<R>] = &[
            &app_menu,
            &file,
            &stub_submenu(app, "Edit")?,
            &stub_submenu(app, "Import")?,
            &stub_submenu(app, "Log")?,
            &view,
            &stub_submenu(app, "Help")?,
        ];
        Menu::with_items(app, items)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let settings_item =
            MenuItem::with_id(app, "settings", "Preferences\u{2026}", true, None::<&str>)?;
        let file = Submenu::with_items(
            app,
            "File",
            true,
            &[
                &file_open,
                &file_new,
                &file_cloud_open,
                &file_dc_download,
                &recent_sep,
                &recent_submenu,
                &settings_item,
            ],
        )?;
        let items: &[&dyn tauri::menu::IsMenuItem<R>] = &[
            &file,
            &stub_submenu(app, "Edit")?,
            &stub_submenu(app, "Import")?,
            &stub_submenu(app, "Log")?,
            &view,
            &stub_submenu(app, "Help")?,
        ];
        Menu::with_items(app, items)
    }
}

pub fn build<R: Runtime>(
    app: &impl Manager<R>,
    recents: &[RecentEntry],
) -> tauri::Result<Menu<R>> {
    let vi = create_view_items(app)?;
    app.manage(Mutex::new(ViewItems {
        all: vi.all.clone(),
        list: vi.list.clone(),
        profile: vi.profile.clone(),
        info: vi.info.clone(),
        map: vi.map.clone(),
    }));
    app.manage(Mutex::new(RecentList(recents.to_vec())));
    assemble_menu(app, &vi, recents)
}

pub fn rebuild<R: Runtime>(app: &AppHandle<R>, recents: &[RecentEntry]) -> tauri::Result<()> {
    // Preserve the current view selection so opening a logbook doesn't reset
    // the menu check state to "All" regardless of what the user had selected.
    let (was_all, was_list, was_profile, was_info, was_map) = {
        let state = app.state::<Mutex<ViewItems<R>>>();
        let guard = state.lock().unwrap();
        (
            guard.all.is_checked().unwrap_or(true),
            guard.list.is_checked().unwrap_or(false),
            guard.profile.is_checked().unwrap_or(false),
            guard.info.is_checked().unwrap_or(false),
            guard.map.is_checked().unwrap_or(false),
        )
    };

    let vi = ViewItems {
        all: CheckMenuItem::with_id(app, "view-all", "All", true, was_all, Some("cmd+1"))?,
        list: CheckMenuItem::with_id(app, "view-list", "Dive List", true, was_list, Some("cmd+2"))?,
        profile: CheckMenuItem::with_id(app, "view-profile", "Dive Profile", true, was_profile, Some("cmd+3"))?,
        info: CheckMenuItem::with_id(app, "view-info", "Info", true, was_info, Some("cmd+4"))?,
        map: CheckMenuItem::with_id(app, "view-map", "Map", true, was_map, Some("cmd+5"))?,
    };

    {
        let state = app.state::<Mutex<ViewItems<R>>>();
        let mut guard = state.lock().unwrap();
        *guard = ViewItems {
            all: vi.all.clone(),
            list: vi.list.clone(),
            profile: vi.profile.clone(),
            info: vi.info.clone(),
            map: vi.map.clone(),
        };
    }
    {
        let state = app.state::<Mutex<RecentList>>();
        state.lock().unwrap().0 = recents.to_vec();
    }

    let new_menu = assemble_menu(app, &vi, recents)?;
    app.set_menu(new_menu)?;
    Ok(())
}

fn stub_submenu<R: Runtime>(app: &impl Manager<R>, label: &str) -> tauri::Result<Submenu<R>> {
    Submenu::with_items(
        app,
        label,
        true,
        &[&MenuItem::new(app, "(not implemented)", false, None::<&str>)?],
    )
}

pub fn handle_event<R: Runtime>(app: &AppHandle<R>, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "file-open" => {
            app.emit("menu:file-open", ()).ok();
        }
        "file-new" => {
            app.emit("menu:file-new", ()).ok();
        }
        "file-cloud-open" => {
            app.emit("menu:cloud-open", ()).ok();
        }
        "file-dc-download" => {
            app.emit("menu:dc-download", ()).ok();
        }
        "settings" => {
            let label = "preferences";
            if let Some(win) = app.get_webview_window(label) {
                win.show().ok();
                win.set_focus().ok();
            } else {
                WebviewWindowBuilder::new(app, label, WebviewUrl::App("prefs.html".into()))
                    .title("Preferences")
                    .inner_size(720.0, 480.0)
                    .resizable(false)
                    .build()
                    .ok();
            }
        }
        id @ ("view-all" | "view-list" | "view-profile" | "view-info" | "view-map") => {
            let state = app.state::<Mutex<ViewItems<R>>>();
            let items = state.lock().unwrap();
            let clicked = match id {
                "view-all" => &items.all,
                "view-list" => &items.list,
                "view-profile" => &items.profile,
                "view-info" => &items.info,
                "view-map" => &items.map,
                _ => return,
            };
            // muda auto-toggles before on_menu_event fires. If the item is now
            // unchecked the user clicked the already-selected item — revert and
            // ignore (radio buttons can't be deselected).
            if !clicked.is_checked().unwrap_or(false) {
                clicked.set_checked(true).ok();
                return;
            }
            let all_items = [
                &items.all,
                &items.list,
                &items.profile,
                &items.info,
                &items.map,
            ];
            for item in &all_items {
                if item.id() != clicked.id() {
                    item.set_checked(false).ok();
                }
            }
            let show_all = id == "view-all";
            app.emit(
                "menu:set-panels",
                SetPanelsPayload {
                    list: show_all || id == "view-list",
                    profile: show_all || id == "view-profile",
                    info: show_all || id == "view-info",
                    map: show_all || id == "view-map",
                },
            )
            .ok();
        }
        id if id.starts_with("recent-") => {
            let idx: usize = match id["recent-".len()..].parse() {
                Ok(n) => n,
                Err(_) => return,
            };
            let state = app.state::<Mutex<RecentList>>();
            let list = state.lock().unwrap();
            if let Some(entry) = list.0.get(idx) {
                app.emit("menu:open-recent", entry).ok();
            }
        }
        _ => {}
    }
}
