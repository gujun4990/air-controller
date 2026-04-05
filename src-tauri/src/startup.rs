use auto_launch::AutoLaunchBuilder;

use crate::models::ServiceResult;

pub const SYSTEM_AUTOSTART_ARG: &str = "--system-autostart";

fn launcher() -> Result<auto_launch::AutoLaunch, String> {
    let app_path =
        std::env::current_exe().map_err(|error| format!("读取当前程序路径失败: {error}"))?;
    let app_path = app_path.to_string_lossy().into_owned();
    AutoLaunchBuilder::new()
        .set_app_name("air-controller")
        .set_app_path(&app_path)
        .set_use_launch_agent(true)
        .set_args(&[SYSTEM_AUTOSTART_ARG])
        .build()
        .map_err(|error| format!("初始化自启动失败: {error}"))
}

pub fn launched_from_system_startup() -> bool {
    std::env::args().any(|arg| arg == SYSTEM_AUTOSTART_ARG)
}

pub fn set_launch_on_startup(enabled: bool) -> ServiceResult<bool> {
    let launcher = match launcher() {
        Ok(launcher) => launcher,
        Err(message) => return ServiceResult::fail(message),
    };

    let result = if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    };
    if let Err(error) = result {
        return ServiceResult::fail(format!("更新系统自启动失败: {error}"));
    }

    ServiceResult::ok("系统自启动状态已更新。", enabled)
}
