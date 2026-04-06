use tokio::time::{sleep, Duration};

use crate::{
    ha_client::HomeAssistantClient,
    models::{AppConfig, ClimateState, ServiceResult},
};

const STARTUP_DELAY_SECONDS: u64 = 8;
const RETRY_COUNT: i32 = 3;
const POST_ACTION_REFRESH_RETRY_COUNT: i32 = 5;
const POST_ACTION_REFRESH_INTERVAL_SECONDS: u64 = 2;

fn is_operable(state: &ClimateState) -> bool {
    state.is_available
        && state.is_on
        && state.target_temperature.is_some()
        && state.min_temperature.is_some()
        && state.max_temperature.is_some()
}

pub async fn execute(
    config: AppConfig,
    client: &HomeAssistantClient,
) -> ServiceResult<ClimateState> {
    sleep(Duration::from_secs(STARTUP_DELAY_SECONDS)).await;

    let retries = RETRY_COUNT;
    let mut last_message = String::from("启动自动开机失败。");

    for _ in 0..retries {
        let turned_on = client.turn_on().await;
        if !turned_on.success {
            last_message = turned_on.message;
            sleep(Duration::from_secs(2)).await;
            continue;
        }

        let set_result = client.set_temperature(config.default_temperature).await;
        if set_result.success {
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

                sleep(Duration::from_secs(POST_ACTION_REFRESH_INTERVAL_SECONDS)).await;
            }

            last_message = last_refresh_message;
            sleep(Duration::from_secs(2)).await;
            continue;
        }

        last_message = set_result.message;
        sleep(Duration::from_secs(2)).await;
    }

    ServiceResult::fail(last_message)
}
