use std::{fs, path::PathBuf};

use directories::ProjectDirs;

use crate::models::{AppConfig, ServiceResult};

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

    pub fn validate(&self, config: &AppConfig) -> Result<(), String> {
        if config.base_url.trim().is_empty() {
            return Err("baseUrl 不能为空。".into());
        }

        match reqwest::Url::parse(&config.base_url) {
            Ok(url) if url.scheme() == "http" || url.scheme() == "https" => {}
            Ok(_) => return Err("baseUrl 仅支持 http 或 https 协议。".into()),
            Err(_) => return Err("baseUrl 不是有效地址。".into()),
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

        Ok(())
    }
}
