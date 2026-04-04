use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub base_url: String,
    pub climate_entity_id: String,
    pub launch_on_system_startup: bool,
    pub auto_power_on_on_startup: bool,
    pub startup_delay_seconds: i32,
    pub retry_count: i32,
    pub default_temperature: f64,
    pub min_temperature: f64,
    pub max_temperature: f64,
    pub temperature_step: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            climate_entity_id: "climate.living_room_ac".into(),
            launch_on_system_startup: false,
            auto_power_on_on_startup: false,
            startup_delay_seconds: 8,
            retry_count: 3,
            default_temperature: 26.0,
            min_temperature: 16.0,
            max_temperature: 30.0,
            temperature_step: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClimateState {
    pub entity_id: String,
    pub state: String,
    pub hvac_mode: String,
    pub hvac_action: String,
    pub current_temperature: Option<f64>,
    pub target_temperature: Option<f64>,
    pub is_available: bool,
    pub is_on: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResult<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ServiceResult<T> {
    pub fn ok(message: impl Into<String>, data: T) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn fail(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LegacyRoot {
    pub home_assistant: Option<LegacyHomeAssistant>,
    pub startup: Option<LegacyStartup>,
    pub climate: Option<LegacyClimate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LegacyHomeAssistant {
    pub base_url: Option<String>,
    pub long_lived_token: Option<String>,
    pub encrypted_long_lived_token: Option<String>,
    pub climate_entity_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LegacyStartup {
    pub launch_on_windows_startup: Option<bool>,
    pub auto_power_on_on_startup: Option<bool>,
    pub startup_delay_seconds: Option<i32>,
    pub retry_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LegacyClimate {
    pub default_temperature: Option<f64>,
    pub min_temperature: Option<f64>,
    pub max_temperature: Option<f64>,
    pub temperature_step: Option<f64>,
}
