import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { type MouseEvent, useEffect, useState } from "react";
import ConfigPage from "../pages/ConfigPage";
import MainPage from "../pages/MainPage";
import {
  getConfig,
  getLaunchOnStartup,
  hasToken as checkToken,
  hideWindow,
  refreshState,
  runAutoPowerOn,
  saveConfig,
  saveToken,
  setTemperature,
  testConnection,
  turnOff,
  turnOn
} from "../lib/commands";
import { defaultConfig, type AppConfig, type ClimateState } from "../lib/types";

type TabKey = "main" | "config";
type StatusTone = "info" | "success" | "error";
type StatusState = { tone: StatusTone; text: string };

const STEP_CELSIUS = 1;

export default function App() {
  const [activeTab, setActiveTab] = useState<TabKey>("main");
  const [config, setConfig] = useState<AppConfig>(defaultConfig);
  const [climateState, setClimateState] = useState<ClimateState | null>(null);
  const [busy, setBusy] = useState(false);
  const [hasToken, setHasToken] = useState(false);
  const [status, setStatus] = useState<StatusState>({
    tone: "info",
    text: "正在加载配置..."
  });

  useEffect(() => {
    void initialize();
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    void listen<string>("navigate", (event) => {
      if (event.payload === "config") {
        setActiveTab("config");
      }

      if (event.payload === "main") {
        setActiveTab("main");
      }
    }).then((dispose) => {
      unlisten = dispose;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  async function initialize() {
    setBusy(true);
    try {
      const [configResult, tokenResult, startupResult] = await Promise.all([
        getConfig(),
        checkToken(),
        getLaunchOnStartup()
      ]);

      if (!configResult.success || !configResult.data) {
        setStatus({ tone: "error", text: normalizeStatusText(configResult.message) });
        return;
      }

      const nextConfig: AppConfig = {
        ...configResult.data,
        launchOnSystemStartup:
          startupResult.data ?? configResult.data.launchOnSystemStartup
      };

      setConfig(nextConfig);
      setHasToken(Boolean(tokenResult.data));

      if (!tokenResult.data) {
        setStatus({
          tone: "info",
          text: "请先在配置页保存访问令牌。"
        });
        return;
      }

      if (nextConfig.autoPowerOnOnStartup) {
        await runClimateAction(runAutoPowerOn);
      } else {
        await handleRefresh();
      }
    } catch (error) {
      setStatus({ tone: "error", text: normalizeStatusText(`初始化失败: ${String(error)}`) });
    } finally {
      setBusy(false);
    }
  }

  async function runClimateAction(
    action: () => Promise<{
      success: boolean;
      message: string;
      data: ClimateState | null;
    }>
  ) {
    setBusy(true);
    try {
      const result = await action();
      if (result.data) {
        setClimateState(result.data);
      }
      setStatus({
        tone: result.success ? "success" : "error",
        text: normalizeStatusText(result.message)
      });
    } catch (error) {
      setStatus({ tone: "error", text: normalizeStatusText(`请求失败: ${String(error)}`) });
    } finally {
      setBusy(false);
    }
  }

  async function handleRefresh() {
    await runClimateAction(refreshState);
  }

  async function handleTurnOn() {
    await runClimateAction(turnOn);
  }

  async function handleTurnOff() {
    await runClimateAction(turnOff);
  }

  async function handleChangeTemperature(delta: number) {
    const base = climateState?.targetTemperature ?? config.defaultTemperature;
    const next = clampTemperature(base + delta, config, climateState);
    await runClimateAction(() => setTemperature(next));
  }

  async function handleSaveConfig(nextConfig: AppConfig): Promise<boolean> {
    setBusy(true);
    try {
      const result = await saveConfig(nextConfig);
      setStatus({
        tone: result.success ? "success" : "error",
        text: normalizeStatusText(result.message)
      });

      if (!result.success || !result.data) {
        return false;
      }

      setConfig(result.data);

      if (hasToken) {
        const testResult = await testConnection();
        setStatus({
          tone: testResult.success ? "success" : "error",
          text: normalizeStatusText(testResult.message)
        });
      }

      return true;
    } catch (error) {
      setStatus({ tone: "error", text: normalizeStatusText(`保存配置失败: ${String(error)}`) });
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function handleSaveToken(token: string): Promise<boolean> {
    setBusy(true);
    try {
      const result = await saveToken(token);
      setHasToken(Boolean(result.data));
      if (!result.success) {
        setStatus({ tone: "error", text: normalizeStatusText(result.message) });
        return false;
      }

      const testResult = await testConnection();
      setStatus({
        tone: testResult.success ? "success" : "error",
        text: normalizeStatusText(testResult.message)
      });
      return true;
    } catch (error) {
      setStatus({ tone: "error", text: normalizeStatusText(`保存访问令牌失败: ${String(error)}`) });
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function handleMinimize() {
    await hideWindow();
  }

  async function handleClose() {
    await hideWindow();
  }

  async function handleStartDragging(event: MouseEvent<HTMLElement>) {
    if (event.button !== 0) {
      return;
    }

    await getCurrentWindow().startDragging();
  }

  return (
    <div className="app-shell">
      <header className="title-bar" data-tauri-drag-region onMouseDown={(event) => void handleStartDragging(event)}>
        <div className="title-bar-drag" data-tauri-drag-region>
          <span className="title-bar-text">空调控制器</span>
        </div>
        <div className="title-bar-controls">
          <button
            className="title-bar-btn"
            onMouseDown={(event) => event.stopPropagation()}
            onClick={() => void handleMinimize()}
            title="最小化"
            type="button"
          >
            <span className="title-bar-icon minimize-icon" />
          </button>
          <button
            className="title-bar-btn close-btn"
            onMouseDown={(event) => event.stopPropagation()}
            onClick={() => void handleClose()}
            title="关闭"
            type="button"
          >
            <span className="title-bar-icon close-icon">×</span>
          </button>
        </div>
      </header>

      <aside className="sidebar">
        <div className="brand-block">
          <h1>空调控制器</h1>
          <p>空调控制面板</p>
        </div>

        <nav className="nav-list">
          <button
            className={activeTab === "main" ? "nav-item active" : "nav-item"}
            onClick={() => setActiveTab("main")}
          >
            控制台
          </button>
          <button
            className={activeTab === "config" ? "nav-item active" : "nav-item"}
            onClick={() => setActiveTab("config")}
          >
            配置
          </button>
        </nav>

        <div className="sidebar-footer">
          <span>
            {climateState
              ? climateState.isAvailable
                ? "已连接"
                : "不可用"
              : "未连接"}
          </span>
          <span>
            {climateState
              ? climateState.isOn
                ? "已开机"
                : "已关机"
              : "状态未知"}
          </span>
        </div>
      </aside>

      <main className="content-shell">
        <div className={`status-banner ${status.tone}`}>
          <strong>
            {status.tone === "error"
              ? "错误"
              : status.tone === "success"
                ? "状态"
                : "提示"}
          </strong>
          <span>{status.text}</span>
        </div>

        <div className="page-shell">
          {activeTab === "main" ? (
            <MainPage
              busy={busy}
              config={config}
              onChangeTemperature={handleChangeTemperature}
              onRefresh={handleRefresh}
              onTurnOff={handleTurnOff}
              onTurnOn={handleTurnOn}
              state={climateState}
            />
          ) : (
            <ConfigPage
              busy={busy}
              config={config}
              hasToken={hasToken}
              onSaveConfig={handleSaveConfig}
              onSaveToken={handleSaveToken}
            />
          )}
        </div>
      </main>
    </div>
  );
}

function clampTemperature(value: number, config: AppConfig, climateState: ClimateState | null) {
  const runtimeMin = climateState?.minTemperature ?? config.minTemperature;
  const runtimeMax = climateState?.maxTemperature ?? config.maxTemperature;
  const bounded = Math.min(runtimeMax, Math.max(runtimeMin, value));
  return Math.round(bounded / STEP_CELSIUS) * STEP_CELSIUS;
}

function normalizeStatusText(text: string) {
  return text
    .replace(/(\d+(?:\.\d+)?)\s*摄氏度/g, (_match, value) => `${Math.round(Number(value))} 摄氏度`)
    .replace(/(\d+(?:\.\d+)?)\s*°C/g, (_match, value) => `${Math.round(Number(value))} °C`);
}
