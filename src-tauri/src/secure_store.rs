use keyring::Entry;

use crate::models::ServiceResult;

pub struct SecureStore {
    service_name: &'static str,
    account_name: &'static str,
}

impl SecureStore {
    pub fn new() -> Self {
        Self {
            service_name: "air-controller",
            account_name: "home-assistant-token",
        }
    }

    fn entry(&self) -> Result<Entry, String> {
        Entry::new(self.service_name, self.account_name)
            .map_err(|error| format!("初始化密钥链失败: {error}"))
    }

    pub fn has_token(&self) -> ServiceResult<bool> {
        match self.load_token_value() {
            Ok(token) => ServiceResult::ok("Token 状态已读取。", !token.trim().is_empty()),
            Err(message) => ServiceResult::fail(message),
        }
    }

    pub fn save_token(&self, token: &str) -> ServiceResult<bool> {
        let entry = match self.entry() {
            Ok(entry) => entry,
            Err(message) => return ServiceResult::fail(message),
        };

        if let Err(error) = entry.set_password(token) {
            return ServiceResult::fail(format!("保存 Token 失败: {error}"));
        }

        ServiceResult::ok("Token 已保存到系统密钥链。", true)
    }

    pub fn load_token_value(&self) -> Result<String, String> {
        let entry = self.entry()?;
        entry
            .get_password()
            .map_err(|error| format!("读取 Token 失败: {error}"))
    }

    pub fn delete_token(&self) -> ServiceResult<bool> {
        let entry = match self.entry() {
            Ok(entry) => entry,
            Err(message) => return ServiceResult::fail(message),
        };

        match entry.delete_credential() {
            Ok(()) => ServiceResult::ok("Token 已从系统密钥链删除。", true),
            Err(error) => ServiceResult::fail(format!("删除 Token 失败: {error}")),
        }
    }
}
