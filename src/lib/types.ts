export type AppConfig = {
  baseUrl: string;
  climateEntityId: string;
  launchOnSystemStartup: boolean;
  autoPowerOnOnStartup: boolean;
  startupDelaySeconds: number;
  retryCount: number;
  defaultTemperature: number;
  minTemperature: number;
  maxTemperature: number;
  temperatureStep: number;
};

export type ClimateState = {
  entityId: string;
  state: string;
  hvacMode: string;
  hvacAction: string;
  currentTemperature: number | null;
  targetTemperature: number | null;
  minTemperature: number | null;
  maxTemperature: number | null;
  temperatureStep: number | null;
  isAvailable: boolean;
  isOn: boolean;
};

export type ServiceResult<T> = {
  success: boolean;
  message: string;
  data: T | null;
};

export type SavedSettings = {
  config: AppConfig;
  hasToken: boolean;
};

export const defaultConfig: AppConfig = {
  baseUrl: "",
  climateEntityId: "climate.living_room_ac",
  launchOnSystemStartup: false,
  autoPowerOnOnStartup: false,
  startupDelaySeconds: 8,
  retryCount: 3,
  defaultTemperature: 26,
  minTemperature: 16,
  maxTemperature: 30,
  temperatureStep: 1
};
