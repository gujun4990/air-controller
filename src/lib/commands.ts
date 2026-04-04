import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, ClimateState, ServiceResult } from "./types";

export function getConfig() {
  return invoke<ServiceResult<AppConfig>>("get_config");
}

export function saveConfig(config: AppConfig) {
  return invoke<ServiceResult<AppConfig>>("save_config", { config });
}

export function hasToken() {
  return invoke<ServiceResult<boolean>>("has_token");
}

export function saveToken(token: string) {
  return invoke<ServiceResult<boolean>>("save_token", { token });
}

export function deleteToken() {
  return invoke<ServiceResult<boolean>>("delete_token");
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

export function importLegacyConfig(path?: string) {
  return invoke<ServiceResult<AppConfig>>("import_legacy_config", { path });
}

export function exportConfig(path: string) {
  return invoke<ServiceResult<boolean>>("export_config", { path });
}

export function getLaunchOnStartup() {
  return invoke<ServiceResult<boolean>>("get_launch_on_startup");
}

export function runAutoPowerOn() {
  return invoke<ServiceResult<ClimateState>>("run_auto_power_on");
}

export function getConfigDirectory() {
  return invoke<ServiceResult<string>>("get_config_directory");
}
