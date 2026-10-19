import { useEffect, useMemo, useState } from "react";import {
  Archive,
  BookOpen,
  ChevronDown,
  Copy,
  Download,
  FileDown,
  MoreHorizontal,
  Pencil,
  Sparkles,
  Trash2,
} from "lucide-react";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { cn, relativeTime } from "@/lib/utils";
import { ipc } from "@/lib/ipc";
import { Button, IconButton } from "@/components/common/IconButton";
import { CardStage } from "@/components/card/CardStage";
import { Composer } from "@/components/composer/Composer";

export function ProjectView() {
  const { locale, t } = useI18n();
  const project = useAppStore((s) => s.currentProject);
  const currentCard = useAppStore((s) => s.currentCard);
  const graph = useAppStore((s) => s.graph);
  const streaming = useAppStore((s) => s.streaming);
  const sendMessage = useAppStore((s) => s.sendMessage);
  const cancelStream = useAppStore((s) => s.cancelStream);
  const openContext = useAppStore((s) => s.openContext);
  const setView = useAppStore((s) => s.setView);
  const loadKnowledge = useAppStore((s) => s.loadKnowledge);
  const toast = useAppStore((s) => s.toast);
  const exportProject = useAppStore((s) => s.exportProject);
  const duplicateProject = useAppStore((s) => s.duplicateProject);
  const archiveProject = useAppStore((s) => s.archiveProject);
  const deleteProject = useAppStore((s) => s.deleteProject);
  const updateProject = useAppStore((s) => s.updateProject);

  const [draft, setDraft] = useState("");
  const [menuOpen, setMenuOpen] = useState(false);
  const [summarizing, setSummarizing] = useState(false);
  const [renaming, setRenaming] = useState(false);
  const [name, setName] = useState(project?.project.name ?? "");
  const draftToInsert = useAppStore((s) => s.draftToInsert);
  const consumeDraft = useAppStore((s) => s.consumeDraft);

  useEffect(() => {
    if (!draftToInsert) return;
    setDraft((prev) => `${draftToInsert}${prev}`);
    consumeDraft();
  }, [draftToInsert, consumeDraft]);

  const projectData = project?.project;
  const readOnly = project?.readOnly ?? false;

  const send = async () => {
    if (!draft.trim()) return;
    const ok = await sendMessage(draft);
    if (ok) setDraft("");
  };

  const onQuote = (text: string) => {
    const quoted = text
      .split(/\r?\n/)
      .map((line) => `> ${line}`)
      .join("\n");
    setDraft((prev) => `${quoted}\n\n${prev}`);
  };

  const onSummary = async () => {
    if (!projectData) return;
    setSummarizing(true);
    try {
      const modelId = useAppStore.getState().composerModelId ?? undefined;
      const knowledge = await ipc.projectSummary(projectData.id, modelId);
      await loadKnowledge(projectData.id);
      setView("knowledge");
      toast("success", locale === "en" ? "Summary saved to knowledge" : "摘要已保存到知识");
      void knowledge;
    } catch (error) {
      const e = error as { message?: string; hint?: string };
      toast("error", e.message ?? "Summary failed", e.hint);
    } finally {
      setSummarizing(false);
    }
  };

  const saveName = async () => {
    if (!projectData || !name.trim() || name.trim() === projectData.name) {
      setRenaming(false);
      return;
    }
    await updateProject(projectData.id, { name: name.trim() }, projectData.revision);
    setRenaming(false);
  };

  const breadcrumb = useMemo(() => currentCard?.path ?? [], [currentCard]);

  if (!projectData) return null;

  return (
    <div className="project-view">
      <header className="project-head">
        <div className="project-head-main">
          <div className="project-title-row">
            <span className="project-cover" style={{ background: projectData.color || "var(--brand)" }} />
            {renaming ? (
              <input
                className="project-title-input"
                value={name}
                autoFocus
                onChange={(e) => setName(e.target.value)}
                onBlur={() => void saveName()}
                onKeyDown={(e) => {
                  if (e.key === "Enter") void saveName();
                  if (e.key === "Escape") {
                    setName(projectData.name);
                    setRenaming(false);
                  }
                }}
              />
            ) : (
              <h1 onDoubleClick={() => setRenaming(true)} title={projectData.name}>
                {projectData.name}
              </h1>
            )}
            {readOnly && <span className="pill">{t("project.readOnly")}</span>}
          </div>
          <div className="project-sub">
            <span>{t("project.cards", { count: projectData.cardCount })}</span>
            <span>·</span>
            <span>{t("project.messages", { count: projectData.messageCount })}</span>
            <span>·</span>
            <span>{relativeTime(projectData.updatedAt, locale)}</span>
            {projectData.description && (
              <>
                <span>·</span>
                <span className="project-desc">{projectData.description}</span>
              </>
            )}
          </div>
          {breadcrumb.length > 0 && (
            <div className="breadcrumb">
              {breadcrumb.map((item, index) => (
                <span key={item.id} className={cn(index === breadcrumb.length - 1 && "is-current")}>
                  {index > 0 && <ChevronDown size={12} className="breadcrumb-sep" />}
                  {item.title}
                </span>
              ))}
            </div>
          )}
        </div>
        <div className="project-head-actions">
          <Button variant="ghost" size="sm" onClick={() => void onSummary()} disabled={summarizing}>
            <Sparkles size={15} /> {summarizing ? "…" : t("project.summary")}
          </Button>
          <IconButton label={t("card.focus")} onClick={() => setRenaming(true)}>
            <Pencil size={16} />
          </IconButton>
          <div className="menu-anchor">
            <IconButton label={t("app.more")} onClick={() => setMenuOpen((v) => !v)}>
              <MoreHorizontal size={18} />
            </IconButton>
            {menuOpen && (
              <>
                <div className="menu-backdrop" onClick={() => setMenuOpen(false)} />
                <div className="dropdown-menu">
                  <button
                    type="button"
                    onClick={() => {
                      setMenuOpen(false);
                      void exportProject(projectData.id, "json");
                    }}
                  >
                    <Download size={15} /> {t("project.exportJson")}
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      setMenuOpen(false);
                      void exportProject(projectData.id, "markdown");
                    }}
                  >
                    <FileDown size={15} /> {t("project.exportMarkdown")}
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      setMenuOpen(false);
                      void duplicateProject(projectData.id);
                    }}
                  >
                    <Copy size={15} /> {t("project.duplicate")}
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      setMenuOpen(false);
                      void archiveProject(projectData.id, !projectData.archived);
                    }}
                  >
                    <Archive size={15} /> {projectData.archived ? t("project.unarchive") : t("project.archive")}
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      setMenuOpen(false);
                      void loadKnowledge(projectData.id);
                      setView("knowledge");
                    }}
                  >
                    <BookOpen size={15} /> {t("knowledge.title")}
                  </button>
                  <button
                    type="button"
                    className="danger"
                    onClick={() => {
                      setMenuOpen(false);
                      if (window.confirm(t("project.deleteConfirm", { name: projectData.name }))) {
                        void deleteProject(projectData.id);
                      }
                    }}
                  >
                    <Trash2 size={15} /> {t("project.delete")}
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
      </header>

      <div className="project-body">
        <CardStage
          onQuote={onQuote}
          onPrompt={(text) => setDraft(text)}
        />
      </div>

      <div className="composer-dock">
        <Composer
          draft={draft}
          setDraft={setDraft}
          onSend={send}
          onContext={() => void openContext(draft)}
          streaming={Boolean(streaming?.active)}
          onStop={() => void cancelStream()}
          disabled={readOnly || Boolean(currentCard?.card.locked)}
          graphCardCount={graph?.cards.length ?? 0}
        />
      </div>
    </div>
  );
}
