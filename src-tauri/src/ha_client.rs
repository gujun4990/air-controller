use reqwest::{header, Client};
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};

use crate::models::{AppConfig, ClimateState, ServiceResult};

#[derive(Clone, Copy, PartialEq, Eq)]
enum TemperatureUnit {
    Celsius,
    Fahrenheit,
    Unknown,
}

struct ClimateSnapshot {
    state: ClimateState,
    temperature_unit: TemperatureUnit,
    entity_min_temperature: Option<f64>,
    entity_max_temperature: Option<f64>,
    entity_temperature_step: Option<f64>,
}

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
        match self.get_state_snapshot().await {
            Ok(snapshot) => ServiceResult::ok("状态已刷新。", snapshot.state),
            Err(message) => ServiceResult::fail(message),
        }
    }

    async fn get_state_snapshot(&self) -> Result<ClimateSnapshot, String> {
        let url = format!(
            "{}/api/states/{}",
            self.config.base_url.trim_end_matches('/'),
            self.config.climate_entity_id
        );
        let response = match self.client.get(url).send().await {
            Ok(response) => response,
            Err(error) => return Err(format!("连接 Home Assistant 失败: {error}")),
        };

        let status = response.status();
        let body = match response.text().await {
            Ok(body) => body,
            Err(error) => return Err(format!("读取响应失败: {error}")),
        };

        if !status.is_success() {
            return Err(describe_http_failure(
                "获取空调状态失败",
                status.as_u16(),
                &body,
            ));
        }

        let value = match serde_json::from_str::<Value>(&body) {
            Ok(value) => value,
            Err(error) => return Err(format!("解析状态失败: {error}")),
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
        let entity_target_temperature = parse_entity_target_temperature(&attributes);
        let entity_min_temperature = parse_double(attributes.get("min_temp"));
        let entity_max_temperature = parse_double(attributes.get("max_temp"));
        let entity_temperature_step = parse_double(attributes.get("target_temp_step"));
        let temperature_unit = parse_temperature_unit(
            attributes.get("temperature_unit"),
            entity_target_temperature,
            entity_min_temperature,
            entity_max_temperature,
            parse_double(attributes.get("current_temperature")),
        );
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
            current_temperature: convert_entity_temperature(
                parse_double(attributes.get("current_temperature")),
                temperature_unit,
            ),
            target_temperature: convert_entity_temperature(
                entity_target_temperature,
                temperature_unit,
            ),
            min_temperature: convert_entity_temperature(entity_min_temperature, temperature_unit),
            max_temperature: convert_entity_temperature(entity_max_temperature, temperature_unit),
            temperature_step: convert_entity_temperature_step(
                entity_temperature_step,
                temperature_unit,
            ),
            is_available: !state_text.eq_ignore_ascii_case("unavailable"),
            is_on: !matches!(state_text.as_str(), "off" | "unavailable"),
        };

        Ok(ClimateSnapshot {
            state: climate_state,
            temperature_unit,
            entity_min_temperature,
            entity_max_temperature,
            entity_temperature_step,
        })
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
        let snapshot = match self.get_state_snapshot().await {
            Ok(snapshot) => snapshot,
            Err(message) => return ServiceResult::fail(message),
        };
        let entity_temperature = normalize_entity_temperature(temperature, &snapshot);
        let mut payload = json!({
            "entity_id": self.config.climate_entity_id,
            "temperature": entity_temperature
        });

        if !snapshot.state.hvac_mode.is_empty() && snapshot.state.hvac_mode != "off" {
            payload["hvac_mode"] = Value::String(snapshot.state.hvac_mode);
        }

        let service_result = self.post_service_raw("set_temperature", payload).await;
        if !service_result.success {
            return ServiceResult::fail(service_result.message);
        }

        let expected_celsius =
            convert_entity_temperature(Some(entity_temperature), snapshot.temperature_unit)
                .unwrap_or(temperature);
        self.wait_for_target_temperature(expected_celsius).await
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

fn convert_entity_temperature(value: Option<f64>, unit: TemperatureUnit) -> Option<f64> {
    let raw = value?;
    Some(match unit {
        TemperatureUnit::Fahrenheit => fahrenheit_to_celsius(raw),
        TemperatureUnit::Celsius | TemperatureUnit::Unknown => raw,
    })
}

fn convert_entity_temperature_step(value: Option<f64>, unit: TemperatureUnit) -> Option<f64> {
    let raw = value?;
    Some(match unit {
        TemperatureUnit::Fahrenheit => round_one_decimal(raw * 5.0 / 9.0),
        TemperatureUnit::Celsius | TemperatureUnit::Unknown => raw,
    })
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

fn parse_entity_target_temperature(attributes: &Value) -> Option<f64> {
    parse_double(attributes.get("temperature"))
        .or_else(|| parse_double(attributes.get("target_temp_high")))
        .or_else(|| parse_double(attributes.get("target_temp_low")))
}

fn parse_temperature_unit(
    value: Option<&Value>,
    target_temperature: Option<f64>,
    min_temperature: Option<f64>,
    max_temperature: Option<f64>,
    current_temperature: Option<f64>,
) -> TemperatureUnit {
    match value
        .and_then(Value::as_str)
        .map(|item| item.trim().to_ascii_uppercase())
    {
        Some(unit) if unit == "F" || unit == "°F" => TemperatureUnit::Fahrenheit,
        Some(unit) if unit == "C" || unit == "°C" => TemperatureUnit::Celsius,
        _ => infer_temperature_unit(
            target_temperature,
            min_temperature,
            max_temperature,
            current_temperature,
        ),
    }
}

fn infer_temperature_unit(
    target_temperature: Option<f64>,
    min_temperature: Option<f64>,
    max_temperature: Option<f64>,
    current_temperature: Option<f64>,
) -> TemperatureUnit {
    let candidates = [
        target_temperature,
        min_temperature,
        max_temperature,
        current_temperature,
    ];

    if candidates.into_iter().flatten().any(|value| value > 45.0) {
        return TemperatureUnit::Fahrenheit;
    }

    if min_temperature
        .zip(max_temperature)
        .is_some_and(|(min, max)| min >= 8.0 && max <= 40.0)
    {
        return TemperatureUnit::Celsius;
    }

    TemperatureUnit::Unknown
}

fn convert_celsius_to_entity_unit(value: f64, unit: TemperatureUnit) -> f64 {
    match unit {
        TemperatureUnit::Fahrenheit => celsius_to_fahrenheit(value),
        TemperatureUnit::Celsius | TemperatureUnit::Unknown => value,
    }
}

fn normalize_entity_temperature(value_celsius: f64, snapshot: &ClimateSnapshot) -> f64 {
    let mut value = convert_celsius_to_entity_unit(value_celsius, snapshot.temperature_unit);

    if let Some(minimum) = snapshot.entity_min_temperature {
        value = value.max(minimum);
    }

    if let Some(maximum) = snapshot.entity_max_temperature {
        value = value.min(maximum);
    }

    if let Some(step) = snapshot.entity_temperature_step.filter(|step| *step > 0.0) {
        let anchor = snapshot.entity_min_temperature.unwrap_or(0.0);
        value = anchor + ((value - anchor) / step).round() * step;
    }

    round_one_decimal(value)
}

fn fahrenheit_to_celsius(value: f64) -> f64 {
    round_one_decimal((value - 32.0) * 5.0 / 9.0)
}

fn celsius_to_fahrenheit(value: f64) -> f64 {
    round_one_decimal((value * 9.0 / 5.0) + 32.0)
}

fn round_one_decimal(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn describe_http_failure(prefix: &str, status: u16, body: &str) -> String {
    match status {
        401 => format!("{prefix}: Token 无效或已过期。"),
        403 => format!("{prefix}: 当前 Token 没有访问权限。"),
        404 => format!("{prefix}: 地址或实体不存在，请检查配置。"),
        _ => format!("{prefix}: HTTP {status} {body}"),
    }
}
