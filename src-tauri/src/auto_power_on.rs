use tokio::time::{sleep, Duration};

use crate::{
    ha_client::HomeAssistantClient,
    models::{AppConfig, ClimateState, ServiceResult},
};

pub async fn execute(
    config: AppConfig,
    client: &HomeAssistantClient,
) -> ServiceResult<ClimateState> {
    if !config.auto_power_on_on_startup {
        return client.get_state().await;
    }

    if config.startup_delay_seconds > 0 {
        sleep(Duration::from_secs(config.startup_delay_seconds as u64)).await;
    }

    let retries = config.retry_count.max(1);
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
            return ServiceResult::ok("启动自动开机成功。", set_result.data.unwrap_or_default());
        }

        last_message = set_result.message;
        sleep(Duration::from_secs(2)).await;
    }

    ServiceResult::fail(last_message)
}
