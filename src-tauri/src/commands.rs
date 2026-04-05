use crate::{
    auto_power_on,
    config::ConfigStore,
    ha_client::HomeAssistantClient,
    models::{AppConfig, ClimateState, ServiceResult},
    secure_store::SecureStore,
    startup,
};

fn config_store() -> Result<ConfigStore, String> {
    ConfigStore::new()
}

fn secure_store() -> SecureStore {
    SecureStore::new()
}

fn rollback_token(store: &SecureStore, previous_token: Option<String>) -> Result<(), String> {
    if let Some(token) = previous_token {
        let result = store.save_token(&token);
        if !result.success {
            return Err(result.message);
        }
    } else {
        let result = store.delete_token();
        if !result.success {
            return Err(result.message);
        }
    }

    Ok(())
}

fn load_client() -> Result<HomeAssistantClient, String> {
    let config_store = config_store()?;
    let config_result = config_store.load();
    let config = config_result.data.ok_or(config_result.message)?;
    let token = secure_store().load_token_value()?;
    HomeAssistantClient::new(config, token)
}

#[tauri::command]
pub fn get_config() -> ServiceResult<AppConfig> {
    match config_store() {
        Ok(store) => store.load(),
        Err(message) => ServiceResult::fail(message),
    }
}

#[tauri::command]
pub fn save_settings(config: AppConfig, token: String) -> ServiceResult<AppConfig> {
    let store = match config_store() {
        Ok(store) => store,
        Err(message) => return ServiceResult::fail(message),
    };
    let secure_store = secure_store();

    if token.trim().is_empty() {
        return ServiceResult::fail("访问令牌不能为空。".to_string());
    }

    let previous_config = store.load().data;
    let previous_token = secure_store.load_token_value().ok();

    let token_result = secure_store.save_token(token.trim());
    if !token_result.success {
        return ServiceResult::fail(token_result.message);
    }

    let result = store.save(&config);
    if !result.success {
        if let Err(message) = rollback_token(&secure_store, previous_token) {
            return ServiceResult::fail(format!("{} 访问令牌回滚失败: {}", result.message, message));
        }

        return result;
    }

    let startup_result = startup::set_launch_on_startup(true);
    if !startup_result.success {
        if let Some(previous_config) = previous_config {
            let rollback_result = store.save(&previous_config);
            if !rollback_result.success {
                return ServiceResult::fail(format!(
                    "{} 配置回滚也失败: {}",
                    startup_result.message, rollback_result.message
                ));
            }
        }

        if let Err(message) = rollback_token(&secure_store, previous_token) {
            return ServiceResult::fail(format!(
                "{} 访问令牌回滚失败: {}",
                startup_result.message, message
            ));
        }

        return ServiceResult::fail(startup_result.message);
    }

    ServiceResult::ok("配置已保存，并已同步系统自启动。", config)
}

#[tauri::command]
pub fn has_token() -> ServiceResult<bool> {
    secure_store().has_token()
}

#[tauri::command]
pub async fn get_state() -> ServiceResult<ClimateState> {
    let client = match load_client() {
        Ok(client) => client,
        Err(message) => return ServiceResult::fail(message),
    };

    client.get_state().await
}

#[tauri::command]
pub async fn turn_on() -> ServiceResult<ClimateState> {
    let client = match load_client() {
        Ok(client) => client,
        Err(message) => return ServiceResult::fail(message),
    };

    client.turn_on().await
}

#[tauri::command]
pub async fn turn_off() -> ServiceResult<ClimateState> {
    let client = match load_client() {
        Ok(client) => client,
        Err(message) => return ServiceResult::fail(message),
    };

    client.turn_off().await
}

#[tauri::command]
pub async fn set_temperature(temperature: f64) -> ServiceResult<ClimateState> {
    let client = match load_client() {
        Ok(client) => client,
        Err(message) => return ServiceResult::fail(message),
    };

    client.set_temperature(temperature).await
}

pub async fn run_auto_power_on_internal() -> ServiceResult<ClimateState> {
    let store = match config_store() {
        Ok(store) => store,
        Err(message) => return ServiceResult::fail(message),
    };
    let config_result = store.load();
    let config = match config_result.data {
        Some(config) => config,
        None => return ServiceResult::fail(config_result.message),
    };

    let token = match secure_store().load_token_value() {
        Ok(token) => token,
        Err(message) => return ServiceResult::fail(message),
    };

    let client = match HomeAssistantClient::new(config.clone(), token) {
        Ok(client) => client,
        Err(message) => return ServiceResult::fail(message),
    };

    auto_power_on::execute(config, &client).await
}

#[tauri::command]
pub fn hide_window(window: tauri::WebviewWindow) -> ServiceResult<bool> {
    match window.hide() {
        Ok(()) => ServiceResult::ok("窗口已隐藏。", true),
        Err(error) => ServiceResult::fail(format!("隐藏窗口失败: {error}")),
    }
}
