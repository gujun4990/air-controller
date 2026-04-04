import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import MainPage from "../pages/MainPage";
import ConfigPage from "../pages/ConfigPage";
import {
  exportConfig,
  getConfig,
  getLaunchOnStartup,
  hasToken as checkToken,
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

export default function App() {
  const [activeTab, setActiveTab] = useState<TabKey>("main");
  const [config, setConfig] = useState<AppConfig>(defaultConfig);
  const [climateState, setClimateState] = useState<ClimateState | null>(null);
  const [busy, setBusy] = useState(false);
  const [hasToken, setHasToken] = useState(false);
  const [status, setStatus] = useState<{ tone: StatusTone; text: string }>({
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
      const [configResult, tokenResult, startupResult] = await Promise.all([getConfig(), checkToken(), getLaunchOnStartup()]);
      const nextConfig = configResult.data
        ? {
            ...configResult.data,
            launchOnSystemStartup: startupResult.data ?? configResult.data.launchOnSystemStartup
          }
        : defaultConfig;

      if (configResult.data) {
        setConfig(nextConfig);
      }
      setHasToken(Boolean(tokenResult.data));

      if (!configResult.success) {
        setStatus({ tone: "error", text: configResult.message });
        return;
      }

      if (!tokenResult.data) {
        setStatus({ tone: "info", text: "请先在配置页保存 Home Assistant Token。" });
        return;
      }

      if (nextConfig.autoPowerOnOnStartup) {
        await runClimateAction(runAutoPowerOn);
      } else {
        await handleRefresh();
      }
    } catch (error) {
      setStatus({ tone: "error", text: `初始化失败: ${String(error)}` });
    } finally {
      setBusy(false);
    }
  }

  async function runClimateAction(action: () => Promise<{ success: boolean; message: string; data: ClimateState | null }>) {
    setBusy(true);
    try {
      const result = await action();
      if (result.data) {
        setClimateState(result.data);
      }
      setStatus({ tone: result.success ? "success" : "error", text: result.message });
    } catch (error) {
      setStatus({ tone: "error", text: `请求失败: ${String(error)}` });
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
    const next = clampTemperature(base + delta, config);
    await runClimateAction(() => setTemperature(next));
  }

  async function handleSaveConfig(nextConfig: AppConfig) {
    setBusy(true);
    try {
      const result = await saveConfig(nextConfig);
      setStatus({ tone: result.success ? "success" : "error", text: result.message });

      if (!result.success) {
        return;
      }

      if (result.data) {
        setConfig(result.data);
      }

      if (hasToken) {
        const testResult = await testConnection();
        setStatus({ tone: testResult.success ? "success" : "error", text: testResult.message });
      }
    } catch (error) {
      setStatus({ tone: "error", text: `保存配置失败: ${String(error)}` });
    } finally {
      setBusy(false);
    }
  }

  async function handleSaveToken(token: string) {
    setBusy(true);
    try {
      const result = await saveToken(token);
      setHasToken(Boolean(result.data));
      if (!result.success) {
        setStatus({ tone: "error", text: result.message });
        return;
      }

      const testResult = await testConnection();
      setStatus({ tone: testResult.success ? "success" : "error", text: testResult.message });
    } catch (error) {
      setStatus({ tone: "error", text: `保存 Token 失败: ${String(error)}` });
    } finally {
      setBusy(false);
    }
  }

  async function handleExportConfig() {
    const selected = await save({
      filters: [{ name: "JSON", extensions: ["json"] }],
      title: "导出配置",
      defaultPath: "air-controller.config.json"
    });

    if (!selected) {
      return;
    }

    setBusy(true);
    try {
      const result = await exportConfig(selected);
      setStatus({ tone: result.success ? "success" : "error", text: result.message });
    } catch (error) {
      setStatus({ tone: "error", text: `导出配置失败: ${String(error)}` });
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div>
          <h1>AirController</h1>
          <p>Home Assistant 空调控制</p>
        </div>

        <nav className="nav-list">
          <button className={activeTab === "main" ? "nav-item active" : "nav-item"} onClick={() => setActiveTab("main")}>
            控制台
          </button>
          <button className={activeTab === "config" ? "nav-item active" : "nav-item"} onClick={() => setActiveTab("config")}>
            配置
          </button>
        </nav>

        <div className="sidebar-footer">
          <span>{hasToken ? "Token 已保存" : "尚未保存 Token"}</span>
          <span>{busy ? "正在处理请求" : "空闲"}</span>
        </div>
      </aside>

      <main className="content-shell">
        <div className={`status-banner ${status.tone}`}>
          <strong>{status.tone === "error" ? "错误" : status.tone === "success" ? "状态" : "提示"}</strong>
          <span>{status.text}</span>
        </div>

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
      </main>
    </div>
  );
}

function clampTemperature(value: number, config: AppConfig) {
  const bounded = Math.min(config.maxTemperature, Math.max(config.minTemperature, value));
  const step = config.temperatureStep || 1;
  const normalized = Math.round(bounded / step) * step;
  return Number(normalized.toFixed(1));
}
