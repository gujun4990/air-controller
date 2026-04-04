import { type FormEvent, useEffect, useState } from "react";
import type { AppConfig } from "../lib/types";

type Props = {
  config: AppConfig;
  hasToken: boolean;
  busy: boolean;
  onSaveSettings: (config: AppConfig, token: string) => Promise<boolean>;
};

export default function ConfigPage({
  config,
  hasToken,
  busy,
  onSaveSettings
}: Props) {
  const [draft, setDraft] = useState<AppConfig>(config);
  const [tokenInput, setTokenInput] = useState("");

  useEffect(() => {
    setDraft(config);
  }, [config]);

  function update<K extends keyof AppConfig>(key: K, value: AppConfig[K]) {
    setDraft((current) => ({ ...current, [key]: value }));
  }

  function handleLaunchOnStartupChange(enabled: boolean) {
    setDraft((current) => ({
      ...current,
      launchOnSystemStartup: enabled,
      autoPowerOnOnStartup: enabled ? current.autoPowerOnOnStartup : false
    }));
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const payload: AppConfig = {
      ...draft,
      autoPowerOnOnStartup: draft.launchOnSystemStartup && draft.autoPowerOnOnStartup
    };

    const saved = await onSaveSettings(payload, tokenInput);
    if (!saved) {
      setDraft(config);
      return;
    }

    setDraft(payload);
    setTokenInput("");
  }

  return (
    <section className="panel config-shell">
      <div className="panel-header config-header">
        <div>
          <span className="eyebrow">配置中心</span>
          <h2>配置</h2>
          <p>保存基础配置，访问令牌独立进入系统密钥链。</p>
        </div>
      </div>

      <form className="config-form" onSubmit={handleSubmit}>
        <div className="config-grid">
          <section className="settings-section">
            <div className="settings-head">
              <span className="eyebrow">服务连接</span>
              <h3>连接信息</h3>
            </div>

            <div className="settings-grid">
              <label>
                <span>服务地址</span>
                <input
                  value={draft.baseUrl}
                  onChange={(event) => update("baseUrl", event.target.value)}
                />
              </label>

              <label>
                <span>空调ID</span>
                <input
                  value={draft.climateEntityId}
                  onChange={(event) => update("climateEntityId", event.target.value)}
                />
              </label>

              <label>
                <span>访问令牌</span>
                <input
                  type="password"
                  placeholder={
                    hasToken ? "留空则保留当前已保存访问令牌" : "输入新的访问令牌"
                  }
                  value={tokenInput}
                  onChange={(event) => setTokenInput(event.target.value)}
                />
              </label>
            </div>
          </section>

          <section className="settings-section">
            <div className="settings-head">
              <span className="eyebrow">启动设置</span>
              <h3>启动方式</h3>
            </div>

            <div className="settings-grid checkbox-grid">
              <label className="checkbox-row">
                <input
                  type="checkbox"
                  checked={draft.launchOnSystemStartup}
                  onChange={(event) => handleLaunchOnStartupChange(event.target.checked)}
                />
                <span>系统登录后自动启动应用</span>
              </label>

              <label className="checkbox-row">
                <input
                  type="checkbox"
                  checked={draft.autoPowerOnOnStartup}
                  disabled={!draft.launchOnSystemStartup}
                  onChange={(event) => update("autoPowerOnOnStartup", event.target.checked)}
                />
                <span>系统自启动时自动开启空调</span>
              </label>
            </div>
          </section>
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
