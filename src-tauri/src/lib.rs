mod desktop;
mod keyboard;

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
fn hold_keys(
    keys: Vec<KeySelection>,
    duration_seconds: Option<u64>,
    app: tauri::AppHandle,
    state: tauri::State<'_, KeyboardState>,
) -> Result<(), String> {
    let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
    state
        .controller
        .lock()
        .map_err(|_| "Le contrôleur clavier ne répond plus".to_string())?
        .hold(keys)?;

    if let Some(seconds) = duration_seconds.filter(|value| *value > 0) {
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(seconds));
            let state = app.state::<KeyboardState>();
            if state
                .generation
                .compare_exchange(
                    generation,
                    generation + 1,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                )
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
fn release_keys(app: tauri::AppHandle) -> Result<(), String> {
    release_managed_keyboard(&app)
}

fn compact_dimensions(key_count: usize) -> (f64, f64) {
    match key_count {
        0 | 1 => (132.0, 132.0),
        2 => (148.0, 132.0),
        3 => (164.0, 132.0),
        4 => (180.0, 132.0),
        5 | 6 => (200.0, 148.0),
        _ => (220.0, 156.0),
    }
}

#[tauri::command]
fn set_compact_mode(
    compact: bool,
    key_count: usize,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri::{LogicalSize, PhysicalPosition};

    const FULL_WIDTH: f64 = 336.0;
    const FULL_HEIGHT: f64 = 428.0;
    const EDGE_MARGIN: f64 = 14.0;

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Fenêtre principale introuvable".to_string())?;

    if !compact {
        window
            .set_size(LogicalSize::new(FULL_WIDTH, FULL_HEIGHT))
            .map_err(|error| error.to_string())?;
        window.center().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let monitor = window
        .current_monitor()
        .map_err(|error| error.to_string())?
        .or(window
            .primary_monitor()
            .map_err(|error| error.to_string())?)
        .ok_or_else(|| "Écran actif introuvable".to_string())?;
    let scale = monitor.scale_factor();
    let work_area = monitor.work_area();
    let (compact_width, compact_height) = compact_dimensions(key_count);
    let compact_physical_width = (compact_width * scale).round() as i32;
    let margin_physical = (EDGE_MARGIN * scale).round() as i32;

    window
        .set_size(LogicalSize::new(compact_width, compact_height))
        .map_err(|error| error.to_string())?;
    window
        .set_position(PhysicalPosition::new(
            work_area.position.x + work_area.size.width as i32
                - compact_physical_width
                - margin_physical,
            work_area.position.y + margin_physical,
        ))
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let keyboard = KeyController::new().expect("failed to initialize keyboard controller");

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(KeyboardState::new(keyboard))
        .setup(desktop::setup)
        .invoke_handler(tauri::generate_handler![
            hold_keys,
            release_keys,
            set_compact_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running KeyHold");
}

#[cfg(test)]
mod tests {
    use super::compact_dimensions;

    #[test]
    fn compact_window_grows_gradually_for_more_keys() {
        assert_eq!(compact_dimensions(1), (132.0, 132.0));
        assert_eq!(compact_dimensions(2), (148.0, 132.0));
        assert_eq!(compact_dimensions(4), (180.0, 132.0));
        assert_eq!(compact_dimensions(6), (200.0, 148.0));
        assert_eq!(compact_dimensions(8), (220.0, 156.0));
    }
}
