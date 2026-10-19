import { useEffect, useMemo, useState } from "react";
import {
  BookOpen,
  Check,
  Link2,
  Plus,
  Search,
  Sparkles,
  Trash2,
  X,
} from "lucide-react";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import type { KnowledgeDto } from "@/lib/types";
import { Button, IconButton } from "@/components/common/IconButton";
import { EmptyState } from "@/components/common/Primitives";
import { Modal } from "@/components/common/Modal";
import { Field } from "@/components/common/Primitives";
import { cn, splitTags } from "@/lib/utils";

type EditorState = {
  open: boolean;
  editing: KnowledgeDto | null;
  title: string;
  body: string;
  category: string;
  confidence: number;
  tags: string;
};

function emptyEditor(): EditorState {
  return {
    open: false,
    editing: null,
    title: "",
    body: "",
    category: "insight",
    confidence: 0.7,
    tags: "",
  };
}

export function KnowledgeView() {
  const { t } = useI18n();
  const knowledge = useAppStore((s) => s.knowledge);
  const currentProject = useAppStore((s) => s.currentProject);
  const loadKnowledge = useAppStore((s) => s.loadKnowledge);
  const createKnowledge = useAppStore((s) => s.createKnowledge);
  const updateKnowledge = useAppStore((s) => s.updateKnowledge);
  const deleteKnowledge = useAppStore((s) => s.deleteKnowledge);
  const checkKnowledge = useAppStore((s) => s.checkKnowledge);
  const toast = useAppStore((s) => s.toast);
  const contextIds = useAppStore((s) => s.contextOptions.knowledgeIds);
  const toggleKnowledgeContext = useAppStore((s) => s.toggleKnowledgeContext);
  const [query, setQuery] = useState("");
  const [editor, setEditor] = useState<EditorState>(emptyEditor);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [checking, setChecking] = useState(false);

  useEffect(() => {
    void loadKnowledge(currentProject?.project.id ?? null);
  }, [loadKnowledge, currentProject?.project.id]);

  const filtered = useMemo(() => {
    if (!query.trim()) return knowledge;
    const q = query.toLowerCase();
    return knowledge.filter(
      (item) =>
        item.title.toLowerCase().includes(q) ||
        item.body.toLowerCase().includes(q) ||
        item.tags.some((tag) => tag.toLowerCase().includes(q)),
    );
  }, [knowledge, query]);

  const openNew = () => {
    setEditor({ ...emptyEditor(), open: true });
    setFeedback(null);
  };

  const openEdit = (item: KnowledgeDto) => {
    setEditor({
      open: true,
      editing: item,
      title: item.title,
      body: item.body,
      category: item.category,
      confidence: item.confidence,
      tags: item.tags.join(", "),
    });
    setFeedback(null);
  };

  const save = async () => {
    if (!editor.title.trim() || !editor.body.trim()) return;
    const tags = splitTags(editor.tags);
    if (editor.editing) {
      await updateKnowledge(
        editor.editing.id,
        {
          title: editor.title.trim(),
          body: editor.body,
          category: editor.category,
          confidence: editor.confidence,
          tags,
        },
        editor.editing.revision,
      );
    } else {
      await createKnowledge({
        projectId: currentProject?.project.id ?? null,
        title: editor.title.trim(),
        body: editor.body,
        category: editor.category,
        confidence: editor.confidence,
        tags,
      });
    }
    setEditor(emptyEditor());
  };

  const check = async () => {
    if (!editor.editing) {
      toast("info", t("knowledge.check"), t("app.save"));
      return;
    }
    setChecking(true);
    const result = await checkKnowledge(editor.editing.id);
    setFeedback(result);
    setChecking(false);
  };

  return (
    <div className="knowledge-view">
      <header className="knowledge-head">
        <div>
          <h1>{t("knowledge.title")}</h1>
          <p>{t("knowledge.subtitle")}</p>
        </div>
        <div className="knowledge-head-actions">
          <div className="reader-search">
            <Search size={14} />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder={t("knowledge.search")}
            />
          </div>
          <Button variant="primary" onClick={openNew}>
            <Plus size={16} /> {t("knowledge.new")}
          </Button>
        </div>
      </header>

      {filtered.length === 0 ? (
        <EmptyState
          icon={<BookOpen size={28} />}
          title={t("knowledge.empty")}
          body={t("knowledge.emptyHint")}
          actions={
            <Button variant="primary" onClick={openNew}>
              <Plus size={16} /> {t("knowledge.new")}
            </Button>
          }
        />
      ) : (
        <div className="knowledge-grid">
          {filtered.map((item) => (
            <article key={item.id} className="knowledge-card">
              <header>
                <span className="knowledge-category">{t(`knowledge.category.${item.category}`) || item.category}</span>
                <span className="knowledge-confidence">{Math.round(item.confidence * 100)}%</span>
              </header>
              <h2 onClick={() => openEdit(item)}>{item.title}</h2>
              <p>{item.body}</p>
              {item.tags.length > 0 && (
                <div className="tag-row">
                  {item.tags.map((tag) => (
                    <span key={tag} className="tag">
                      {tag}
                    </span>
                  ))}
                </div>
              )}
              <footer>
                <span>
                  <Link2 size={12} /> {item.sourceRefs[0]?.title ?? t("knowledge.noSource")}
                </span>
                <div className="knowledge-card-actions">
                  <button
                    type="button"
                    className={cn(contextIds.includes(item.id) && "is-active")}
                    onClick={() => toggleKnowledgeContext(item.id)}
                  >
                    <Sparkles size={13} />{" "}
                    {contextIds.includes(item.id) ? t("knowledge.removeFromContext") : t("knowledge.addToContext")}
                  </button>
                  <IconButton
                    label={t("app.delete")}
                    size="sm"
                    onClick={() => {
                      if (window.confirm(t("knowledge.deleteConfirm", { title: item.title }))) {
                        void deleteKnowledge(item.id);
                      }
                    }}
                  >
                    <Trash2 size={13} />
                  </IconButton>
                </div>
              </footer>
            </article>
          ))}
        </div>
      )}

      <Modal
        open={editor.open}
        onClose={() => setEditor(emptyEditor())}
        title={editor.editing ? t("knowledge.title") : t("knowledge.new")}
        subtitle={t("knowledge.subtitle")}
        width="lg"
        footer={
          <>
            <Button variant="ghost" onClick={() => setEditor(emptyEditor())}>
              {t("app.cancel")}
            </Button>
            <Button onClick={() => void check()} disabled={!editor.editing || checking}>
              <Sparkles size={15} /> {checking ? t("knowledge.checking") : t("knowledge.check")}
            </Button>
            <Button variant="primary" onClick={() => void save()} disabled={!editor.title.trim() || !editor.body.trim()}>
              <Check size={15} /> {t("knowledge.save")}
            </Button>
          </>
        }
      >
        <div className="knowledge-editor">
          <Field label={t("knowledge.titleField")}>
            <input value={editor.title} onChange={(e) => setEditor({ ...editor, title: e.target.value })} />
          </Field>
          <Field label={t("knowledge.body")}>
            <textarea
              rows={8}
              value={editor.body}
              onChange={(e) => setEditor({ ...editor, body: e.target.value })}
              placeholder={t("knowledge.bodyPlaceholder")}
            />
          </Field>
          <div className="field-row">
            <Field label={t("knowledge.category")}>
              <select
                value={editor.category}
                onChange={(e) => setEditor({ ...editor, category: e.target.value })}
              >
                <option value="insight">{t("knowledge.category.insight")}</option>
                <option value="summary">{t("knowledge.category.summary")}</option>
                <option value="question">{t("knowledge.category.question")}</option>
                <option value="reference">{t("knowledge.category.reference")}</option>
              </select>
            </Field>
            <Field label={`${t("knowledge.confidence")}: ${Math.round(editor.confidence * 100)}%`}>
              <input
                type="range"
                min={0}
                max={1}
                step={0.05}
                value={editor.confidence}
                onChange={(e) => setEditor({ ...editor, confidence: Number(e.target.value) })}
              />
            </Field>
          </div>
          <Field label={t("knowledge.tags")}>
            <input
              value={editor.tags}
              onChange={(e) => setEditor({ ...editor, tags: e.target.value })}
              placeholder={t("knowledge.tagsPlaceholder")}
            />
          </Field>
          {editor.editing?.sourceRefs.length ? (
            <div className="source-refs">
              <strong>{t("knowledge.source")}</strong>
              {editor.editing.sourceRefs.map((ref) => (
                <span key={`${ref.sourceType}:${ref.sourceId}`}>
                  {ref.title}
                  {ref.locator ? ` · ${ref.locator}` : ""}
                </span>
              ))}
            </div>
          ) : null}
          {feedback && (
            <div className="knowledge-feedback">
              <strong>{t("knowledge.feedback")}</strong>
              <p>{feedback}</p>
              <IconButton label={t("app.close")} size="sm" onClick={() => setFeedback(null)}>
                <X size={14} />
              </IconButton>
            </div>
          )}
        </div>
      </Modal>
    </div>
  );
}
