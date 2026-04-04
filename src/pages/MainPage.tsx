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
  const runtimeStep = state?.temperatureStep ?? config.temperatureStep;

  return (
    <section className="control-view">
      <div className="hero-panel panel">
        <div className="hero-head">
          <div>
            <span className="eyebrow">DEVICE CONTROL</span>
            <h2>空调控制</h2>
          </div>
          <button className="ghost" disabled={busy} onClick={() => void onRefresh()}>
            刷新
          </button>
        </div>

        <div className="hero-grid">
          <div className="climate-stage">
            <div className="temperature-ring">
              <div className="ring-orbit orbit-one" />
              <div className="ring-orbit orbit-two" />
              <div className="temperature-core">
                <span>目标温度</span>
                <strong>{formatTemperature(state?.targetTemperature ?? null)}</strong>
                <small>摄氏度</small>
              </div>
            </div>

            <div className="main-control-row unified-control-row">
              <button className="power-button power-on equal-action-button" disabled={busy} onClick={() => void onTurnOn()}>
                开机
              </button>
              <button className="power-button power-off equal-action-button" disabled={busy} onClick={() => void onTurnOff()}>
                关机
              </button>
              <button className="secondary pill-button equal-action-button" disabled={busy} onClick={() => void onChangeTemperature(-runtimeStep)}>
                温度 -
              </button>
              <button className="secondary pill-button equal-action-button" disabled={busy} onClick={() => void onChangeTemperature(runtimeStep)}>
                温度 +
              </button>
            </div>

            <div className="step-hint step-inline">每次调节步长：{runtimeStep.toFixed(1)} °C</div>
          </div>

          <div className="status-stack device-stack">
            <div className="device-summary-card emphasized">
              <span>连接</span>
              <strong>{state ? (state.isAvailable ? "已连接" : "不可用") : "未连接"}</strong>
            </div>

            <div className="device-mini-grid">
              <div className="device-mini-card power-state-card">
                <span>电源</span>
                <strong>{state ? (state.isOn ? "已开机" : "已关机") : "未知"}</strong>
              </div>
              <div className="device-mini-card temp-state-card">
                <span>当前温度</span>
                <strong>{formatTemperature(state?.currentTemperature ?? null)}</strong>
              </div>
              <div className="device-mini-card entity-card compact-entity">
                <span>实体</span>
                <strong>{config.climateEntityId || "-"}</strong>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
