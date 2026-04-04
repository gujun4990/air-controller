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
pub fn save_config(config: AppConfig) -> ServiceResult<AppConfig> {
    let store = match config_store() {
        Ok(store) => store,
        Err(message) => return ServiceResult::fail(message),
    };

    let result = store.save(&config);
    if !result.success {
        return result;
    }

    let startup_result = startup::set_launch_on_startup(config.launch_on_system_startup);
    if !startup_result.success {
        return ServiceResult::fail(startup_result.message);
    }

    ServiceResult::ok("配置已保存，并已同步系统自启动。", config)
}

#[tauri::command]
pub fn has_token() -> ServiceResult<bool> {
    secure_store().has_token()
}

#[tauri::command]
pub fn save_token(token: String) -> ServiceResult<bool> {
    if token.trim().is_empty() {
        return ServiceResult::fail("Token 不能为空。".to_string());
    }

    secure_store().save_token(token.trim())
}

#[tauri::command]
pub fn delete_token() -> ServiceResult<bool> {
    secure_store().delete_token()
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

#[tauri::command]
pub async fn run_auto_power_on() -> ServiceResult<ClimateState> {
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
pub async fn test_connection() -> ServiceResult<bool> {
    let client = match load_client() {
        Ok(client) => client,
        Err(message) => return ServiceResult::fail(message),
    };

    client.test_connection().await
}

#[tauri::command]
pub fn set_launch_on_startup(enabled: bool) -> ServiceResult<bool> {
    startup::set_launch_on_startup(enabled)
}

#[tauri::command]
pub fn get_launch_on_startup() -> ServiceResult<bool> {
    startup::get_launch_on_startup()
}

#[tauri::command]
pub fn import_legacy_config(path: Option<String>) -> ServiceResult<AppConfig> {
    let store = match config_store() {
        Ok(store) => store,
        Err(message) => return ServiceResult::fail(message),
    };

    let import_result = store.import_legacy(path);
    let Some((config, token)) = import_result.data else {
        return ServiceResult::fail(import_result.message);
    };

    if let Some(token) = token.filter(|value| !value.trim().is_empty()) {
        let _ = secure_store().save_token(&token);
    }

    store.save(&config)
}

#[tauri::command]
pub fn export_config(path: String) -> ServiceResult<bool> {
    let store = match config_store() {
        Ok(store) => store,
        Err(message) => return ServiceResult::fail(message),
    };

    store.export(&path)
}
