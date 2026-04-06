export type AppConfig = {
  baseUrl: string;
  climateEntityId: string;
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

export type StartupAutoPowerOnStatus = {
  pending: boolean;
  result: ServiceResult<ClimateState> | null;
};

export const defaultConfig: AppConfig = {
  baseUrl: "",
  climateEntityId: "climate.living_room_ac",
  defaultTemperature: 26,
  minTemperature: 16,
  maxTemperature: 30,
  temperatureStep: 1
};
