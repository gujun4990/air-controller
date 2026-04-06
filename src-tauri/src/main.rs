#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Emitter, Manager};
use crate::models::StartupAutoPowerOnStore;

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
        .manage(StartupAutoPowerOnStore::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            let _ = tray::show_main_window(app);
            let _ = app.emit("navigate", "main");
        }))
        .setup(|app| {
            tray::setup(app)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;

            let startup_result = startup::set_launch_on_startup(true);
            if !startup_result.success {
                eprintln!("注册系统自启动失败: {}", startup_result.message);
            }

            if startup::launched_from_system_startup() {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let result = commands::run_auto_power_on_internal().await;
                    if let Ok(mut store) = app_handle.state::<StartupAutoPowerOnStore>().0.lock() {
                        *store = Some(result.clone());
                    }
                    let _ = app_handle.emit("startup-auto-power-on-finished", result);
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if tray::should_hide_on_close(window) {
                    api.prevent_close();
                    tray::handle_close_requested(window);
                }
            }
            tauri::WindowEvent::Destroyed => {}
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_settings,
            commands::has_token,
            commands::get_state,
            commands::turn_on,
            commands::turn_off,
            commands::set_temperature,
            commands::is_system_startup_launch,
            commands::take_startup_auto_power_on_result,
            commands::minimize_window,
            commands::hide_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
