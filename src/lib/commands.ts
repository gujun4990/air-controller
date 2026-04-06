import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, ClimateState, ServiceResult, StartupAutoPowerOnStatus } from "./types";

export function getConfig() {
  return invoke<ServiceResult<AppConfig>>("get_config");
}

export function saveSettings(config: AppConfig, token: string) {
  return invoke<ServiceResult<AppConfig>>("save_settings", { config, token });
}

export function hasToken() {
  return invoke<ServiceResult<boolean>>("has_token");
}

export function refreshState() {
  return invoke<ServiceResult<ClimateState>>("get_state");
}

export function turnOn() {
  return invoke<ServiceResult<ClimateState>>("turn_on");
}

export function turnOff() {
  return invoke<ServiceResult<ClimateState>>("turn_off");
}

export function setTemperature(temperature: number) {
  return invoke<ServiceResult<ClimateState>>("set_temperature", { temperature });
}

export function getStartupAutoPowerOnStatus() {
  return invoke<StartupAutoPowerOnStatus>("get_startup_auto_power_on_status");
}

export function clearStartupAutoPowerOnStatus() {
  return invoke<void>("clear_startup_auto_power_on_status");
}

export function minimizeWindow() {
  return invoke<ServiceResult<boolean>>("minimize_window");
}

export function hideWindow() {
  return invoke<ServiceResult<boolean>>("hide_window");
}
