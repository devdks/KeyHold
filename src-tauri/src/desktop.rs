use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, AppHandle, Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::release_managed_keyboard;

const RELEASED_EVENT: &str = "keyhold://released";

fn release_keyboard(app: &AppHandle) {
    let _ = release_managed_keyboard(app);
    let _ = app.emit(RELEASED_EVENT, ());
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let emergency_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::F12);
    let shortcut_for_handler = emergency_shortcut.clone();

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if shortcut == &shortcut_for_handler && event.state() == ShortcutState::Pressed {
                    release_keyboard(app);
                }
            })
            .build(),
    )?;
    app.global_shortcut().register(emergency_shortcut)?;

    let show_item = MenuItem::with_id(app, "show", "Afficher KeyHold", true, None::<&str>)?;
    let release_item = MenuItem::with_id(app, "release", "Relâcher la touche", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &release_item, &quit_item])?;

    let mut tray = TrayIconBuilder::new()
        .tooltip("KeyHold")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "release" => release_keyboard(app),
            "quit" => {
                release_keyboard(app);
                app.exit(0);
            }
            _ => {}
        });

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;

    Ok(())
}
