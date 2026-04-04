import { type FormEvent, useEffect, useState } from "react";
import { defaultConfig, type AppConfig } from "../lib/types";

type Props = {
  config: AppConfig;
  hasToken: boolean;
  busy: boolean;
  onSaveConfig: (config: AppConfig) => Promise<void>;
  onSaveToken: (token: string) => Promise<void>;
};

export default function ConfigPage({
  config,
  hasToken,
  busy,
  onSaveConfig,
  onSaveToken
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
    await onSaveConfig({
      ...draft,
      startupDelaySeconds: defaultConfig.startupDelaySeconds,
      retryCount: defaultConfig.retryCount,
      defaultTemperature: defaultConfig.defaultTemperature,
      minTemperature: defaultConfig.minTemperature,
      maxTemperature: defaultConfig.maxTemperature,
      temperatureStep: defaultConfig.temperatureStep
    });

    const token = tokenInput.trim();
    if (token.length > 0) {
      await onSaveToken(token);
      setTokenInput("");
    }
  }

  return (
    <section className="panel config-shell">
      <div className="panel-header config-header">
        <div>
          <span className="eyebrow">SETTINGS</span>
          <h2>配置</h2>
          <p>保存基础配置，Token 独立进入系统密钥链。</p>
        </div>
      </div>

      <form className="config-form" onSubmit={handleSubmit}>
        <div className="settings-section">
          <div className="settings-head">
            <span className="eyebrow">HOME ASSISTANT</span>
            <h3>连接设置</h3>
          </div>

          <div className="settings-grid single-column">
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
          </div>
        </div>

        <div className="settings-section">
          <div className="settings-head">
            <span className="eyebrow">STARTUP</span>
            <h3>启动行为</h3>
          </div>

          <div className="checkbox-grid settings-grid">
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
        </div>

        <div className="actions-row config-actions">
          <button disabled={busy} type="submit">
            保存配置
          </button>
        </div>
      </form>
    </section>
  );
}
