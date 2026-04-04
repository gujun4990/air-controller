use reqwest::{header, Client};
use serde_json::{json, Value};

use crate::models::{AppConfig, ClimateState, ServiceResult};

pub struct HomeAssistantClient {
    config: AppConfig,
    token: String,
    client: Client,
}

impl HomeAssistantClient {
    pub fn new(config: AppConfig, token: String) -> Result<Self, String> {
        let mut headers = header::HeaderMap::new();
        let bearer = format!("Bearer {token}");
        let auth_value = header::HeaderValue::from_str(&bearer)
            .map_err(|error| format!("构造认证头失败: {error}"))?;
        headers.insert(header::AUTHORIZATION, auth_value);

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|error| format!("创建 HTTP 客户端失败: {error}"))?;

        Ok(Self {
            config,
            token,
            client,
        })
    }

    pub async fn get_state(&self) -> ServiceResult<ClimateState> {
        let url = format!(
            "{}/api/states/{}",
            self.config.base_url.trim_end_matches('/'),
            self.config.climate_entity_id
        );
        let response = match self.client.get(url).send().await {
            Ok(response) => response,
            Err(error) => return ServiceResult::fail(format!("连接 Home Assistant 失败: {error}")),
        };

        let status = response.status();
        let body = match response.text().await {
            Ok(body) => body,
            Err(error) => return ServiceResult::fail(format!("读取响应失败: {error}")),
        };

        if !status.is_success() {
            return ServiceResult::fail(format!("获取空调状态失败: {} {}", status.as_u16(), body));
        }

        let value = match serde_json::from_str::<Value>(&body) {
            Ok(value) => value,
            Err(error) => return ServiceResult::fail(format!("解析状态失败: {error}")),
        };

        let attributes = value
            .get("attributes")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let state_text = value
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let climate_state = ClimateState {
            entity_id: value
                .get("entity_id")
                .and_then(Value::as_str)
                .unwrap_or(&self.config.climate_entity_id)
                .to_string(),
            state: state_text.clone(),
            hvac_mode: attributes
                .get("hvac_mode")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            hvac_action: attributes
                .get("hvac_action")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            current_temperature: parse_double(attributes.get("current_temperature")),
            target_temperature: parse_double(attributes.get("temperature")),
            is_available: !state_text.eq_ignore_ascii_case("unavailable"),
            is_on: !matches!(state_text.as_str(), "off" | "unavailable"),
        };

        ServiceResult::ok("状态已刷新。", climate_state)
    }

    pub async fn turn_on(&self) -> ServiceResult<ClimateState> {
        self.post_service(
            "turn_on",
            json!({ "entity_id": self.config.climate_entity_id }),
        )
        .await
    }

    pub async fn turn_off(&self) -> ServiceResult<ClimateState> {
        self.post_service(
            "turn_off",
            json!({ "entity_id": self.config.climate_entity_id }),
        )
        .await
    }

    pub async fn set_temperature(&self, temperature: f64) -> ServiceResult<ClimateState> {
        self.post_service(
            "set_temperature",
            json!({ "entity_id": self.config.climate_entity_id, "temperature": temperature }),
        )
        .await
    }

    async fn post_service(&self, action: &str, payload: Value) -> ServiceResult<ClimateState> {
        let url = format!(
            "{}/api/services/climate/{}",
            self.config.base_url.trim_end_matches('/'),
            action
        );
        let response = match self.client.post(url).json(&payload).send().await {
            Ok(response) => response,
            Err(error) => return ServiceResult::fail(format!("调用 Home Assistant 失败: {error}")),
        };

        let status = response.status();
        let body = match response.text().await {
            Ok(body) => body,
            Err(error) => return ServiceResult::fail(format!("读取响应失败: {error}")),
        };

        if !status.is_success() {
            return ServiceResult::fail(format!("调用失败: {} {}", status.as_u16(), body));
        }

        self.get_state().await
    }

    pub fn token_len(&self) -> usize {
        self.token.len()
    }
}

fn parse_double(value: Option<&Value>) -> Option<f64> {
    let value = value?;
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|text| text.parse::<f64>().ok()))
}
