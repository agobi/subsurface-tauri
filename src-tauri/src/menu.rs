// AI-generated (Claude)
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, Submenu},
    AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};
#[cfg(target_os = "macos")]
use tauri::menu::PredefinedMenuItem;

// Holds the five mutually-exclusive View menu items (radio group).
pub struct ViewItems<R: Runtime> {
    all: CheckMenuItem<R>,
    list: CheckMenuItem<R>,
    profile: CheckMenuItem<R>,
    info: CheckMenuItem<R>,
    map: CheckMenuItem<R>,
}

#[derive(Clone, serde::Serialize)]
struct SetPanelsPayload {
    list: bool,
    profile: bool,
    info: bool,
    map: bool,
}

pub fn build<R: Runtime>(app: &impl Manager<R>) -> tauri::Result<Menu<R>> {
    // View menu — radio group: exactly one item checked at a time.
    // Initial state: "All" checked (all panels visible).
    let view_all =
        CheckMenuItem::with_id(app, "view-all", "All", true, true, Some("cmd+1"))?;
    let view_list =
        CheckMenuItem::with_id(app, "view-list", "Dive List", true, false, Some("cmd+2"))?;
    let view_profile =
        CheckMenuItem::with_id(app, "view-profile", "Dive Profile", true, false, Some("cmd+3"))?;
    let view_info =
        CheckMenuItem::with_id(app, "view-info", "Info", true, false, Some("cmd+4"))?;
    let view_map =
        CheckMenuItem::with_id(app, "view-map", "Map", true, false, Some("cmd+5"))?;

    app.manage(ViewItems {
        all: view_all.clone(),
        list: view_list.clone(),
        profile: view_profile.clone(),
        info: view_info.clone(),
        map: view_map.clone(),
    });

    let file_open = MenuItem::with_id(app, "file-open", "Open Logbook\u{2026}", true, None::<&str>)?;
    let file_new = MenuItem::with_id(app, "file-new", "New Logbook\u{2026}", true, None::<&str>)?;

    let view = Submenu::with_items(
        app,
        "View",
        true,
        &[&view_all, &view_list, &view_profile, &view_info, &view_map],
    )?;

    // On macOS the first submenu becomes the app menu (title overridden by the app
    // name). Prepend a proper app menu so File appears as its own menu in the bar.
    #[cfg(target_os = "macos")]
    {
        let file = Submenu::with_items(app, "File", true, &[&file_open, &file_new])?;

        let app_menu = Submenu::with_items(
            app,
            "Subsurface",
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
        let file = Submenu::with_items(app, "File", true, &[&file_open, &file_new, &settings_item])?;

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
        "settings" => {
            let label = "preferences";
            if let Some(win) = app.get_webview_window(label) {
                win.show().ok();
                win.set_focus().ok();
            } else {
                WebviewWindowBuilder::new(
                    app,
                    label,
                    WebviewUrl::App("prefs.html".into()),
                )
                .title("Preferences")
                .inner_size(720.0, 480.0)
                .resizable(false)
                .build()
                .ok();
            }
        }
        id @ ("view-all" | "view-list" | "view-profile" | "view-info" | "view-map") => {
            let items = app.state::<ViewItems<R>>();
            // muda auto-toggles before on_menu_event fires. If the item is now
            // unchecked the user clicked the already-selected item — revert and
            // ignore (radio buttons can't be deselected).
            let clicked = match id {
                "view-all" => &items.all,
                "view-list" => &items.list,
                "view-profile" => &items.profile,
                "view-info" => &items.info,
                "view-map" => &items.map,
                _ => return,
            };
            if !clicked.is_checked().unwrap_or(false) {
                clicked.set_checked(true).ok();
                return;
            }
            // Uncheck all others, then derive the full panel visibility state.
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
        _ => {}
    }
}
