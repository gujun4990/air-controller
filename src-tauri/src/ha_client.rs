use reqwest::{header, Client};
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};

use crate::models::{AppConfig, ClimateState, ServiceResult};

pub struct HomeAssistantClient {
    config: AppConfig,
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

        Ok(Self { config, client })
    }

    pub async fn test_connection(&self) -> ServiceResult<bool> {
        let url = format!("{}/api/", self.config.base_url.trim_end_matches('/'));
        let response = match self.client.get(url).send().await {
            Ok(response) => response,
            Err(error) => {
                return ServiceResult::fail(format!("连接 Home Assistant 失败: {error}"));
            }
        };

        let status = response.status();
        let body = match response.text().await {
            Ok(body) => body,
            Err(error) => return ServiceResult::fail(format!("读取响应失败: {error}")),
        };

        if !status.is_success() {
            return ServiceResult::fail(describe_http_failure(
                "连接测试失败",
                status.as_u16(),
                &body,
            ));
        }

        ServiceResult::ok("Home Assistant 连接正常。", true)
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
            return ServiceResult::fail(describe_http_failure(
                "获取空调状态失败",
                status.as_u16(),
                &body,
            ));
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
        let current_state = self.get_state().await.data;
        let mut payload = json!({
            "entity_id": self.config.climate_entity_id,
            "temperature": temperature
        });

        if let Some(state) = current_state {
            if !state.hvac_mode.is_empty() && state.hvac_mode != "off" {
                payload["hvac_mode"] = Value::String(state.hvac_mode);
            }
        }

        let service_result = self.post_service_raw("set_temperature", payload).await;
        if !service_result.success {
            return ServiceResult::fail(service_result.message);
        }

        self.wait_for_target_temperature(temperature).await
    }

    async fn post_service(&self, action: &str, payload: Value) -> ServiceResult<ClimateState> {
        let result = self.post_service_raw(action, payload).await;
        if !result.success {
            return ServiceResult::fail(result.message);
        }

        self.get_state().await
    }

    async fn post_service_raw(&self, action: &str, payload: Value) -> ServiceResult<bool> {
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
            return ServiceResult::fail(describe_http_failure("调用失败", status.as_u16(), &body));
        }

        ServiceResult::ok("调用成功。", true)
    }

    async fn wait_for_target_temperature(
        &self,
        expected_temperature: f64,
    ) -> ServiceResult<ClimateState> {
        let mut latest_state: Option<ClimateState> = None;

        for _ in 0..5 {
            sleep(Duration::from_millis(350)).await;

            let state_result = self.get_state().await;
            if let Some(state) = state_result.data.clone() {
                let matched = state
                    .target_temperature
                    .is_some_and(|value| (value - expected_temperature).abs() < 0.11);
                latest_state = Some(state.clone());

                if matched {
                    return ServiceResult::ok(
                        format!("目标温度已设置为 {expected_temperature:.1} 摄氏度。"),
                        state,
                    );
                }
            }
        }

        if let Some(state) = latest_state {
            return ServiceResult::ok(
                format!(
                    "已发送温度设置请求，当前目标温度为 {} 摄氏度。",
                    state.target_temperature.unwrap_or(expected_temperature)
                ),
                state,
            );
        }

        ServiceResult::fail("温度设置请求已发送，但未能确认最新状态。")
    }
}

fn parse_double(value: Option<&Value>) -> Option<f64> {
    let value = value?;
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|text| text.parse::<f64>().ok()))
        .or_else(|| {
            value
                .get("target_temp_high")
                .and_then(|item| parse_double(Some(item)))
                .or_else(|| {
                    value
                        .get("target_temp_low")
                        .and_then(|item| parse_double(Some(item)))
                })
        })
}

fn describe_http_failure(prefix: &str, status: u16, body: &str) -> String {
    match status {
        401 => format!("{prefix}: Token 无效或已过期。"),
        403 => format!("{prefix}: 当前 Token 没有访问权限。"),
        404 => format!("{prefix}: 地址或实体不存在，请检查配置。"),
        _ => format!("{prefix}: HTTP {status} {body}"),
    }
}
