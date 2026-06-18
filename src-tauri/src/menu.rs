// AI-generated (Claude)
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Emitter, Manager, Runtime,
};

pub struct PanelItems<R: Runtime> {
    list: CheckMenuItem<R>,
    profile: CheckMenuItem<R>,
    info: CheckMenuItem<R>,
    map: CheckMenuItem<R>,
}

#[derive(Clone, serde::Serialize)]
struct PanelTogglePayload {
    panel: String,
    visible: bool,
}

pub fn build<R: Runtime>(app: &impl Manager<R>) -> tauri::Result<Menu<R>> {
    let list = CheckMenuItem::with_id(app, "panel-list", "Dive List", true, true, None::<&str>)?;
    let profile =
        CheckMenuItem::with_id(app, "panel-profile", "Dive Profile", true, true, None::<&str>)?;
    let info = CheckMenuItem::with_id(app, "panel-info", "Info", true, true, None::<&str>)?;
    let map = CheckMenuItem::with_id(app, "panel-map", "Map", true, true, None::<&str>)?;

    app.manage(PanelItems {
        list: list.clone(),
        profile: profile.clone(),
        info: info.clone(),
        map: map.clone(),
    });

    let file = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &MenuItem::with_id(app, "file-open", "Open Logbook\u{2026}", true, None::<&str>)?,
            &MenuItem::with_id(app, "file-new", "New Logbook\u{2026}", true, None::<&str>)?,
        ],
    )?;

    let view = Submenu::with_items(app, "View", true, &[&list, &profile, &info, &map])?;

    let items: &[&dyn tauri::menu::IsMenuItem<R>] = &[
        &file,
        &stub_submenu(app, "Edit")?,
        &stub_submenu(app, "Import")?,
        &stub_submenu(app, "Log")?,
        &view,
        &stub_submenu(app, "Help")?,
    ];

    // On macOS the first submenu becomes the app menu (title overridden by the app
    // name). Prepend a proper app menu so File appears as its own menu in the bar.
    #[cfg(target_os = "macos")]
    {
        let app_menu = Submenu::with_items(
            app,
            "Subsurface",
            true,
            &[
                &PredefinedMenuItem::about(app, None, None)?,
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
        let mut all: Vec<&dyn tauri::menu::IsMenuItem<R>> = vec![&app_menu];
        all.extend_from_slice(items);
        return Menu::with_items(app, &all);
    }

    #[allow(unreachable_code)]
    Menu::with_items(app, items)
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
        id => {
            let panel = match id {
                "panel-list" => "list",
                "panel-profile" => "profile",
                "panel-info" => "info",
                "panel-map" => "map",
                _ => return,
            };
            let items = app.state::<PanelItems<R>>();
            let clicked = match panel {
                "list" => &items.list,
                "profile" => &items.profile,
                "info" => &items.info,
                "map" => &items.map,
                _ => return,
            };
            // muda auto-toggles CheckMenuItem before firing on_menu_event, so
            // is_checked() already reflects the post-click state.
            let new_state = clicked.is_checked().unwrap_or(true);
            if !new_state {
                // User unchecked — revert if this would hide all panels.
                let any_visible = [&items.list, &items.profile, &items.info, &items.map]
                    .iter()
                    .any(|i| i.is_checked().unwrap_or(false));
                if !any_visible {
                    clicked.set_checked(true).ok();
                    return;
                }
            }
            app.emit(
                "menu:toggle-panel",
                PanelTogglePayload {
                    panel: panel.to_string(),
                    visible: new_state,
                },
            )
            .ok();
        }
    }
}
