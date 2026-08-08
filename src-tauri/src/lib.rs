mod keyboard;
mod desktop;

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    thread,
    time::Duration,
};

use keyboard::{KeyController, KeySelection};
use tauri::{Emitter, Manager};

pub(crate) struct KeyboardState {
    controller: Mutex<KeyController>,
    generation: AtomicU64,
}

impl KeyboardState {
    fn new(controller: KeyController) -> Self {
        Self {
            controller: Mutex::new(controller),
            generation: AtomicU64::new(0),
        }
    }

    fn release(&self) -> Result<(), String> {
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.controller
            .lock()
            .map_err(|_| "Le contrôleur clavier ne répond plus".to_string())?
            .release()
    }
}

pub(crate) fn release_managed_keyboard(app: &tauri::AppHandle) -> Result<(), String> {
    app.state::<KeyboardState>().release()
}

#[tauri::command]
fn hold_key(
    key: KeySelection,
    duration_seconds: Option<u64>,
    app: tauri::AppHandle,
    state: tauri::State<'_, KeyboardState>,
) -> Result<(), String> {
    let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
    state
        .controller
        .lock()
        .map_err(|_| "Le contrôleur clavier ne répond plus".to_string())?
        .hold(key)?;

    if let Some(seconds) = duration_seconds.filter(|value| *value > 0) {
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(seconds));
            let state = app.state::<KeyboardState>();
            if state
                .generation
                .compare_exchange(generation, generation + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                if let Ok(mut controller) = state.controller.lock() {
                    let _ = controller.release();
                }
                let _ = app.emit("keyhold://released", ());
            }
        });
    }

    Ok(())
}

#[tauri::command]
fn release_key(app: tauri::AppHandle) -> Result<(), String> {
    release_managed_keyboard(&app)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let keyboard = KeyController::new().expect("failed to initialize keyboard controller");

    tauri::Builder::default()
        .manage(KeyboardState::new(keyboard))
        .setup(desktop::setup)
        .invoke_handler(tauri::generate_handler![hold_key, release_key])
        .run(tauri::generate_context!())
        .expect("error while running KeyHold");
}
