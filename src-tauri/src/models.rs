use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub base_url: String,
    pub climate_entity_id: String,
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
    pub min_temperature: Option<f64>,
    pub max_temperature: Option<f64>,
    pub temperature_step: Option<f64>,
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

pub struct StartupAutoPowerOnStore(pub Mutex<Option<ServiceResult<ClimateState>>>);

impl Default for StartupAutoPowerOnStore {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}
