mod keyboard;

use std::sync::Mutex;

use keyboard::{KeyController, KeySelection};

type KeyboardState = Mutex<KeyController>;

#[tauri::command]
fn hold_key(key: KeySelection, state: tauri::State<'_, KeyboardState>) -> Result<(), String> {
    state
        .lock()
        .map_err(|_| "Le contrôleur clavier ne répond plus".to_string())?
        .hold(key)
}

#[tauri::command]
fn release_key(state: tauri::State<'_, KeyboardState>) -> Result<(), String> {
    state
        .lock()
        .map_err(|_| "Le contrôleur clavier ne répond plus".to_string())?
        .release()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let keyboard = KeyController::new().expect("failed to initialize keyboard controller");

    tauri::Builder::default()
        .manage(Mutex::new(keyboard))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![hold_key, release_key])
        .run(tauri::generate_context!())
        .expect("error while running KeyHold");
}
