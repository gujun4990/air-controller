use std::{fs, path::PathBuf};

use directories::ProjectDirs;

use crate::models::{AppConfig, LegacyRoot, ServiceResult};

pub struct ConfigStore {
    config_path: PathBuf,
}

impl ConfigStore {
    pub fn new() -> Result<Self, String> {
        let project_dirs = ProjectDirs::from("com", "opencode", "air-controller")
            .ok_or_else(|| "无法定位系统配置目录。".to_string())?;
        let config_dir = project_dirs.config_dir();
        fs::create_dir_all(config_dir).map_err(|error| format!("创建配置目录失败: {error}"))?;

        Ok(Self {
            config_path: config_dir.join("config.json"),
        })
    }

    pub fn load(&self) -> ServiceResult<AppConfig> {
        if !self.config_path.exists() {
            return ServiceResult::ok("已使用默认配置。", AppConfig::default());
        }

        let text = match fs::read_to_string(&self.config_path) {
            Ok(text) => text,
            Err(error) => return ServiceResult::fail(format!("读取配置失败: {error}")),
        };

        let config = match serde_json::from_str::<AppConfig>(&text) {
            Ok(config) => config,
            Err(error) => return ServiceResult::fail(format!("解析配置失败: {error}")),
        };

        match self.validate(&config) {
            Ok(()) => ServiceResult::ok("配置已加载。", config),
            Err(message) => ServiceResult::fail(message),
        }
    }

    pub fn save(&self, config: &AppConfig) -> ServiceResult<AppConfig> {
        if let Err(message) = self.validate(config) {
            return ServiceResult::fail(message);
        }

        let json = match serde_json::to_string_pretty(config) {
            Ok(json) => json,
            Err(error) => return ServiceResult::fail(format!("序列化配置失败: {error}")),
        };

        if let Err(error) = fs::write(&self.config_path, json) {
            return ServiceResult::fail(format!("写入配置失败: {error}"));
        }

        ServiceResult::ok("配置已保存。", config.clone())
    }

    pub fn export(&self, path: &str) -> ServiceResult<bool> {
        let config = self.load();
        let Some(config) = config.data else {
            return ServiceResult::fail(config.message);
        };

        let json = match serde_json::to_string_pretty(&config) {
            Ok(json) => json,
            Err(error) => return ServiceResult::fail(format!("序列化配置失败: {error}")),
        };

        if let Err(error) = fs::write(path, json) {
            return ServiceResult::fail(format!("导出配置失败: {error}"));
        }

        ServiceResult::ok("配置已导出。", true)
    }

    pub fn import_legacy(
        &self,
        path: Option<String>,
    ) -> ServiceResult<(AppConfig, Option<String>)> {
        let legacy_path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/opt/ac/appsettings.json"));

        let text = match fs::read_to_string(&legacy_path) {
            Ok(text) => text,
            Err(error) => return ServiceResult::fail(format!("读取旧配置失败: {error}")),
        };

        let legacy = match serde_json::from_str::<LegacyRoot>(&text) {
            Ok(legacy) => legacy,
            Err(error) => return ServiceResult::fail(format!("解析旧配置失败: {error}")),
        };

        let home_assistant = legacy.home_assistant;
        let startup = legacy.startup;
        let climate = legacy.climate;

        let config = AppConfig {
            base_url: home_assistant
                .as_ref()
                .and_then(|item| item.base_url.clone())
                .unwrap_or_default(),
            climate_entity_id: home_assistant
                .as_ref()
                .and_then(|item| item.climate_entity_id.clone())
                .unwrap_or_else(|| AppConfig::default().climate_entity_id),
            launch_on_system_startup: startup
                .as_ref()
                .and_then(|item| item.launch_on_windows_startup)
                .unwrap_or(false),
            auto_power_on_on_startup: startup
                .as_ref()
                .and_then(|item| item.auto_power_on_on_startup)
                .unwrap_or(false),
            startup_delay_seconds: startup
                .as_ref()
                .and_then(|item| item.startup_delay_seconds)
                .unwrap_or(8),
            retry_count: startup
                .as_ref()
                .and_then(|item| item.retry_count)
                .unwrap_or(3),
            default_temperature: climate
                .as_ref()
                .and_then(|item| item.default_temperature)
                .unwrap_or(26.0),
            min_temperature: climate
                .as_ref()
                .and_then(|item| item.min_temperature)
                .unwrap_or(16.0),
            max_temperature: climate
                .as_ref()
                .and_then(|item| item.max_temperature)
                .unwrap_or(30.0),
            temperature_step: climate
                .as_ref()
                .and_then(|item| item.temperature_step)
                .unwrap_or(1.0),
        };

        if let Err(message) = self.validate(&config) {
            return ServiceResult::fail(message);
        }

        let encrypted_exists = home_assistant
            .as_ref()
            .and_then(|item| item.encrypted_long_lived_token.as_ref())
            .is_some_and(|value| !value.trim().is_empty());
        let token = home_assistant.and_then(|item| item.long_lived_token);
        let message = if encrypted_exists && token.as_deref().unwrap_or_default().trim().is_empty()
        {
            "旧配置已读取，但加密 token 无法跨平台迁移，请重新填写 Token。"
        } else {
            "旧配置已读取。"
        };

        ServiceResult::ok(message, (config, token))
    }

    pub fn validate(&self, config: &AppConfig) -> Result<(), String> {
        if config.base_url.trim().is_empty() {
            return Err("baseUrl 不能为空。".into());
        }

        if reqwest::Url::parse(&config.base_url).is_err() {
            return Err("baseUrl 不是有效地址。".into());
        }

        if !config.climate_entity_id.starts_with("climate.") {
            return Err("climateEntityId 必须是 climate.xxx 格式。".into());
        }

        if config.min_temperature > config.max_temperature {
            return Err("最小温度不能大于最大温度。".into());
        }

        if config.temperature_step <= 0.0 {
            return Err("temperatureStep 必须大于 0。".into());
        }

        if config.default_temperature < config.min_temperature
            || config.default_temperature > config.max_temperature
        {
            return Err("defaultTemperature 必须在温度范围内。".into());
        }

        if config.startup_delay_seconds < 0 {
            return Err("startupDelaySeconds 不能小于 0。".into());
        }

        if config.retry_count < 1 {
            return Err("retryCount 不能小于 1。".into());
        }

        Ok(())
    }
}
