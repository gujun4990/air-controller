import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import MainPage from "../pages/MainPage";
import ConfigPage from "../pages/ConfigPage";
import {
  deleteToken,
  exportConfig,
  getConfig,
  getConfigDirectory,
  getLaunchOnStartup,
  hasToken as checkToken,
  importLegacyConfig,
  refreshState,
  runAutoPowerOn,
  saveConfig,
  saveToken,
  setTemperature,
  turnOff,
  turnOn
} from "../lib/commands";
import { defaultConfig, type AppConfig, type ClimateState } from "../lib/types";

type TabKey = "main" | "config";

export default function App() {
  const [activeTab, setActiveTab] = useState<TabKey>("main");
  const [config, setConfig] = useState<AppConfig>(defaultConfig);
  const [climateState, setClimateState] = useState<ClimateState | null>(null);
  const [message, setMessage] = useState("正在加载配置...");
  const [busy, setBusy] = useState(false);
  const [hasToken, setHasToken] = useState(false);

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
      setMessage(configResult.message || "配置已加载");

      if (configResult.success && tokenResult.data) {
        if (nextConfig.autoPowerOnOnStartup) {
          await runClimateAction(runAutoPowerOn);
        } else {
          await handleRefresh();
        }
      }
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
      setMessage(result.message);
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
    const next = Math.min(config.maxTemperature, Math.max(config.minTemperature, base + delta));
    await runClimateAction(() => setTemperature(next));
  }

  async function handleRunAutoPowerOn() {
    await runClimateAction(runAutoPowerOn);
  }

  async function handleSaveConfig(nextConfig: AppConfig) {
    setBusy(true);
    try {
      const result = await saveConfig(nextConfig);
      if (result.data) {
        setConfig(result.data);
      }
      setMessage(result.message);
    } finally {
      setBusy(false);
    }
  }

  async function handleSaveToken(token: string) {
    setBusy(true);
    try {
      const result = await saveToken(token);
      setHasToken(Boolean(result.data));
      setMessage(result.message);
    } finally {
      setBusy(false);
    }
  }

  async function handleDeleteToken() {
    setBusy(true);
    try {
      const result = await deleteToken();
      setHasToken(false);
      setMessage(result.message);
    } finally {
      setBusy(false);
    }
  }

  async function handleImportLegacy() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "JSON", extensions: ["json"] }],
      title: "选择旧版 appsettings.json"
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    setBusy(true);
    try {
      const result = await importLegacyConfig(selected);
      if (result.data) {
        setConfig(result.data);
      }
      const tokenResult = await checkToken();
      setHasToken(Boolean(tokenResult.data));
      setMessage(result.message);
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
      setMessage(result.message);
    } finally {
      setBusy(false);
    }
  }

  async function handleOpenConfigDirectory() {
    setBusy(true);
    try {
      const result = await getConfigDirectory();
      if (!result.success || !result.data) {
        setMessage(result.message);
        return;
      }

      await openPath(result.data);
      setMessage(`已打开配置目录：${result.data}`);
    } catch (error) {
      setMessage(`打开配置目录失败: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div>
          <h1>AirController</h1>
          <p>跨平台 Home Assistant 空调控制器</p>
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
        {activeTab === "main" ? (
          <MainPage
            busy={busy}
            config={config}
            onChangeTemperature={handleChangeTemperature}
            onRefresh={handleRefresh}
            onRunAutoPowerOn={handleRunAutoPowerOn}
            onTurnOff={handleTurnOff}
            onTurnOn={handleTurnOn}
            state={climateState}
            statusMessage={message}
          />
        ) : (
          <ConfigPage
            busy={busy}
            config={config}
            hasToken={hasToken}
            onDeleteToken={handleDeleteToken}
            onExportConfig={handleExportConfig}
            onImportLegacy={handleImportLegacy}
            onOpenConfigDirectory={handleOpenConfigDirectory}
            onSaveConfig={handleSaveConfig}
            onSaveToken={handleSaveToken}
          />
        )}
      </main>
    </div>
  );
}
