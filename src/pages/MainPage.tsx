import type { AppConfig, ClimateState } from "../lib/types";

type Props = {
  config: AppConfig;
  state: ClimateState | null;
  statusMessage: string;
  busy: boolean;
  onRefresh: () => Promise<void>;
  onTurnOn: () => Promise<void>;
  onTurnOff: () => Promise<void>;
  onChangeTemperature: (delta: number) => Promise<void>;
  onRunAutoPowerOn: () => Promise<void>;
};

function formatTemperature(value: number | null) {
  return value === null ? "-" : `${value.toFixed(1)} °C`;
}

export default function MainPage({
  config,
  state,
  statusMessage,
  busy,
  onRefresh,
  onTurnOn,
  onTurnOff,
  onChangeTemperature,
  onRunAutoPowerOn
}: Props) {
  return (
    <section className="panel">
      <div className="panel-header">
        <div>
          <h2>空调控制</h2>
          <p>通过 Home Assistant 直接控制目标空调实体。</p>
        </div>
      </div>

      <div className="stats-grid">
        <div className="stat-card">
          <span>连接状态</span>
          <strong>{state ? (state.isAvailable ? "已连接" : "设备不可用") : "未连接"}</strong>
        </div>
        <div className="stat-card">
          <span>实体 ID</span>
          <strong>{config.climateEntityId || "-"}</strong>
        </div>
        <div className="stat-card">
          <span>电源状态</span>
          <strong>{state ? (state.isOn ? "已开机" : "已关机") : "未知"}</strong>
        </div>
        <div className="stat-card">
          <span>当前温度</span>
          <strong>{formatTemperature(state?.currentTemperature ?? null)}</strong>
        </div>
        <div className="stat-card">
          <span>目标温度</span>
          <strong>{formatTemperature(state?.targetTemperature ?? null)}</strong>
        </div>
        <div className="stat-card message-card">
          <span>消息</span>
          <strong>{statusMessage || "等待操作"}</strong>
        </div>
      </div>

      <div className="actions-row">
        <button disabled={busy} onClick={() => void onTurnOn()}>
          开机
        </button>
        <button className="secondary" disabled={busy} onClick={() => void onTurnOff()}>
          关机
        </button>
        <button className="secondary" disabled={busy} onClick={() => void onChangeTemperature(-config.temperatureStep)}>
          温度 -
        </button>
        <button className="secondary" disabled={busy} onClick={() => void onChangeTemperature(config.temperatureStep)}>
          温度 +
        </button>
        <button className="ghost" disabled={busy} onClick={() => void onRefresh()}>
          刷新
        </button>
        <button className="ghost" disabled={busy} onClick={() => void onRunAutoPowerOn()}>
          执行自动开机
        </button>
      </div>
    </section>
  );
}
