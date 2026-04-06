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

            let startup_result = startup::set_launch_on_startup(true);
            if !startup_result.success {
                eprintln!("注册系统自启动失败: {}", startup_result.message);
            }

            if startup::launched_from_system_startup() {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let _ = commands::run_auto_power_on_internal().await;
                    let _ = app_handle.emit("startup-auto-power-on-finished", "done");
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
            commands::minimize_window,
            commands::hide_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
