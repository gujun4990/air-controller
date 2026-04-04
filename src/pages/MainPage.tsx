import type { AppConfig, ClimateState } from "../lib/types";

type Props = {
  config: AppConfig;
  state: ClimateState | null;
  busy: boolean;
  onRefresh: () => Promise<void>;
  onTurnOn: () => Promise<void>;
  onTurnOff: () => Promise<void>;
  onChangeTemperature: (delta: number) => Promise<void>;
};

function formatTemperature(value: number | null) {
  return value === null ? "-" : `${value.toFixed(1)} °C`;
}

export default function MainPage({
  config,
  state,
  busy,
  onRefresh,
  onTurnOn,
  onTurnOff,
  onChangeTemperature
}: Props) {
  return (
    <section className="panel">
      <div className="panel-header compact-header">
        <h2>空调控制</h2>
      </div>

      <div className="overview-row">
        <div className="summary-strip">
          <div className="summary-item">
            <span>连接</span>
            <strong>{state ? (state.isAvailable ? "已连接" : "不可用") : "未连接"}</strong>
          </div>
          <div className="summary-item">
            <span>电源</span>
            <strong>{state ? (state.isOn ? "已开机" : "已关机") : "未知"}</strong>
          </div>
          <div className="summary-item summary-item-wide">
            <span>实体</span>
            <strong>{config.climateEntityId || "-"}</strong>
          </div>
        </div>

        <div className="temperature-panel">
          <div className="temperature-readings">
            <div className="temperature-tile">
              <span>当前</span>
              <strong>{formatTemperature(state?.currentTemperature ?? null)}</strong>
            </div>
            <div className="temperature-tile primary">
              <span>目标</span>
              <strong>{formatTemperature(state?.targetTemperature ?? null)}</strong>
            </div>
          </div>

          <div className="actions-row">
            <button className="secondary" disabled={busy} onClick={() => void onChangeTemperature(-config.temperatureStep)}>
              温度 -
            </button>
            <button className="secondary" disabled={busy} onClick={() => void onChangeTemperature(config.temperatureStep)}>
              温度 +
            </button>
          </div>
        </div>
      </div>

      <div className="actions-row">
        <button disabled={busy} onClick={() => void onTurnOn()}>
          开机
        </button>
        <button className="secondary" disabled={busy} onClick={() => void onTurnOff()}>
          关机
        </button>
        <button className="ghost" disabled={busy} onClick={() => void onRefresh()}>
          刷新
        </button>
      </div>
    </section>
  );
}
