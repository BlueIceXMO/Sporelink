import { useEffect, useRef, useState } from "react";
import {
  BookmarkPlus,
  Check,
  ChevronDown,
  ChevronRight,
  Copy,
  CornerDownRight,
  GitBranch,
  Pencil,
  RefreshCw,
  Star,
  Trash2,
  Waypoints,
} from "lucide-react";
import type { MessageDto, TurnDto } from "@/lib/types";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { cn, copyText, estimateTokens, firstLine, formatCost, formatTokens } from "@/lib/utils";
import { Markdown } from "@/components/common/Markdown";
import { IconButton } from "@/components/common/IconButton";
import { Spinner } from "@/components/common/Primitives";

type MessageProps = {
  message: MessageDto;
  onQuote: (text: string) => void;
  onDive: (messageId: string) => void;
  onDiverge: (messageId: string) => void;
  onBranch: (messageId: string) => void;
  onRegenerate: (messageId: string) => void;
  onDeleteTurn: (turnId: string) => void;
  readOnly: boolean;
};

function SelectionToolbar({
  rect,
  text,
  messageId,
  onClose,
  onQuote,
  onDive,
  onSaveKnowledge,
}: {
  rect: DOMRect;
  text: string;
  messageId: string;
  onClose: () => void;
  onQuote: (text: string) => void;
  onDive: (messageId: string) => void;
  onSaveKnowledge: (messageId: string, text: string) => void;
}) {
  const { t } = useI18n();
  const saveKnowledge = useAppStore((s) => s.saveKnowledgeFromMessage);
  useEffect(() => {
    const onDown = () => onClose();
    window.addEventListener("mousedown", onDown);
    return () => window.removeEventListener("mousedown", onDown);
  }, [onClose]);
  return (
    <div
      className="selection-toolbar"
      style={{ left: Math.max(12, rect.left + rect.width / 2 - 120), top: Math.max(12, rect.top - 48) }}
      onMouseDown={(e) => e.stopPropagation()}
    >
      <button type="button" onClick={() => onQuote(text)}>
        <CornerDownRight size={14} /> {t("message.quote")}
      </button>
      <button type="button" onClick={() => onDive(messageId)}>
        <Waypoints size={14} /> {t("card.dive")}
      </button>
      <button
        type="button"
        onClick={() => {
          void saveKnowledge(messageId, text);
          onSaveKnowledge(messageId, text);
          onClose();
        }}
      >
        <BookmarkPlus size={14} /> {t("message.saveKnowledge")}
      </button>
      <button
        type="button"
        onClick={() => {
          void copyText(text);
          onClose();
        }}
      >
        <Copy size={14} /> {t("message.copy")}
      </button>
    </div>
  );
}

function MessageBubble({
  message,
  onQuote,
  onDive,
  onDiverge,
  onBranch,
  onRegenerate,
  onDeleteTurn,
  readOnly,
}: MessageProps) {
  const { t } = useI18n();
  const [reasoningOpen, setReasoningOpen] = useState(false);
  const [editing, setEditing] = useState(false);
  const [editText, setEditText] = useState(message.content);
  const [selection, setSelection] = useState<{ rect: DOMRect; text: string } | null>(null);
  const [usageOpen, setUsageOpen] = useState(false);
  const bodyRef = useRef<HTMLDivElement>(null);
  const favoriteMessage = useAppStore((s) => s.favoriteMessage);
  const updateMessage = useAppStore((s) => s.updateMessage);
  const toast = useAppStore((s) => s.toast);
  const isUser = message.role === "user";
  const streaming = message.status === "streaming";

  useEffect(() => {
    setEditText(message.content);
  }, [message.content]);

  useEffect(() => {
    const onUp = () => {
      const sel = window.getSelection();
      if (!sel || sel.isCollapsed || !bodyRef.current) {
        setSelection(null);
        return;
      }
      if (!bodyRef.current.contains(sel.anchorNode) || !bodyRef.current.contains(sel.focusNode)) {
        setSelection(null);
        return;
      }
      const text = sel.toString().trim();
      if (!text) {
        setSelection(null);
        return;
      }
      const range = sel.getRangeAt(0);
      setSelection({ rect: range.getBoundingClientRect(), text });
    };
    document.addEventListener("mouseup", onUp);
    return () => document.removeEventListener("mouseup", onUp);
  }, []);

  const saveEdit = async (resend = false) => {
    if (!editText.trim()) return;
    await updateMessage(message.id, { content: editText.trim() }, message.revision);
    setEditing(false);
    if (resend) onRegenerate(message.id);
  };

  return (
    <article className={cn("message", isUser ? "message-user" : "message-assistant")}>
      <div className="message-gutter">
        {isUser ? (
          <span className="message-avatar user">{t("message.user").slice(0, 1)}</span>
        ) : (
          <span className="message-avatar assistant">
            <span />
            <span />
            <span />
          </span>
        )}
      </div>
      <div className="message-main">
        <div className="message-meta">
          <span className="message-role">{isUser ? t("message.user") : t("message.assistant")}</span>
          {streaming && (
            <span className="message-status">
              <Spinner size={12} /> {t("message.streaming")}
            </span>
          )}
          {message.status === "error" && <span className="message-status error">{t("message.error")}</span>}
          {message.status === "cancelled" && <span className="message-status">{t("message.stopped")}</span>}
          {message.favorite && <Star size={12} className="message-fav" />}
        </div>

        {message.reasoning && (
          <div className={cn("reasoning-block", reasoningOpen && "is-open")}>
            <button type="button" className="reasoning-toggle" onClick={() => setReasoningOpen((v) => !v)}>
              {reasoningOpen ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
              {t("message.reasoning")}
            </button>
            {reasoningOpen && <div className="reasoning-body">{message.reasoning}</div>}
          </div>
        )}

        {editing ? (
          <div className="message-edit">
            <textarea
              autoFocus
              value={editText}
              rows={Math.min(12, Math.max(3, editText.split("\n").length + 1))}
              onChange={(e) => setEditText(e.target.value)}
              placeholder={t("message.editPlaceholder")}
            />
            <div className="message-edit-actions">
              <button type="button" className="text-button" onClick={() => setEditing(false)}>
                {t("app.cancel")}
              </button>
              <button type="button" className="text-button" onClick={() => void saveEdit(false)}>
                <Check size={13} /> {t("app.save")}
              </button>
              <button type="button" className="text-button accent" onClick={() => void saveEdit(true)}>
                {t("message.editAndResend")}
              </button>
            </div>
          </div>
        ) : (
          <div ref={bodyRef} className={cn("message-body", isUser && "user-bubble")}>
            {isUser ? <p>{message.content}</p> : <Markdown>{message.content}</Markdown>}
          </div>
        )}

        {message.citations.length > 0 && (
          <div className="citation-row">
            {message.citations.map((citation) => (
              <span key={`${citation.sourceType}:${citation.sourceId}`} className="citation-chip" title={citation.locator ?? undefined}>
                {citation.title}
              </span>
            ))}
          </div>
        )}

        {message.attachments.length > 0 && (
          <div className="attachment-row">
            {message.attachments.map((attachment) => (
              <span key={attachment.id} className="attachment-chip">
                {attachment.fileName}
              </span>
            ))}
          </div>
        )}

        <div className="message-actions">
          <IconButton
            label={t("message.copy")}
            size="sm"
            onClick={() => {
              void copyText(message.content);
              toast("success", t("toast.copied"));
            }}
          >
            <Copy size={13} />
          </IconButton>
          <IconButton
            label={message.favorite ? t("message.unfavorite") : t("message.favorite")}
            size="sm"
            active={message.favorite}
            onClick={() => void favoriteMessage(message.id, !message.favorite)}
          >
            <Star size={13} />
          </IconButton>
          {isUser && !readOnly && (
            <IconButton label={t("message.edit")} size="sm" onClick={() => setEditing(true)}>
              <Pencil size={13} />
            </IconButton>
          )}
          {!isUser && !readOnly && (
            <>
              <IconButton label={t("message.regenerate")} size="sm" onClick={() => onRegenerate(message.id)}>
                <RefreshCw size={13} />
              </IconButton>
              <IconButton label={t("card.newDive")} size="sm" onClick={() => onDive(message.id)}>
                <Waypoints size={13} />
              </IconButton>
              <IconButton label={t("card.newDiverge")} size="sm" onClick={() => onDiverge(message.id)}>
                <GitBranch size={13} />
              </IconButton>
              <IconButton label={t("card.newBranch")} size="sm" onClick={() => onBranch(message.id)}>
                <GitBranch size={13} />
              </IconButton>
            </>
          )}
          <IconButton label={t("turn.delete")} size="sm" onClick={() => onDeleteTurn(message.turnId)}>
            <Trash2 size={13} />
          </IconButton>
          {message.usage && (
            <button type="button" className="usage-button" onClick={() => setUsageOpen((v) => !v)}>
              {formatTokens(message.usage.inputTokens + message.usage.outputTokens)} tok
              {message.usage.cost > 0 ? ` · ${formatCost(message.usage.cost, message.usage.currency)}` : ""}
            </button>
          )}
        </div>
        {usageOpen && message.usage && (
          <div className="usage-detail">
            <span>in {formatTokens(message.usage.inputTokens)}</span>
            <span>out {formatTokens(message.usage.outputTokens)}</span>
            <span>{formatCost(message.usage.cost, message.usage.currency)}</span>
            <span>{message.usage.latencyMs} ms</span>
            {message.usage.estimated && <span>{t("app.estimated")}</span>}
          </div>
        )}
      </div>

      {selection && !isUser && (
        <SelectionToolbar
          rect={selection.rect}
          text={selection.text}
          messageId={message.id}
          onClose={() => setSelection(null)}
          onQuote={onQuote}
          onDive={onDive}
          onSaveKnowledge={() => undefined}
        />
      )}
    </article>
  );
}

function TurnBlock({
  turn,
  onQuote,
  onDive,
  onDiverge,
  onBranch,
  onRegenerate,
  onDeleteTurn,
  readOnly,
}: {
  turn: TurnDto;
  onQuote: (text: string) => void;
  onDive: (messageId: string) => void;
  onDiverge: (messageId: string) => void;
  onBranch: (messageId: string) => void;
  onRegenerate: (messageId: string) => void;
  onDeleteTurn: (turnId: string) => void;
  readOnly: boolean;
}) {
  const { t } = useI18n();
  const toggleTurn = useAppStore((s) => s.toggleTurn);
  const deleteTurn = useAppStore((s) => s.deleteTurn);
  const firstUser = turn.messages.find((m) => m.role === "user");
  const preview = firstUser ? firstLine(firstUser.content, 90) : t("turn.index", { index: turn.index + 1 });

  return (
    <section className={cn("turn", turn.collapsed && "is-collapsed")}>
      <header className="turn-head">
        <button type="button" className="turn-toggle" onClick={() => void toggleTurn(turn.id, !turn.collapsed)}>
          {turn.collapsed ? <ChevronRight size={15} /> : <ChevronDown size={15} />}
          <span className="turn-index">{t("turn.index", { index: turn.index + 1 })}</span>
          <span className="turn-preview">{preview}</span>
        </button>
        <div className="turn-actions">
          <IconButton
            label={t("turn.delete")}
            size="sm"
            onClick={() => {
              if (window.confirm(t("turn.deleteConfirm"))) void deleteTurn(turn.id);
            }}
          >
            <Trash2 size={13} />
          </IconButton>
        </div>
      </header>
      {!turn.collapsed && (
        <div className="turn-body">
          {turn.messages.map((message) => (
            <MessageBubble
              key={message.id}
              message={message}
              onQuote={onQuote}
              onDive={onDive}
              onDiverge={onDiverge}
              onBranch={onBranch}
              onRegenerate={onRegenerate}
              onDeleteTurn={onDeleteTurn}
              readOnly={readOnly}
            />
          ))}
        </div>
      )}
    </section>
  );
}

export function TurnList({
  onQuote,
  onDive,
  onDiverge,
  onBranch,
}: {
  onQuote: (text: string) => void;
  onDive: (messageId: string) => void;
  onDiverge: (messageId: string) => void;
  onBranch: (messageId: string) => void;
}) {
  const card = useAppStore((s) => s.currentCard);
  const project = useAppStore((s) => s.currentProject);
  const regenerate = useAppStore((s) => s.regenerate);
  const deleteTurn = useAppStore((s) => s.deleteTurn);
  const readOnly = project?.readOnly ?? false;
  if (!card) return null;
  return (
    <div className="turn-list">
      {card.turns.map((turn) => (
        <TurnBlock
          key={turn.id}
          turn={turn}
          onQuote={onQuote}
          onDive={onDive}
          onDiverge={onDiverge}
          onBranch={onBranch}
          onRegenerate={(messageId) => void regenerate(messageId)}
          onDeleteTurn={(turnId) => void deleteTurn(turnId)}
          readOnly={readOnly}
        />
      ))}
    </div>
  );
}
