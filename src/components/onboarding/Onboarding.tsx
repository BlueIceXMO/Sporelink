import { useMemo, useState } from "react";
import { ArrowRight, Check, Globe2, Plus, Sparkles } from "lucide-react";
import { useAppStore } from "@/state/store";
import { useI18n, translate } from "@/lib/i18n";
import { THEMES } from "@/lib/themes";
import { Button } from "@/components/common/IconButton";
import { cn } from "@/lib/utils";

const LANGS = [
  { id: "zh-CN", label: "简体中文", sub: "Simplified Chinese" },
  { id: "zh-TW", label: "繁體中文", sub: "Traditional Chinese" },
  { id: "en", label: "English", sub: "English" },
];

export function Onboarding() {
  const settings = useAppStore((s) => s.settings);
  const models = useAppStore((s) => s.models);
  const saveSettings = useAppStore((s) => s.saveSettings);
  const previewTheme = useAppStore((s) => s.previewTheme);
  const setNewProjectOpen = useAppStore((s) => s.setNewProjectOpen);
  const openSample = useAppStore((s) => s.openSample);
  const [step, setStep] = useState(() => {
    if (!settings) return 0;
    if (!settings.onboarding.themeDone) return 0;
    if (!settings.onboarding.languageDone) return 1;
    if (!settings.onboarding.modelDone) return 2;
    return 3;
  });
  const [themeId, setThemeId] = useState(settings?.themeId ?? "ink");
  const [locale, setLocale] = useState(settings?.locale ?? "zh-CN");
  const [modelChoice, setModelChoice] = useState<"mock" | "later">("mock");

  const mock = useMemo(() => models.find((m) => m.data.provider === "mock"), [models]);

  if (!settings) return null;

  const finish = async (completed: boolean) => {
    const next = {
      ...settings,
      themeId,
      locale: locale as typeof settings.locale,
      onboarding: {
        completed,
        themeDone: true,
        languageDone: true,
        modelDone: true,
      },
      modelAssignments: {
        ...settings.modelAssignments,
        chat:
          modelChoice === "mock" && mock
            ? mock.id
            : settings.modelAssignments.chat ?? mock?.id ?? null,
      },
    };
    await saveSettings(next);
  };

  const canNext = step < 3;

  return (
    <div className="onboarding-backdrop">
      <div className="onboarding-card">
        <div className="onboarding-head">
          <div className="onboarding-brand">
            <span className="brand-mark">
              <span />
              <span />
              <span />
            </span>
            <span>Sporelink</span>
          </div>
          <button type="button" className="text-button" onClick={() => void finish(true)}>
            {translate(locale, "app.skip")}
          </button>
        </div>

        <div className="onboarding-progress">
          {[0, 1, 2, 3].map((i) => (
            <span key={i} className={cn("onboarding-dot", i === step && "is-active", i < step && "is-done")} />
          ))}
        </div>

        {step === 0 && (
          <section className="onboarding-step">
            <h1>{translate(locale, "onboarding.theme.title")}</h1>
            <p>{translate(locale, "onboarding.theme.subtitle")}</p>
            <div className="theme-list">
              {THEMES.map((theme) => (
                <button
                  key={theme.id}
                  type="button"
                  className={cn("theme-option", themeId === theme.id && "is-active")}
                  onClick={() => {
                    setThemeId(theme.id);
                    previewTheme(theme.id);
                  }}
                >
                  <span className="theme-swatch" style={{ background: theme.tokens.bg }}>
                    <span style={{ background: theme.tokens.card }} />
                    <span style={{ background: theme.tokens.brand }} />
                    <span style={{ background: theme.tokens.text }} />
                  </span>
                  <span className="theme-name">
                    {translate(locale, `theme.${theme.id}`)}
                    <small>{theme.name}</small>
                  </span>
                  {themeId === theme.id && <Check size={16} />}
                </button>
              ))}
            </div>
          </section>
        )}

        {step === 1 && (
          <section className="onboarding-step">
            <h1>{translate(locale, "onboarding.language.title")}</h1>
            <p>{translate(locale, "onboarding.language.subtitle")}</p>
            <div className="language-list">
              {LANGS.map((lang) => (
                <button
                  key={lang.id}
                  type="button"
                  className={cn("language-option", locale === lang.id && "is-active")}
                  onClick={() => setLocale(lang.id as "zh-CN" | "zh-TW" | "en")}
                >
                  <Globe2 size={18} />
                  <span>
                    <strong>{lang.label}</strong>
                    <small>{lang.sub}</small>
                  </span>
                  {locale === lang.id && <Check size={16} />}
                </button>
              ))}
            </div>
          </section>
        )}

        {step === 2 && (
          <section className="onboarding-step">
            <h1>{translate(locale, "onboarding.model.title")}</h1>
            <p>{translate(locale, "onboarding.model.subtitle")}</p>
            <div className="model-choice-list">
              <button
                type="button"
                className={cn("model-choice", modelChoice === "mock" && "is-active")}
                onClick={() => setModelChoice("mock")}
                disabled={!mock}
              >
                <span className="model-choice-icon">
                  <Sparkles size={20} />
                </span>
                <span>
                  <strong>{translate(locale, "onboarding.model.mock")}</strong>
                  <small>{translate(locale, "onboarding.model.mockDesc")}</small>
                </span>
                {modelChoice === "mock" && <Check size={16} />}
              </button>
              <button
                type="button"
                className={cn("model-choice", modelChoice === "later" && "is-active")}
                onClick={() => setModelChoice("later")}
              >
                <span className="model-choice-icon">
                  <Plus size={20} />
                </span>
                <span>
                  <strong>{translate(locale, "onboarding.model.later")}</strong>
                  <small>{translate(locale, "onboarding.model.laterDesc")}</small>
                </span>
                {modelChoice === "later" && <Check size={16} />}
              </button>
            </div>
          </section>
        )}

        {step === 3 && (
          <section className="onboarding-step onboarding-welcome">
            <h1>{translate(locale, "onboarding.welcome.title")}</h1>
            <p>{translate(locale, "onboarding.welcome.body")}</p>
            <div className="onboarding-welcome-actions">
              <Button
                variant="primary"
                onClick={() => {
                  void finish(true);
                  void openSample();
                }}
              >
                <Sparkles size={16} /> {translate(locale, "onboarding.welcome.how")}
              </Button>
              <Button
                onClick={() => {
                  void finish(true);
                  setNewProjectOpen(true);
                }}
              >
                <Plus size={16} /> {translate(locale, "onboarding.welcome.new")}
              </Button>
            </div>
            <p className="onboarding-privacy">{translate(locale, "onboarding.welcome.privacy")}</p>
          </section>
        )}

        <div className="onboarding-foot">
          <div className="onboarding-step-label">
            {step < 3
              ? `${translate(locale, "onboarding.step." + ["theme", "language", "model"][step])} · ${step + 1}/3`
              : translate(locale, "onboarding.step.welcome")}
          </div>
          {canNext && (
            <Button variant="primary" onClick={() => setStep((s) => Math.min(3, s + 1))}>
              {translate(locale, "app.next")} <ArrowRight size={16} />
            </Button>
          )}
          {step === 3 && (
            <Button variant="ghost" onClick={() => void finish(true)}>
              {translate(locale, "app.finish")}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
