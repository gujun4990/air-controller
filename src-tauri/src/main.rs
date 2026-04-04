#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Emitter;

mod auto_power_on;
mod commands;
mod config;
mod ha_client;
mod models;
mod secure_store;
mod startup;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(tray::CloseState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = tray::show_main_window(app);
            let _ = app.emit("navigate", "main");
        }))
        .setup(|app| {
            tray::setup(app)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if tray::should_hide_on_close(window) {
                    api.prevent_close();
                    tray::handle_close_requested(window);
                }
            }
            tauri::WindowEvent::Resized(_) => {
                if window.is_minimized().unwrap_or(false) {
                    let _ = window.hide();
                }
            }
            tauri::WindowEvent::Destroyed => {}
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::has_token,
            commands::save_token,
            commands::delete_token,
            commands::get_state,
            commands::turn_on,
            commands::turn_off,
            commands::set_temperature,
            commands::run_auto_power_on,
            commands::test_connection,
            commands::set_launch_on_startup,
            commands::get_launch_on_startup,
            commands::import_legacy_config,
            commands::export_config,
            commands::hide_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
