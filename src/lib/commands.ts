import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, ClimateState, ServiceResult } from "./types";

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

export function isSystemStartupLaunch() {
  return invoke<boolean>("is_system_startup_launch");
}

export function takeStartupAutoPowerOnResult() {
  return invoke<ServiceResult<ClimateState> | null>("take_startup_auto_power_on_result");
}

export function minimizeWindow() {
  return invoke<ServiceResult<boolean>>("minimize_window");
}

export function hideWindow() {
  return invoke<ServiceResult<boolean>>("hide_window");
}
