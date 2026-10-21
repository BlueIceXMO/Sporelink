import { useEffect } from "react";
import { Eye, EyeOff, Pin, PinOff, X } from "lucide-react";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { formatTokens } from "@/lib/utils";
import { IconButton } from "@/components/common/IconButton";
import { Spinner } from "@/components/common/Primitives";

const KIND_KEY: Record<string, string> = {
  system: "context.system",
  path: "context.path",
  pinned: "context.pinned",
  citation: "context.citation",
  knowledge: "context.knowledge",
  document: "context.document",
};

export function ContextDrawer() {
  const { t } = useI18n();
  const open = useAppStore((s) => s.contextOpen);
  const setOpen = useAppStore((s) => s.closeContext);
  const preview = useAppStore((s) => s.contextPreview);
  const options = useAppStore((s) => s.contextOptions);
  const toggleContextItem = useAppStore((s) => s.toggleContextItem);
  const setContextPinned = useAppStore((s) => s.setContextPinned);

  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpen();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, setOpen]);

  if (!open) return null;

  const used = preview?.totalTokens ?? 0;
  const budget = preview?.budget ?? 0;
  const percent = budget > 0 ? Math.min(100, Math.round((used / budget) * 100)) : 0;

  return (
    <div className="context-backdrop" onClick={() => setOpen()}>
      <aside className="context-drawer" onClick={(e) => e.stopPropagation()}>
        <header className="context-head">
          <div>
            <h2>{t("context.title")}</h2>
            <p>{t("context.subtitle")}</p>
          </div>
          <IconButton label={t("app.close")} size="sm" onClick={() => setOpen()}>
            <X size={18} />
          </IconButton>
        </header>

        <div className="context-budget">
          <div className="context-budget-row">
            <span>{t("context.used")}</span>
            <span>
              {formatTokens(used)} / {formatTokens(budget)}
            </span>
          </div>
          <div className="budget-bar">
            <span style={{ width: `${percent}%` }} />
          </div>
          {preview && preview.droppedCount > 0 && (
            <div className="context-dropped">{t("context.dropped", { count: preview.droppedCount })}</div>
          )}
        </div>

        {!preview ? (
          <div className="context-loading">
            <Spinner /> {t("app.loading")}
          </div>
        ) : preview.items.length === 0 ? (
          <div className="context-empty">{t("context.noItems")}</div>
        ) : (
          <div className="context-items">
            {preview.items.map((item) => {
              const excluded = options.excludedItemIds.includes(item.id);
              const pinned = item.kind === "pinned" || options.pinnedMessageIds.includes(item.sourceId);
              return (
                <div key={item.id} className={`context-item ${excluded ? "is-excluded" : ""}`}>
                  <div className="context-item-main">
                    <div className="context-item-top">
                      <span className="context-kind">{t(KIND_KEY[item.kind] ?? "context.path")}</span>
                      <span className="context-tokens">{formatTokens(item.tokens)} tok</span>
                    </div>
                    <div className="context-title">{item.title}</div>
                    {item.path && <code className="context-path">{item.path}</code>}
                    <div className="context-reason">{item.reason}</div>
                    {item.droppedReason && <div className="context-dropped-reason">{item.droppedReason}</div>}
                  </div>
                  <div className="context-item-actions">
                    <IconButton
                      label={excluded ? t("context.toggle") : t("context.toggle")}
                      size="sm"
                      onClick={() => toggleContextItem(item.id)}
                    >
                      {excluded ? <EyeOff size={14} /> : <Eye size={14} />}
                    </IconButton>
                    {item.kind !== "system" && (
                      <IconButton
                        label={pinned ? t("context.unpin") : t("context.pin")}
                        size="sm"
                        active={pinned}
                        onClick={() => setContextPinned(item.sourceId, !pinned)}
                      >
                        {pinned ? <PinOff size={14} /> : <Pin size={14} />}
                      </IconButton>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </aside>
    </div>
  );
}
