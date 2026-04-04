import { type FormEvent, useEffect, useState } from "react";
import type { AppConfig } from "../lib/types";

type Props = {
  config: AppConfig;
  hasToken: boolean;
  busy: boolean;
  onSaveConfig: (config: AppConfig) => Promise<void>;
  onSaveToken: (token: string) => Promise<void>;
  onExportConfig: () => Promise<void>;
};

export default function ConfigPage({
  config,
  hasToken,
  busy,
  onSaveConfig,
  onSaveToken,
  onExportConfig
}: Props) {
  const [draft, setDraft] = useState<AppConfig>(config);
  const [tokenInput, setTokenInput] = useState("");

  useEffect(() => {
    setDraft(config);
  }, [config]);

  function update<K extends keyof AppConfig>(key: K, value: AppConfig[K]) {
    setDraft((current) => ({ ...current, [key]: value }));
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onSaveConfig(draft);

    const token = tokenInput.trim();
    if (token.length > 0) {
      await onSaveToken(token);
      setTokenInput("");
    }
  }

  return (
    <section className="panel">
      <div className="panel-header">
        <div>
          <h2>配置</h2>
          <p>保存基础配置，Token 独立进入系统密钥链。</p>
        </div>
      </div>

      <form className="config-form" onSubmit={handleSubmit}>
        <label>
          <span>HA 地址</span>
          <input value={draft.baseUrl} onChange={(event) => update("baseUrl", event.target.value)} />
        </label>

        <label>
          <span>空调实体 ID</span>
          <input value={draft.climateEntityId} onChange={(event) => update("climateEntityId", event.target.value)} />
        </label>

        <label>
          <span>Token</span>
          <input
            type="password"
            placeholder={hasToken ? "留空则保留当前已保存 token" : "输入新的 Long-Lived Token"}
            value={tokenInput}
            onChange={(event) => setTokenInput(event.target.value)}
          />
        </label>

        <div className="checkbox-grid">
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={draft.launchOnSystemStartup}
              onChange={(event) => update("launchOnSystemStartup", event.target.checked)}
            />
            <span>系统登录后自动启动</span>
          </label>
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={draft.autoPowerOnOnStartup}
              onChange={(event) => update("autoPowerOnOnStartup", event.target.checked)}
            />
            <span>程序启动后自动开空调</span>
          </label>
        </div>

        <div className="field-grid">
          <label>
            <span>启动延迟（秒）</span>
            <input
              type="number"
              value={draft.startupDelaySeconds}
              onChange={(event) => update("startupDelaySeconds", Number(event.target.value))}
            />
          </label>
          <label>
            <span>失败重试次数</span>
            <input type="number" value={draft.retryCount} onChange={(event) => update("retryCount", Number(event.target.value))} />
          </label>
          <label>
            <span>默认温度</span>
            <input
              type="number"
              step="0.1"
              value={draft.defaultTemperature}
              onChange={(event) => update("defaultTemperature", Number(event.target.value))}
            />
          </label>
          <label>
            <span>最小温度</span>
            <input type="number" step="0.1" value={draft.minTemperature} onChange={(event) => update("minTemperature", Number(event.target.value))} />
          </label>
          <label>
            <span>最大温度</span>
            <input type="number" step="0.1" value={draft.maxTemperature} onChange={(event) => update("maxTemperature", Number(event.target.value))} />
          </label>
          <label>
            <span>调温步长</span>
            <input
              type="number"
              step="0.1"
              value={draft.temperatureStep}
              onChange={(event) => update("temperatureStep", Number(event.target.value))}
            />
          </label>
        </div>

        <div className="actions-row">
          <button disabled={busy} type="submit">
            保存配置
          </button>
          <button className="ghost" disabled={busy} type="button" onClick={() => void onExportConfig()}>
            导出配置
          </button>
        </div>
      </form>
    </section>
  );
}
