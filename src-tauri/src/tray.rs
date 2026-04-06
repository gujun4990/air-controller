use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, Runtime, Window,
};

pub struct CloseState(pub AtomicBool);

impl Default for CloseState {
    fn default() -> Self {
        Self(AtomicBool::new(false))
    }
}

const OPEN_ID: &str = "open";
const QUIT_ID: &str = "quit";

pub fn setup(app: &App) -> Result<(), String> {
    let open = MenuItemBuilder::with_id(OPEN_ID, "打开")
        .build(app)
        .map_err(|error| format!("创建托盘菜单失败: {error}"))?;
    let quit = MenuItemBuilder::with_id(QUIT_ID, "退出")
        .build(app)
        .map_err(|error| format!("创建托盘菜单失败: {error}"))?;
    let menu = MenuBuilder::new(app)
        .item(&open)
        .item(&quit)
        .build()
        .map_err(|error| format!("创建托盘菜单失败: {error}"))?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false);
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_main_window(&tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            OPEN_ID => {
                let _ = show_main_window(app);
                let _ = app.emit("navigate", "main");
            }
            QUIT_ID => {
                request_exit(app);
            }
            _ => {}
        })
        .build(app)
        .map_err(|error| format!("创建托盘图标失败: {error}"))?;

    Ok(())
}

pub fn handle_close_requested<R: Runtime>(window: &Window<R>) {
    let _ = window.hide();
}

pub fn should_hide_on_close<R: Runtime>(window: &Window<R>) -> bool {
    !window.state::<CloseState>().0.load(Ordering::SeqCst)
}

pub fn request_exit(app: &AppHandle) {
    app.state::<CloseState>().0.store(true, Ordering::SeqCst);
    app.exit(0);
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "未找到主窗口。".to_string())?;
    window
        .show()
        .map_err(|error| format!("显示主窗口失败: {error}"))?;
    window
        .unminimize()
        .map_err(|error| format!("恢复主窗口失败: {error}"))?;
    window
        .set_focus()
        .map_err(|error| format!("聚焦主窗口失败: {error}"))?;
    Ok(())
}
