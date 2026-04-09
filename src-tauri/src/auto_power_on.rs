use std::{future::Future, pin::Pin};

use tokio::time::{sleep, Duration};

use crate::{
    ha_client::HomeAssistantClient,
    models::{ClimateState, ServiceResult},
};

const STARTUP_DELAY_SECONDS: u64 = 8;
const RETRY_COUNT: i32 = 3;
const POST_ACTION_REFRESH_RETRY_COUNT: i32 = 5;
const POST_ACTION_REFRESH_INTERVAL_SECONDS: u64 = 2;

fn is_operable(state: &ClimateState) -> bool {
    state.is_available && state.is_on
}

trait AutoPowerOnClient {
    fn turn_on(&self) -> Pin<Box<dyn Future<Output = ServiceResult<ClimateState>> + Send + '_>>;
    fn get_state(&self) -> Pin<Box<dyn Future<Output = ServiceResult<ClimateState>> + Send + '_>>;
}

impl AutoPowerOnClient for HomeAssistantClient {
    fn turn_on(&self) -> Pin<Box<dyn Future<Output = ServiceResult<ClimateState>> + Send + '_>> {
        Box::pin(async move { HomeAssistantClient::turn_on(self).await })
    }

    fn get_state(&self) -> Pin<Box<dyn Future<Output = ServiceResult<ClimateState>> + Send + '_>> {
        Box::pin(async move { HomeAssistantClient::get_state(self).await })
    }
}

pub async fn execute(client: &HomeAssistantClient) -> ServiceResult<ClimateState> {
    execute_with_delays(
        client,
        Duration::from_secs(STARTUP_DELAY_SECONDS),
        Duration::from_secs(2),
        Duration::from_secs(POST_ACTION_REFRESH_INTERVAL_SECONDS),
    )
    .await
}

async fn execute_with_delays<C: AutoPowerOnClient>(
    client: &C,
    startup_delay: Duration,
    retry_delay: Duration,
    refresh_interval: Duration,
) -> ServiceResult<ClimateState> {
    sleep(startup_delay).await;

    let retries = RETRY_COUNT;
    let mut last_message = String::from("启动自动开机失败。");

    for _ in 0..retries {
        let turned_on = client.turn_on().await;
        if !turned_on.success {
            last_message = turned_on.message;
            sleep(retry_delay).await;
            continue;
        }

        let mut last_refresh_message = String::from("空调未在预期时间内进入可操作状态。");

        for _ in 0..POST_ACTION_REFRESH_RETRY_COUNT {
            let refresh_result = client.get_state().await;
            if refresh_result.success {
                if let Some(state) = refresh_result.data {
                    if is_operable(&state) {
                        return ServiceResult::ok("启动自动开机成功。", state);
                    }

                    last_refresh_message = String::from("空调尚未进入可操作状态，继续等待刷新。");
                } else {
                    last_refresh_message = String::from("刷新状态成功，但未获取到空调状态数据。");
                }
            } else {
                last_refresh_message = refresh_result.message;
            }

            sleep(refresh_interval).await;
        }

        last_message = last_refresh_message;
        sleep(retry_delay).await;
    }

    ServiceResult::fail(last_message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::VecDeque, sync::Mutex};

    struct MockClient {
        turn_on_results: Mutex<VecDeque<ServiceResult<ClimateState>>>,
        get_state_results: Mutex<VecDeque<ServiceResult<ClimateState>>>,
    }

    impl MockClient {
        fn new(
            turn_on_results: Vec<ServiceResult<ClimateState>>,
            get_state_results: Vec<ServiceResult<ClimateState>>,
        ) -> Self {
            Self {
                turn_on_results: Mutex::new(turn_on_results.into()),
                get_state_results: Mutex::new(get_state_results.into()),
            }
        }
    }

    impl AutoPowerOnClient for MockClient {
        fn turn_on(
            &self,
        ) -> Pin<Box<dyn Future<Output = ServiceResult<ClimateState>> + Send + '_>> {
            Box::pin(async move {
                self.turn_on_results
                    .lock()
                    .expect("turn_on_results lock poisoned")
                    .pop_front()
                    .expect("missing mocked turn_on result")
            })
        }

        fn get_state(
            &self,
        ) -> Pin<Box<dyn Future<Output = ServiceResult<ClimateState>> + Send + '_>> {
            Box::pin(async move {
                self.get_state_results
                    .lock()
                    .expect("get_state_results lock poisoned")
                    .pop_front()
                    .expect("missing mocked get_state result")
            })
        }
    }

    fn climate_state(is_available: bool, is_on: bool) -> ClimateState {
        ClimateState {
            entity_id: "climate.test".into(),
            state: if is_on { "cool".into() } else { "off".into() },
            hvac_mode: if is_on { "cool".into() } else { "off".into() },
            hvac_action: String::new(),
            current_temperature: None,
            target_temperature: None,
            min_temperature: None,
            max_temperature: None,
            temperature_step: None,
            is_available,
            is_on,
        }
    }

    #[tokio::test]
    async fn succeeds_after_turn_on_without_temperature_fields() {
        let client = MockClient::new(
            vec![ServiceResult::ok("turned on", climate_state(true, true))],
            vec![ServiceResult::ok("state refreshed", climate_state(true, true))],
        );

        let result = execute_with_delays(
            &client,
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
        )
        .await;

        assert!(result.success);
        assert_eq!(result.message, "启动自动开机成功。");
        assert!(result.data.expect("missing climate state").is_on);
    }

    #[tokio::test]
    async fn fails_after_three_turn_on_attempts() {
        let client = MockClient::new(
            vec![
                ServiceResult::fail("调用失败 1"),
                ServiceResult::fail("调用失败 2"),
                ServiceResult::fail("调用失败 3"),
            ],
            vec![],
        );

        let result = execute_with_delays(
            &client,
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
        )
        .await;

        assert!(!result.success);
        assert_eq!(result.message, "调用失败 3");
        assert!(result.data.is_none());
    }

    #[tokio::test]
    async fn fails_when_device_never_becomes_operable() {
        let client = MockClient::new(
            vec![
                ServiceResult::ok("turned on", climate_state(true, false)),
                ServiceResult::ok("turned on", climate_state(true, false)),
                ServiceResult::ok("turned on", climate_state(true, false)),
            ],
            vec![ServiceResult::ok("state refreshed", climate_state(true, false)); 15],
        );

        let result = execute_with_delays(
            &client,
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
        )
        .await;

        assert!(!result.success);
        assert_eq!(result.message, "空调尚未进入可操作状态，继续等待刷新。");
    }

    #[tokio::test]
    async fn fails_with_last_refresh_error_when_state_query_keeps_failing() {
        let client = MockClient::new(
            vec![
                ServiceResult::ok("turned on", climate_state(true, true)),
                ServiceResult::ok("turned on", climate_state(true, true)),
                ServiceResult::ok("turned on", climate_state(true, true)),
            ],
            vec![ServiceResult::fail("刷新失败"); 15],
        );

        let result = execute_with_delays(
            &client,
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
        )
        .await;

        assert!(!result.success);
        assert_eq!(result.message, "刷新失败");
    }
}
