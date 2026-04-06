import { type FormEvent, useEffect, useState } from "react";
import type { AppConfig } from "../lib/types";

type Props = {
  config: AppConfig;
  busy: boolean;
  onSave: (config: AppConfig, token: string) => Promise<boolean>;
  onRequireToken: () => void;
};

export default function ConfigPage({
  config,
  busy,
  onSave,
  onRequireToken
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

    const payload: AppConfig = { ...draft };
    const token = tokenInput.trim();

    if (token.length === 0) {
      onRequireToken();
      return;
    }

    const saved = await onSave(payload, token);
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
          <p>请联系管理员获取服务地址、空调ID和访问令牌后再进行配置。</p>
        </div>
      </div>

      <form className="config-form" onSubmit={handleSubmit}>
        <section className="settings-section primary-settings">
          <div className="settings-head">
            <span className="eyebrow">服务连接</span>
            <h3>连接信息</h3>
          </div>

          <div className="settings-grid single-column-grid">
            <label>
              <span>
                服务地址 <strong className="required-mark">*</strong>
              </span>
              <input value={draft.baseUrl} onChange={(event) => update("baseUrl", event.target.value)} />
            </label>

            <label>
              <span>
                空调ID <strong className="required-mark">*</strong>
              </span>
              <input
                placeholder="例如：climate.living_room_ac"
                value={draft.climateEntityId}
                onChange={(event) => update("climateEntityId", event.target.value)}
              />
            </label>

            <label>
              <span>
                访问令牌 <strong className="required-mark">*</strong>
              </span>
              <input
                required
                type="password"
                value={tokenInput}
                onChange={(event) => setTokenInput(event.target.value)}
              />
            </label>
          </div>

          <div className="actions-row config-actions">
            <button disabled={busy} type="submit">
              保存配置
            </button>
          </div>
        </section>
      </form>
    </section>
  );
}
