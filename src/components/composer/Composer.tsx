import React, { useEffect, useMemo, useRef } from "react";
import {
  ArrowUp,
  BrainCircuit,
  Paperclip,
  Settings2,
  Square,
  Waypoints,
} from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { cn, estimateTokens, formatTokens } from "@/lib/utils";
import { IconButton } from "@/components/common/IconButton";

export function Composer({
  draft,
  setDraft,
  onSend,
  onContext,
  streaming,
  onStop,
  disabled,
  graphCardCount,
}: {
  draft: string;
  setDraft: (value: string) => void;
  onSend: () => void;
  onContext: () => void;
  streaming: boolean;
  onStop: () => void;
  disabled: boolean;
  graphCardCount: number;
}) {
  const { t } = useI18n();
  const models = useAppStore((s) => s.models);
  const settings = useAppStore((s) => s.settings);
  const composerModelId = useAppStore((s) => s.composerModelId);
  const setComposerModelId = useAppStore((s) => s.setComposerModelId);
  const setSettingsOpen = useAppStore((s) => s.setSettingsOpen);
  const importDocument = useAppStore((s) => s.importDocument);
  const currentProject = useAppStore((s) => s.currentProject);
  const currentCard = useAppStore((s) => s.currentCard);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const model = useMemo(
    () => models.find((m) => m.id === composerModelId) ?? models.find((m) => m.data.provider === "mock") ?? models[0],
    [models, composerModelId],
  );

  const budget = model ? Math.max(1024, model.data.contextWindow - model.data.maxOutput) : 8192;
  const draftTokens = estimateTokens(draft) + (graphCardCount > 0 ? 900 : 0);

  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = "0px";
    el.style.height = `${Math.min(220, Math.max(24, el.scrollHeight))}px`;
  }, [draft]);

  const canSend = Boolean(draft.trim()) && !disabled && !streaming && Boolean(model);

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      if (canSend) onSend();
    }
  };

  const attach = async () => {
    const paths = await openDialog({
      multiple: true,
      filters: [
        { name: "Documents", extensions: ["pdf", "docx", "pptx", "md", "txt", "html", "json", "csv", "rs", "ts", "tsx", "py", "js"] },
      ],
    });
    if (!paths) return;
    const list = Array.isArray(paths) ? paths : [paths];
    for (const path of list) {
      await importDocument(path);
    }
    textareaRef.current?.focus();
  };

  return (
    <div className={cn("composer", disabled && "is-disabled")}>
      <div className="composer-topline">
        <div className="composer-chip-row">
          {model ? (
            <label className="model-picker" title={t("composer.model")}>
              <BrainCircuit size={14} />
              <select value={model.id} onChange={(e) => setComposerModelId(e.target.value)}>
                {models.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.data.label} · {m.data.model}
                  </option>
                ))}
              </select>
            </label>
          ) : (
            <button type="button" className="model-picker missing" onClick={() => setSettingsOpen(true)}>
              <Settings2 size={14} /> {t("composer.noModel")}
            </button>
          )}
          <button type="button" className="composer-chip" onClick={onContext} title={t("composer.context")}>
            <Waypoints size={14} /> {t("composer.context")}
            <span className="chip-count">{formatTokens(draftTokens)}</span>
          </button>
          <span className="composer-budget">
            {formatTokens(draftTokens)} / {formatTokens(budget)}
          </span>
        </div>
        {currentCard && (
          <span className="composer-card-hint">
            {t("card.turns")} · {currentCard.turns.length}
          </span>
        )}
      </div>

      <div className="composer-input-row">
        <textarea
          ref={textareaRef}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={onKeyDown}
          placeholder={disabled ? t("composer.readOnly") : t("composer.placeholder")}
          rows={1}
          disabled={disabled}
        />
        <div className="composer-actions">
          <IconButton label={t("composer.attach")} size="sm" onClick={() => void attach()} disabled={disabled}>
            <Paperclip size={17} />
          </IconButton>
          {streaming ? (
            <button type="button" className="send-button stop" onClick={onStop} title={t("composer.stop")}>
              <Square size={16} />
            </button>
          ) : (
            <button
              type="button"
              className="send-button"
              onClick={onSend}
              disabled={!canSend}
              title={t("composer.send")}
            >
              <ArrowUp size={18} />
            </button>
          )}
        </div>
      </div>

      {!model && (
        <button type="button" className="composer-warning" onClick={() => setSettingsOpen(true)}>
          {t("composer.emptyModel")}
        </button>
      )}
      {currentProject?.readOnly && <div className="composer-warning">{t("composer.readOnly")}</div>}
    </div>
  );
}
