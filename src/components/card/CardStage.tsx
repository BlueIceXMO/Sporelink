import { useMemo, useState } from "react";
import {
  Check,
  Copy,
  CornerDownRight,
  GitBranch,
  Home,
  Lock,
  MoreHorizontal,
  Pencil,
  Star,
  Trash2,
  Unlock,
  Waypoints,
} from "lucide-react";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { cn, copyText, firstLine } from "@/lib/utils";
import { IconButton } from "@/components/common/IconButton";
import { EmptyState, SurfaceBadge } from "@/components/common/Primitives";
import { TurnList } from "./TurnList";

const SUGGESTIONS = {
  "zh-CN": [
    "用一个具体例子解释这个概念",
    "这个结论有哪些反例或边界条件？",
    "把它拆成三个可以验证的小问题",
  ],
  "zh-TW": [
    "用一個具體例子解釋這個概念",
    "這個結論有哪些反例或邊界條件？",
    "把它拆成三個可以驗證的小問題",
  ],
  en: [
    "Explain this concept with a concrete example",
    "What are the counterexamples or boundary conditions?",
    "Break it into three testable sub-questions",
  ],
};

export function CardStage({
  onQuote,
  onPrompt,
}: {
  onQuote: (text: string) => void;
  onPrompt: (text: string) => void;
}) {
  const { locale, t } = useI18n();
  const card = useAppStore((s) => s.currentCard);
  const project = useAppStore((s) => s.currentProject);
  const createCard = useAppStore((s) => s.createCard);
  const deleteCard = useAppStore((s) => s.deleteCard);
  const favoriteCard = useAppStore((s) => s.favoriteCard);
  const lockCard = useAppStore((s) => s.lockCard);
  const updateCard = useAppStore((s) => s.updateCard);
  const openCard = useAppStore((s) => s.openCard);
  const graph = useAppStore((s) => s.graph);
  const toast = useAppStore((s) => s.toast);

  const [editingTitle, setEditingTitle] = useState(false);
  const [title, setTitle] = useState("");
  const [moreOpen, setMoreOpen] = useState(false);

  const suggestions = SUGGESTIONS[(locale as keyof typeof SUGGESTIONS) in SUGGESTIONS ? (locale as keyof typeof SUGGESTIONS) : "zh-CN"];

  const surfaceLabel = useMemo(() => {
    if (!card) return "";
    return t(`card.${card.card.surfaceKind}`);
  }, [card, t]);

  if (!card) return null;

  const locked = card.card.locked;
  const readOnly = project?.readOnly ?? false;

  const startRename = () => {
    setTitle(card.card.title);
    setEditingTitle(true);
  };

  const saveTitle = async () => {
    if (!title.trim() || title.trim() === card.card.title) {
      setEditingTitle(false);
      return;
    }
    await updateCard({ title: title.trim() }, card.card.revision);
    setEditingTitle(false);
  };

  const copyCard = async () => {
    const lines: string[] = [`# ${card.card.title}`];
    for (const turn of card.turns) {
      for (const message of turn.messages) {
        lines.push("", message.role === "user" ? `**${t("message.user")}**` : `**${t("message.assistant")}**`, "", message.content);
      }
    }
    await copyText(lines.join("\n"));
    toast("success", t("toast.copied"));
  };

  return (
    <section className="card-stage">
      <div className="card-panel">
        <header className="card-panel-head">
          <div className="card-title-wrap">
            <SurfaceBadge surface={card.card.surfaceKind} />
            {editingTitle ? (
              <input
                className="card-title-input"
                value={title}
                autoFocus
                onChange={(e) => setTitle(e.target.value)}
                onBlur={() => void saveTitle()}
                onKeyDown={(e) => {
                  if (e.key === "Enter") void saveTitle();
                  if (e.key === "Escape") setEditingTitle(false);
                }}
              />
            ) : (
              <h2 onDoubleClick={readOnly ? undefined : startRename} title={card.card.title}>
                {card.card.title}
              </h2>
            )}
            {locked && (
              <span className="pill">
                <Lock size={11} /> {t("card.locked")}
              </span>
            )}
          </div>
          <div className="card-tools">
            <IconButton label={t("message.copy")} size="sm" onClick={() => void copyCard()}>
              <Copy size={16} />
            </IconButton>
            <IconButton
              label={card.card.favorite ? t("card.unfavorite") : t("card.favorite")}
              size="sm"
              active={card.card.favorite}
              onClick={() => void favoriteCard(!card.card.favorite)}
            >
              <Star size={16} />
            </IconButton>
            <IconButton
              label={locked ? t("card.unlock") : t("card.lock")}
              size="sm"
              active={locked}
              onClick={() => void lockCard(!locked)}
            >
              {locked ? <Unlock size={16} /> : <Lock size={16} />}
            </IconButton>
            {graph?.rootCardId && graph.rootCardId !== card.card.id && (
              <IconButton label={t("card.rootAction")} size="sm" onClick={() => void openCard(graph.rootCardId!)}>
                <Home size={16} />
              </IconButton>
            )}
            <div className="menu-anchor">
              <IconButton label={t("app.more")} size="sm" onClick={() => setMoreOpen((v) => !v)}>
                <MoreHorizontal size={17} />
              </IconButton>
              {moreOpen && (
                <>
                  <div className="menu-backdrop" onClick={() => setMoreOpen(false)} />
                  <div className="dropdown-menu align-right">
                    <button
                      type="button"
                      onClick={() => {
                        setMoreOpen(false);
                        startRename();
                      }}
                    >
                      <Pencil size={14} /> {t("app.rename")}
                    </button>
                    <button
                      type="button"
                      onClick={() => {
                        setMoreOpen(false);
                        void copyCard();
                      }}
                    >
                      <Copy size={14} /> {t("card.copy")}
                    </button>
                    <button
                      type="button"
                      className="danger"
                      onClick={() => {
                        setMoreOpen(false);
                        if (window.confirm(t("card.deleteConfirm", { name: card.card.title }))) {
                          void deleteCard("tree");
                        }
                      }}
                    >
                      <Trash2 size={14} /> {t("card.delete")}
                    </button>
                    <div className="menu-meta">{surfaceLabel}</div>
                  </div>
                </>
              )}
            </div>
          </div>
        </header>

        {readOnly && <div className="card-banner">{t("card.readOnly")}</div>}
        {locked && <div className="card-banner locked">{t("composer.locked")}</div>}

        {card.turns.length === 0 ? (
          <div className="card-empty">
            <EmptyState
              title={t("card.empty")}
              body={t("card.emptyHint")}
              actions={
                <div className="suggestion-list">
                  {suggestions.map((suggestion) => (
                    <button
                      key={suggestion}
                      type="button"
                      className="suggestion"
                      onClick={() => {
                        onPrompt(suggestion);
                      }}
                    >
                      <CornerDownRight size={14} />
                      {suggestion}
                    </button>
                  ))}
                </div>
              }
            />
            <div className="relation-actions">
              <button type="button" onClick={() => void createCard("dive")}>
                <Waypoints size={15} /> {t("card.newDive")}
              </button>
              <button type="button" onClick={() => void createCard("diverge")}>
                <GitBranch size={15} /> {t("card.newDiverge")}
              </button>
              <button type="button" onClick={() => void createCard("branch")}>
                <GitBranch size={15} /> {t("card.newBranch")}
              </button>
            </div>
          </div>
        ) : (
          <TurnList
            onQuote={onQuote}
            onDive={(messageId) => void createCard("dive", messageId)}
            onDiverge={(messageId) => void createCard("diverge", messageId)}
            onBranch={(messageId) => void createCard("branch", messageId)}
          />
        )}

        {card.turns.length > 0 && (
          <footer className="card-foot">
            <span>
              {t("message.selectHint")} · {t("card.turns")}: {card.turns.length}
            </span>
            <div className="card-foot-actions">
              <button type="button" onClick={() => void createCard("dive")}>
                <Waypoints size={13} /> {t("card.dive")}
              </button>
              <button type="button" onClick={() => void createCard("diverge")}>
                <GitBranch size={13} /> {t("card.diverge")}
              </button>
              <button type="button" onClick={() => void createCard("branch")}>
                <GitBranch size={13} /> {t("card.branch")}
              </button>
            </div>
          </footer>
        )}
      </div>
    </section>
  );
}
