import React, { useEffect, useState } from "react";
import {
  Check,
  Database,
  ExternalLink,
  FolderOpen,
  Gauge,
  KeyRound,
  Languages,
  LayoutDashboard,
  Palette,
  Plus,
  RefreshCw,
  RotateCcw,
  Shield,
  SlidersHorizontal,
  Sparkles,
  Trash2,
  X,
} from "lucide-react";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { THEMES } from "@/lib/themes";
import { ipc } from "@/lib/ipc";
import type {
  Diagnostics,
  ModelProfile,
  ModelProfileData,
  SnapshotSummary,
  Settings,
} from "@/lib/types";
import { cn, formatDate, formatTokens } from "@/lib/utils";
import { Button, IconButton } from "@/components/common/IconButton";
import { Field, Segmented, Switch } from "@/components/common/Primitives";

type Tab =
  | "models"
  | "assignment"
  | "keys"
  | "theme"
  | "permissions"
  | "privacy"
  | "shortcuts"
  | "performance"
  | "auto"
  | "language"
  | "layout"
  | "workspace"
  | "about";

const TABS: Array<{ id: Tab; icon: React.ReactNode; key: string }> = [
  { id: "models", icon: <Sparkles size={16} />, key: "settings.availableModels" },
  { id: "assignment", icon: <SlidersHorizontal size={16} />, key: "settings.modelAssignment" },
  { id: "keys", icon: <KeyRound size={16} />, key: "settings.apiKeys" },
  { id: "theme", icon: <Palette size={16} />, key: "settings.colorTheme" },
  { id: "permissions", icon: <Shield size={16} />, key: "settings.editPermissions" },
  { id: "privacy", icon: <Shield size={16} />, key: "settings.privacy" },
  { id: "shortcuts", icon: <Gauge size={16} />, key: "settings.shortcuts" },
  { id: "performance", icon: <Gauge size={16} />, key: "settings.performance" },
  { id: "auto", icon: <RotateCcw size={16} />, key: "settings.autoBehavior" },
  { id: "language", icon: <Languages size={16} />, key: "settings.language" },
  { id: "layout", icon: <LayoutDashboard size={16} />, key: "settings.layout" },
  { id: "workspace", icon: <FolderOpen size={16} />, key: "settings.workspace" },
  { id: "about", icon: <Database size={16} />, key: "settings.about" },
];

const PROVIDERS: Array<{ id: ModelProfileData["provider"]; label: string; baseUrl: string; model: string }> = [
  { id: "openai_compatible", label: "OpenAI-compatible", baseUrl: "https://api.openai.com/v1", model: "gpt-4.1-mini" },
  { id: "anthropic", label: "Anthropic", baseUrl: "https://api.anthropic.com", model: "claude-sonnet-4-5" },
  { id: "gemini", label: "Google Gemini", baseUrl: "https://generativelanguage.googleapis.com", model: "gemini-2.5-flash" },
  { id: "mock", label: "Mock (offline)", baseUrl: "", model: "mock-1" },
];

function emptyProfile(provider: ModelProfileData["provider"] = "openai_compatible"): ModelProfileData {
  const template = PROVIDERS.find((p) => p.id === provider) ?? PROVIDERS[0];
  return {
    provider,
    label: template.label,
    baseUrl: template.baseUrl,
    model: template.model,
    contextWindow: 128000,
    maxOutput: 8192,
    capabilities: { vision: false, tools: false, json: false, reasoning: false },
    pricing: { inputPerMillion: 0, outputPerMillion: 0, currency: "USD" },
    roles: ["chat"],
    isBuiltin: false,
  };
}

function ModelEditor({
  open,
  initial,
  editing,
  onClose,
  onSaved,
}: {
  open: boolean;
  initial: ModelProfile | null;
  editing: boolean;
  onClose: () => void;
  onSaved: (profile: ModelProfile) => void;
}) {
  const { t } = useI18n();
  const toast = useAppStore((s) => s.toast);
  const [data, setData] = useState<ModelProfileData>(initial?.data ?? emptyProfile());
  const [key, setKey] = useState("");
  const [keySet, setKeySet] = useState(false);
  const [busy, setBusy] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setData(initial?.data ?? emptyProfile());
    setKey("");
    setTestResult(null);
    if (initial) {
      void ipc.secretStatus(initial.id).then((s) => setKeySet(s.configured));
    } else {
      setKeySet(false);
    }
  }, [open, initial]);

  if (!open) return null;

  const save = async () => {
    setBusy(true);
    try {
      let profile: ModelProfile;
      if (editing && initial) {
        profile = await ipc.modelProfileUpdate(initial.id, data, initial.revision);
      } else {
        profile = await ipc.modelProfileCreate(data);
      }
      if (key.trim() && profile.data.provider !== "mock") {
        await ipc.secretSet(profile.id, key.trim());
      }
      onSaved(profile);
      onClose();
      toast("success", t("settings.saved"));
    } catch (error) {
      const e = error as { message?: string; hint?: string };
      toast("error", e.message ?? "Save failed", e.hint);
    } finally {
      setBusy(false);
    }
  };

  const test = async () => {
    if (!initial) {
      toast("info", t("settings.model.test"), t("app.save"));
      return;
    }
    setBusy(true);
    try {
      if (key.trim() && initial.data.provider !== "mock") {
        await ipc.secretSet(initial.id, key.trim());
        setKeySet(true);
      }
      const result = await ipc.modelTest(initial.id);
      setTestResult(
        result.ok
          ? t("settings.model.testOk", { ms: result.latencyMs })
          : t("settings.model.testFail", { message: result.message }),
      );
    } catch (error) {
      const e = error as { message?: string };
      setTestResult(t("settings.model.testFail", { message: e.message ?? "unknown" }));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="settings-submodal">
      <div className="settings-subhead">
        <h2>{editing ? t("settings.model.edit") : t("settings.model.add")}</h2>
        <IconButton label={t("app.close")} size="sm" onClick={onClose}>
          <X size={16} />
        </IconButton>
      </div>
      <div className="settings-form-grid">
        <Field label={t("settings.model.provider")}>
          <select
            value={data.provider}
            onChange={(e) => {
              const provider = e.target.value as ModelProfileData["provider"];
              const template = PROVIDERS.find((p) => p.id === provider);
              setData({
                ...data,
                provider,
                baseUrl: template?.baseUrl ?? data.baseUrl,
                model: template?.model ?? data.model,
                label: template?.label ?? data.label,
              });
            }}
          >
            {PROVIDERS.map((provider) => (
              <option key={provider.id} value={provider.id}>
                {provider.label}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t("settings.model.label")}>
          <input value={data.label} onChange={(e) => setData({ ...data, label: e.target.value })} />
        </Field>
        <Field label={t("settings.model.baseUrl")}>
          <input
            value={data.baseUrl}
            onChange={(e) => setData({ ...data, baseUrl: e.target.value })}
            placeholder="https://api.openai.com/v1"
          />
        </Field>
        <Field label={t("settings.model.modelId")}>
          <input value={data.model} onChange={(e) => setData({ ...data, model: e.target.value })} />
        </Field>
        <Field label={t("settings.model.contextWindow")}>
          <input
            type="number"
            value={data.contextWindow}
            onChange={(e) => setData({ ...data, contextWindow: Number(e.target.value) || 0 })}
          />
        </Field>
        <Field label={t("settings.model.maxOutput")}>
          <input
            type="number"
            value={data.maxOutput}
            onChange={(e) => setData({ ...data, maxOutput: Number(e.target.value) || 0 })}
          />
        </Field>
        {data.provider !== "mock" && (
          <Field label={t("settings.model.key")} hint={keySet ? t("settings.model.keySet") : t("settings.model.keyMissing")}>
            <input
              type="password"
              value={key}
              onChange={(e) => setKey(e.target.value)}
              placeholder={t("settings.model.keyPlaceholder")}
              autoComplete="off"
            />
          </Field>
        )}
      </div>
      <div className="capability-row">
        {(["vision", "tools", "json", "reasoning"] as const).map((cap) => (
          <label key={cap} className="check-row">
            <input
              type="checkbox"
              checked={data.capabilities[cap]}
              onChange={(e) =>
                setData({ ...data, capabilities: { ...data.capabilities, [cap]: e.target.checked } })
              }
            />
            {cap}
          </label>
        ))}
      </div>
      {testResult && <div className="settings-test-result">{testResult}</div>}
      <div className="settings-subfooter">
        <Button variant="ghost" onClick={onClose}>
          {t("app.cancel")}
        </Button>
        {editing && data.provider !== "mock" && (
          <Button onClick={() => void test()} disabled={busy}>
            {busy ? t("settings.model.testing") : t("settings.model.test")}
          </Button>
        )}
        <Button variant="primary" onClick={() => void save()} disabled={busy}>
          <Check size={15} /> {t("app.save")}
        </Button>
      </div>
    </div>
  );
}

export function SettingsModal() {
  const { locale, t } = useI18n();
  const open = useAppStore((s) => s.settingsOpen);
  const setOpen = useAppStore((s) => s.setSettingsOpen);
  const settings = useAppStore((s) => s.settings);
  const models = useAppStore((s) => s.models);
  const saveSettings = useAppStore((s) => s.saveSettings);
  const refreshModels = useAppStore((s) => s.refreshModels);
  const previewTheme = useAppStore((s) => s.previewTheme);
  const cancelThemePreview = useAppStore((s) => s.cancelThemePreview);
  const exportVault = useAppStore((s) => s.exportVault);
  const importVault = useAppStore((s) => s.importVault);
  const createSnapshot = useAppStore((s) => s.createSnapshot);
  const restoreSnapshot = useAppStore((s) => s.restoreSnapshot);
  const toast = useAppStore((s) => s.toast);
  const boot = useAppStore((s) => s.boot);
  const openWorkspace = useAppStore((s) => s.openWorkspace);
  const openDefaultWorkspace = useAppStore((s) => s.openDefaultWorkspace);

  const [tab, setTab] = useState<Tab>("models");
  const [draft, setDraft] = useState<Settings | null>(settings);
  const [diagnostics, setDiagnostics] = useState<Diagnostics | null>(null);
  const [snapshots, setSnapshots] = useState<SnapshotSummary[]>([]);
  const [keyStatus, setKeyStatus] = useState<Record<string, boolean>>({});
  const [editor, setEditor] = useState<{ open: boolean; profile: ModelProfile | null; editing: boolean }>({
    open: false,
    profile: null,
    editing: false,
  });
  const [clearText, setClearText] = useState("");

  useEffect(() => {
    if (open && settings) {
      setDraft(settings);
      setTab("models");
      void ipc.diagnostics().then(setDiagnostics);
      void ipc.snapshotList().then(setSnapshots);
      void refreshModels();
    }
  }, [open, settings, refreshModels]);

  useEffect(() => {
    if (!open) return;
    const load = async () => {
      const entries: Record<string, boolean> = {};
      for (const model of models) {
        if (model.data.provider === "mock") continue;
        try {
          entries[model.id] = (await ipc.secretStatus(model.id)).configured;
        } catch {
          entries[model.id] = false;
        }
      }
      setKeyStatus(entries);
    };
    void load();
  }, [open, models]);

  const current = draft ?? settings;
  if (!open || !current) return null;

  const patch = (updater: (value: Settings) => Settings) => {
    setDraft((value) => (value ? updater(value) : value));
  };

  const save = async () => {
    if (!draft) return;
    const ok = await saveSettings(draft);
    if (ok) setOpen(false);
  };

  const close = () => {
    cancelThemePreview();
    setOpen(false);
  };

  const setField = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    patch((s) => ({ ...s, [key]: value }));
  };

  const chooseWorkspace = async () => {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      directory: true,
      multiple: false,
      title: t("settings.workspace.change"),
    });
    if (typeof selected === "string") {
      if (await openWorkspace(selected)) setOpen(false);
    }
  };

  const chooseDefaultWorkspace = async () => {
    if (await openDefaultWorkspace()) {
      setOpen(false);
      toast("success", t("settings.workspace.default"), t("settings.workspace.current"));
    }
  };

  return (
    <div className="settings-backdrop" onMouseDown={(e) => e.target === e.currentTarget && close()}>
      <div className="settings-modal" role="dialog" aria-modal="true">
        <div className="settings-left">
          <div className="settings-title">
            <h1>{t("settings.title")}</h1>
            <IconButton label={t("app.close")} size="sm" onClick={close}>
              <X size={17} />
            </IconButton>
          </div>
          <nav className="settings-nav">
            {TABS.map((item) => (
              <button
                key={item.id}
                type="button"
                className={cn("settings-tab", tab === item.id && "is-active")}
                onClick={() => setTab(item.id)}
              >
                {item.icon}
                <span>{t(item.key)}</span>
              </button>
            ))}
          </nav>
          <div className="settings-left-foot">
            <span>{t("settings.about.localFirst")}</span>
          </div>
        </div>

        <div className="settings-right">
          <div className="settings-scroll">
            {tab === "models" && (
              <section className="settings-section">
                <h2>{t("settings.availableModels")}</h2>
                <p className="settings-desc">
                  {t("settings.model.rolesHint")} {t("settings.model.keyPlaceholder")}
                </p>
                <button
                  type="button"
                  className="settings-add"
                  onClick={() => setEditor({ open: true, profile: null, editing: false })}
                >
                  <Plus size={16} /> {t("settings.model.add")}
                </button>
                {models.length === 0 && <div className="settings-empty">{t("settings.model.noProfiles")}</div>}
                <div className="model-list">
                  {models.map((model) => (
                    <div key={model.id} className="model-row">
                      <div className="model-row-main">
                        <div className="model-row-title">
                          <strong>{model.data.label}</strong>
                          <span className="badge">{model.data.provider}</span>
                          {model.data.capabilities.vision && <span className="badge">{t("settings.model.vision")}</span>}
                          {model.data.capabilities.tools && <span className="badge">{t("settings.model.tools")}</span>}
                          {model.data.isBuiltin && <span className="badge">{t("settings.model.builtin")}</span>}
                        </div>
                        <div className="model-row-sub">
                          {model.data.model} · {formatTokens(model.data.contextWindow)} ctx
                          {model.data.provider !== "mock" && (
                            <span className={cn("key-state", keyStatus[model.id] && "is-set")}>
                              {keyStatus[model.id] ? t("settings.model.keySet") : t("settings.model.keyMissing")}
                            </span>
                          )}
                        </div>
                      </div>
                      <div className="model-row-actions">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => setEditor({ open: true, profile: model, editing: true })}
                        >
                          {t("settings.model.edit")}
                        </Button>
                        {!model.data.isBuiltin && (
                          <IconButton
                            label={t("settings.model.delete")}
                            size="sm"
                            onClick={async () => {
                              if (!window.confirm(t("settings.model.deleteConfirm", { name: model.data.label }))) return;
                              await ipc.modelProfileDelete(model.id);
                              await refreshModels();
                            }}
                          >
                            <Trash2 size={14} />
                          </IconButton>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </section>
            )}

            {tab === "assignment" && (
              <section className="settings-section">
                <h2>{t("settings.modelAssignment")}</h2>
                <div className="settings-form-grid">
                  {(["chat", "reasoning", "title", "summary"] as const).map((role) => (
                    <Field key={role} label={t(`settings.assignment.${role}`)}>
                      <select
                        value={current.modelAssignments[role] ?? ""}
                        onChange={(e) =>
                          patch((s) => ({
                            ...s,
                            modelAssignments: { ...s.modelAssignments, [role]: e.target.value || null },
                          }))
                        }
                      >
                        <option value="">{t("settings.assignment.sameAsChat")}</option>
                        {models.map((model) => (
                          <option key={model.id} value={model.id}>
                            {model.data.label}
                          </option>
                        ))}
                      </select>
                    </Field>
                  ))}
                </div>
              </section>
            )}

            {tab === "keys" && (
              <section className="settings-section">
                <h2>{t("settings.apiKeys")}</h2>
                <p className="settings-desc">{t("settings.model.keyPlaceholder")}</p>
                <div className="key-list">
                  {models
                    .filter((m) => m.data.provider !== "mock")
                    .map((model) => (
                      <div key={model.id} className="key-row">
                        <div>
                          <strong>{model.data.label}</strong>
                          <div className="model-row-sub">{model.data.baseUrl || "—"}</div>
                        </div>
                        <span className={cn("key-state", keyStatus[model.id] && "is-set")}>
                          {keyStatus[model.id] ? t("settings.model.keySet") : t("settings.model.keyMissing")}
                        </span>
                        <Button
                          size="sm"
                          onClick={() => setEditor({ open: true, profile: model, editing: true })}
                        >
                          {keyStatus[model.id] ? t("settings.model.saveKey") : t("settings.model.key")}
                        </Button>
                      </div>
                    ))}
                  {models.filter((m) => m.data.provider !== "mock").length === 0 && (
                    <div className="settings-empty">{t("settings.model.noProfiles")}</div>
                  )}
                </div>
              </section>
            )}

            {tab === "theme" && (
              <section className="settings-section">
                <h2>{t("settings.colorTheme")}</h2>
                <p className="settings-desc">{t("settings.theme.subtitle")}</p>
                <div className="theme-grid">
                  {THEMES.map((theme) => (
                    <button
                      key={theme.id}
                      type="button"
                      className={cn("theme-card", current.themeId === theme.id && "is-active")}
                      onClick={() => {
                        setField("themeId", theme.id);
                        previewTheme(theme.id);
                      }}
                    >
                      <span className="theme-card-preview" style={{ background: theme.tokens.bg }}>
                        <span style={{ background: theme.tokens.card }} />
                        <span style={{ background: theme.tokens.brand }} />
                      </span>
                      <span>{t(`theme.${theme.id}`)}</span>
                      {current.themeId === theme.id && <Check size={14} />}
                    </button>
                  ))}
                </div>
              </section>
            )}

            {tab === "permissions" && (
              <section className="settings-section">
                <h2>{t("settings.editPermissions")}</h2>
                <Switch
                  checked={current.editPermissions.readOnly}
                  onChange={(value) =>
                    setField("editPermissions", { ...current.editPermissions, readOnly: value })
                  }
                  label={t("settings.permissions.readOnly")}
                  description={t("settings.permissions.readOnlyDesc")}
                />
                <p className="settings-desc">{t("settings.permissions.lockDesc")}</p>
              </section>
            )}

            {tab === "privacy" && (
              <section className="settings-section">
                <h2>{t("settings.privacy")}</h2>
                <div className="privacy-block">
                  <Shield size={18} />
                  <div>
                    <strong>{t("settings.privacy.local")}</strong>
                    <p>{t("settings.privacy.localDesc")}</p>
                  </div>
                </div>
                <div className="privacy-paths">
                  <Field label={t("settings.privacy.dataPath")}>
                    <input readOnly value={diagnostics?.dataDir ?? ""} />
                  </Field>
                  <Field label={t("settings.privacy.dbPath")}>
                    <input readOnly value={diagnostics?.dbPath ?? ""} />
                  </Field>
                </div>
                <div className="privacy-targets">
                  <strong>{t("settings.privacy.network")}</strong>
                  <p>{t("settings.privacy.networkDesc")}</p>
                  {models
                    .filter((m) => m.data.provider !== "mock" && m.data.baseUrl)
                    .map((m) => (
                      <code key={m.id}>{m.data.baseUrl}</code>
                    ))}
                </div>
                <div className="settings-button-row">
                  <Button onClick={() => void exportVault()}>{t("settings.privacy.export")}</Button>
                  <Button onClick={() => void importVaultDialog()}>{t("app.import")}</Button>
                </div>
                <div className="danger-zone">
                  <strong>{t("settings.privacy.clear")}</strong>
                  <p>{t("settings.privacy.clearConfirm")}</p>
                  <div className="settings-button-row">
                    <input
                      value={clearText}
                      onChange={(e) => setClearText(e.target.value)}
                      placeholder="DELETE"
                    />
                    <Button
                      variant="danger"
                      disabled={clearText !== "DELETE"}
                      onClick={async () => {
                        await ipc.vaultClear(clearText);
                        setClearText("");
                        toast("success", t("settings.privacy.clearDone"));
                        await useAppStore.getState().refreshBootstrap();
                      }}
                    >
                      {t("settings.privacy.clear")}
                    </Button>
                  </div>
                </div>
              </section>
            )}

            {tab === "shortcuts" && (
              <section className="settings-section">
                <h2>{t("settings.shortcuts")}</h2>
                <p className="settings-desc">{t("settings.shortcuts.subtitle")}</p>
                <div className="shortcut-list">
                  {Object.entries(current.shortcuts).map(([action, keys]) => (
                    <div key={action} className="shortcut-row">
                      <span>{action}</span>
                      <kbd>{keys}</kbd>
                    </div>
                  ))}
                </div>
              </section>
            )}

            {tab === "performance" && (
              <section className="settings-section">
                <h2>{t("settings.performance")}</h2>
                <p className="settings-desc">{t("settings.performance.subtitle")}</p>
                {diagnostics && (
                  <div className="diagnostic-grid">
                    <div><span>{t("settings.about.version")}</span><strong>{diagnostics.version}</strong></div>
                    <div><span>Nodes</span><strong>{diagnostics.counts.nodes}</strong></div>
                    <div><span>Edges</span><strong>{diagnostics.counts.edges}</strong></div>
                    <div><span>Cards</span><strong>{diagnostics.counts.cards}</strong></div>
                    <div><span>Messages</span><strong>{diagnostics.counts.messages}</strong></div>
                    <div><span>Keychain</span><strong>{diagnostics.keychainAvailable ? "yes" : "no"}</strong></div>
                  </div>
                )}
                <div className="settings-subhead-row">
                  <strong>{t("settings.performance.snapshots")}</strong>
                  <Button size="sm" onClick={() => void createSnapshot().then(() => ipc.snapshotList().then(setSnapshots))}>
                    <Plus size={14} /> {t("settings.performance.snapshotCreate")}
                  </Button>
                </div>
                <div className="snapshot-list">
                  {snapshots.map((snapshot) => (
                    <div key={snapshot.id} className="snapshot-row">
                      <div>
                        <strong>{snapshot.label}</strong>
                        <div className="model-row-sub">
                          {formatDate(snapshot.createdAt, locale)} · {snapshot.nodeCount} nodes
                        </div>
                      </div>
                      <Button
                        size="sm"
                        onClick={() => {
                          if (window.confirm(t("settings.performance.snapshotRestoreConfirm", { label: snapshot.label }))) {
                            void restoreSnapshot(snapshot.id);
                          }
                        }}
                      >
                        <RefreshCw size={14} /> {t("settings.performance.snapshotRestore")}
                      </Button>
                    </div>
                  ))}
                  {snapshots.length === 0 && <div className="settings-empty">{t("settings.performance.noSnapshots")}</div>}
                </div>
              </section>
            )}

            {tab === "auto" && (
              <section className="settings-section">
                <h2>{t("settings.autoBehavior")}</h2>
                <Switch
                  checked={current.autoBehavior.autoTitle}
                  onChange={(value) =>
                    setField("autoBehavior", { ...current.autoBehavior, autoTitle: value })
                  }
                  label={t("settings.auto.autoTitle")}
                />
                <Switch
                  checked={current.autoBehavior.confirmDelete}
                  onChange={(value) =>
                    setField("autoBehavior", { ...current.autoBehavior, confirmDelete: value })
                  }
                  label={t("settings.auto.confirmDelete")}
                />
                <Switch
                  checked={current.autoBehavior.defaultSearch}
                  onChange={(value) =>
                    setField("autoBehavior", { ...current.autoBehavior, defaultSearch: value })
                  }
                  label={t("settings.auto.defaultSearch")}
                />
                <Field label={t("settings.auto.checkpoint")}>
                  <input
                    type="number"
                    value={current.autoBehavior.checkpointMs}
                    onChange={(e) =>
                      setField("autoBehavior", {
                        ...current.autoBehavior,
                        checkpointMs: Number(e.target.value) || 450,
                      })
                    }
                  />
                </Field>
              </section>
            )}

            {tab === "language" && (
              <section className="settings-section">
                <h2>{t("settings.language")}</h2>
                <p className="settings-desc">{t("settings.language.subtitle")}</p>
                <div className="language-list">
                  {[
                    { id: "zh-CN", label: "简体中文" },
                    { id: "zh-TW", label: "繁體中文" },
                    { id: "en", label: "English" },
                  ].map((lang) => (
                    <button
                      key={lang.id}
                      type="button"
                      className={cn("language-option", current.locale === lang.id && "is-active")}
                      onClick={() => setField("locale", lang.id as Settings["locale"])}
                    >
                      <Languages size={18} />
                      <span>{lang.label}</span>
                      {current.locale === lang.id && <Check size={16} />}
                    </button>
                  ))}
                </div>
              </section>
            )}

            {tab === "layout" && (
              <section className="settings-section">
                <h2>{t("settings.layout")}</h2>
                <p className="settings-desc">{t("settings.layout.subtitle")}</p>
                <Field label={t("settings.layout.density")}>
                  <Segmented
                    value={current.layout.density}
                    options={[
                      { value: "comfortable", label: t("settings.layout.comfortable") },
                      { value: "compact", label: t("settings.layout.compact") },
                    ]}
                    onChange={(value) => setField("layout", { ...current.layout, density: value })}
                  />
                </Field>
                <Field label={`${t("settings.layout.cardWidth")}: ${current.layout.cardWidth}px`}>
                  <input
                    type="range"
                    min={620}
                    max={980}
                    step={20}
                    value={current.layout.cardWidth}
                    onChange={(e) =>
                      setField("layout", { ...current.layout, cardWidth: Number(e.target.value) })
                    }
                  />
                </Field>
                <Switch
                  checked={current.layout.showTree}
                  onChange={(value) => setField("layout", { ...current.layout, showTree: value })}
                  label={t("settings.layout.showTree")}
                />
                <Switch
                  checked={current.layout.reducedMotion}
                  onChange={(value) => setField("layout", { ...current.layout, reducedMotion: value })}
                  label={t("settings.layout.reducedMotion")}
                />
                <Switch
                  checked={current.layout.force2dTree}
                  onChange={(value) => setField("layout", { ...current.layout, force2dTree: value })}
                  label={t("settings.layout.force2d")}
                />
              </section>
            )}

            {tab === "workspace" && (
              <section className="settings-section">
                <h2>{t("settings.workspace")}</h2>
                <p className="settings-desc">{t("settings.workspace.subtitle")}</p>
                <div className="workspace-card">
                  <div className="workspace-block">
                    <span>{t("settings.workspace.current")}</span>
                    <strong>{boot?.workspace.root ?? "—"}</strong>
                  </div>
                  <div className="workspace-block">
                    <span>{t("settings.workspace.dataDir")}</span>
                    <code>{boot?.workspace.internal ?? "—"}</code>
                  </div>
                  <div className="workspace-actions">
                    <Button variant="primary" onClick={() => void chooseWorkspace()}>
                      <FolderOpen size={15} /> {t("settings.workspace.change")}
                    </Button>
                    <Button variant="ghost" onClick={() => void chooseDefaultWorkspace()}>
                      <RotateCcw size={15} /> {t("settings.workspace.default")}
                    </Button>
                    <Button variant="ghost" onClick={() => void ipc.openDataDir()}>
                      <ExternalLink size={15} /> {t("settings.workspace.open")}
                    </Button>
                  </div>
                </div>
                <div className="workspace-access">
                  <span>{t("settings.workspace.access")}</span>
                  <button type="button" className="access-pill is-active">
                    {t("settings.workspace.rw")}
                  </button>
                  <button type="button" className="access-pill" disabled>
                    {t("settings.workspace.ro")}
                  </button>
                  <button type="button" className="access-pill" disabled>
                    {t("settings.workspace.full")}
                  </button>
                </div>
                <p className="settings-desc">{t("settings.workspace.roadmap")}</p>
                <p className="settings-desc">{t("settings.workspace.changeHint")}</p>
              </section>
            )}

            {tab === "about" && (
              <section className="settings-section">
                <h2>{t("settings.about")}</h2>
                <div className="about-card">
                  <div className="about-logo">Sporelink</div>
                  <div className="about-meta">
                    <div>
                      <span>{t("settings.about.version")}</span>
                      <strong>{useAppStore.getState().boot?.version ?? "0.1.0-alpha"}</strong>
                    </div>
                    <div>
                      <span>{t("settings.about.license")}</span>
                      <strong>{t("settings.about.licenseValue")}</strong>
                    </div>
                  </div>
                </div>
                <div className="about-links">
                  <a href="https://velesdb.com" target="_blank" rel="noreferrer">
                    <Database size={15} /> {t("settings.about.veles")} <ExternalLink size={13} />
                  </a>
                  <p>{t("settings.about.velesDesc")}</p>
                  <a href="https://github.com/BlueIceXMO/Sporelink" target="_blank" rel="noreferrer">
                    <ExternalLink size={15} /> {t("settings.about.repository")}
                  </a>
                </div>
                <p className="settings-desc">{t("settings.about.localFirst")}</p>
              </section>
            )}
          </div>

          <div className="settings-footer">
            <Button variant="ghost" onClick={close}>
              {t("app.cancel")}
            </Button>
            <Button variant="primary" onClick={() => void save()}>
              <Check size={16} /> {t("app.save")}
            </Button>
          </div>
        </div>

        <ModelEditor
          open={editor.open}
          initial={editor.profile}
          editing={editor.editing}
          onClose={() => setEditor({ open: false, profile: null, editing: false })}
          onSaved={async () => {
            await refreshModels();
            setEditor({ open: false, profile: null, editing: false });
          }}
        />
      </div>
    </div>
  );
}

async function importVaultDialog() {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const path = await open({
    multiple: false,
    filters: [{ name: "Sporelink vault", extensions: ["json"] }],
  });
  if (typeof path === "string") {
    await useAppStore.getState().importVault(path);
  }
}
