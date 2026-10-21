import { useEffect, useMemo, useRef, useState } from "react";
import {
  BookOpen,
  FileText,
  Link2,
  Plus,
  Search,
  Trash2,
  Upload,
} from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { cn, copyText, formatDate } from "@/lib/utils";
import { Button, IconButton } from "@/components/common/IconButton";
import { EmptyState, Spinner } from "@/components/common/Primitives";
import type { ChunkDto } from "@/lib/types";

export function DocumentView() {
  const { locale, t } = useI18n();
  const documents = useAppStore((s) => s.documents);
  const currentDocument = useAppStore((s) => s.currentDocument);
  const loadDocuments = useAppStore((s) => s.loadDocuments);
  const openDocument = useAppStore((s) => s.openDocument);
  const importDocument = useAppStore((s) => s.importDocument);
  const importUrl = useAppStore((s) => s.importUrl);
  const deleteDocument = useAppStore((s) => s.deleteDocument);
  const busy = useAppStore((s) => s.busy);
  const currentProject = useAppStore((s) => s.currentProject);
  const currentCard = useAppStore((s) => s.currentCard);
  const loadKnowledge = useAppStore((s) => s.loadKnowledge);
  const toast = useAppStore((s) => s.toast);
  const [query, setQuery] = useState("");
  const [activeChunk, setActiveChunk] = useState<string | null>(null);
  const [urlOpen, setUrlOpen] = useState(false);
  const [url, setUrl] = useState("");
  const chunkRefs = useRef<Record<string, HTMLDivElement | null>>({});
  const insertDraft = useAppStore((s) => s.insertDraft);
  const setView = useAppStore((s) => s.setView);

  useEffect(() => {
    void loadDocuments(currentProject?.project.id);
  }, [loadDocuments, currentProject?.project.id]);

  const chunks = currentDocument?.chunks ?? [];
  const filtered = useMemo(() => {
    if (!query.trim()) return chunks;
    const q = query.toLowerCase();
    return chunks.filter((chunk) => chunk.text.toLowerCase().includes(q));
  }, [chunks, query]);

  const chooseFile = async () => {
    const paths = await openDialog({
      multiple: true,
      filters: [
        { name: "Documents", extensions: ["pdf", "docx", "pptx", "md", "txt", "html", "json"] },
      ],
    });
    if (!paths) return;
    const list = Array.isArray(paths) ? paths : [paths];
    for (const path of list) await importDocument(path);
  };

  const quote = (text: string) => {
    const quoted = text
      .split(/\r?\n/)
      .map((line) => `> ${line}`)
      .join("\n");
    insertDraft(`${quoted}\n\n`);
    if (currentCard) {
      setView("project");
      toast("success", t("documents.quote"));
    } else {
      toast("info", t("documents.quote"), t("welcome.subtitle"));
    }
  };

  const saveChunkKnowledge = async (chunk: ChunkDto) => {
    await useAppStore.getState().createKnowledge({
      projectId: currentProject?.project.id ?? null,
      title: chunk.heading ?? currentDocument?.document.title ?? t("knowledge.title"),
      body: chunk.text,
      category: "reference",
      confidence: 0.7,
      tags: [],
      sourceRefs: currentDocument
        ? [
            {
              sourceType: "document",
              sourceId: currentDocument.document.id,
              title: currentDocument.document.title,
              locator: chunk.page ? String(chunk.page) : undefined,
            },
          ]
        : [],
    });
    await loadKnowledge();
  };

  return (
    <div className="documents-view">
      <aside className="documents-list-panel">
        <div className="documents-list-head">
          <h1>{t("documents.title")}</h1>
          <div className="documents-list-actions">
            <IconButton label={t("documents.importUrl")} size="sm" onClick={() => setUrlOpen(true)}>
              <Link2 size={16} />
            </IconButton>
            <IconButton label={t("documents.import")} size="sm" onClick={() => void chooseFile()}>
              <Plus size={16} />
            </IconButton>
          </div>
        </div>
        <button type="button" className="documents-import" onClick={() => void chooseFile()}>
          <Upload size={16} />
          <span>{t("documents.import")}</span>
        </button>
        <div className="documents-list">
          {documents.length === 0 && <div className="sidebar-empty">{t("documents.emptyHint")}</div>}
          {documents.map((doc) => (
            <button
              key={doc.id}
              type="button"
              className={cn("document-row", currentDocument?.document.id === doc.id && "is-active")}
              onClick={() => void openDocument(doc.id)}
            >
              <FileText size={16} />
              <span className="document-row-main">
                <strong>{doc.title}</strong>
                <small>
                  {doc.path || doc.fileName} ·{" "}
                  {doc.pages > 0 ? t("documents.pages", { count: doc.pages }) : t("documents.chunks", { count: doc.chunkCount })}
                </small>
              </span>
            </button>
          ))}
        </div>
      </aside>

      <section className="document-reader">
        {busy.document && (
          <div className="documents-loading">
            <Spinner /> {t("documents.importing")}
          </div>
        )}
        {!currentDocument ? (
          <EmptyState
            icon={<BookOpen size={28} />}
            title={t("documents.empty")}
            body={t("documents.emptyHint")}
            actions={
              <Button variant="primary" onClick={() => void chooseFile()}>
                <Upload size={16} /> {t("documents.import")}
              </Button>
            }
          />
        ) : (
          <>
            <header className="reader-head">
              <div>
                <h1>{currentDocument.document.title}</h1>
                <p>
                  {currentDocument.document.path || currentDocument.document.fileName} ·{" "}
                  {currentDocument.document.pages > 0
                    ? t("documents.pages", { count: currentDocument.document.pages })
                    : t("documents.chunks", { count: currentDocument.document.chunkCount })}{" "}
                  · {formatDate(currentDocument.document.updatedAt, locale)}
                </p>
              </div>
              <div className="reader-head-actions">
                <div className="reader-search">
                  <Search size={14} />
                  <input
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    placeholder={t("documents.search")}
                  />
                </div>
                <IconButton
                  label={t("documents.delete")}
                  size="sm"
                  onClick={() => {
                    if (window.confirm(t("documents.deleteConfirm", { title: currentDocument.document.title }))) {
                      void deleteDocument(currentDocument.document.id);
                    }
                  }}
                >
                  <Trash2 size={16} />
                </IconButton>
              </div>
            </header>

            <div className="reader-body">
              <aside className="reader-toc">
                <div className="reader-toc-title">{t("documents.toc")}</div>
                {filtered.map((chunk) => (
                  <button
                    key={chunk.id}
                    type="button"
                    className={cn("toc-row", activeChunk === chunk.id && "is-active")}
                    onClick={() => {
                      setActiveChunk(chunk.id);
                      chunkRefs.current[chunk.id]?.scrollIntoView({ behavior: "smooth", block: "start" });
                    }}
                  >
                    <span className="toc-index">{chunk.index + 1}</span>
                    <span>{chunk.heading ?? (chunk.page ? t("documents.page", { page: chunk.page }) : `#${chunk.index + 1}`)}</span>
                  </button>
                ))}
              </aside>
              <div className="reader-scroll">
                {filtered.length === 0 && <div className="settings-empty">{t("documents.noChunks")}</div>}
                {filtered.map((chunk) => (
                  <div
                    key={chunk.id}
                    ref={(node) => {
                      chunkRefs.current[chunk.id] = node;
                    }}
                    className="reader-chunk"
                  >
                    <div className="reader-chunk-head">
                      <span>{chunk.heading ?? (chunk.page ? t("documents.page", { page: chunk.page }) : `#${chunk.index + 1}`)}</span>
                      <div className="reader-chunk-actions">
                        <button type="button" onClick={() => void copyText(chunk.text)}>
                          {t("app.copy")}
                        </button>
                        <button type="button" onClick={() => quote(chunk.text)}>
                          {t("documents.quote")}
                        </button>
                        <button type="button" onClick={() => void saveChunkKnowledge(chunk)}>
                          {t("documents.saveKnowledge")}
                        </button>
                      </div>
                    </div>
                    <p>{chunk.text}</p>
                  </div>
                ))}
              </div>
            </div>
          </>
        )}
      </section>

      {urlOpen && (
        <div className="modal-backdrop" onMouseDown={(e) => e.target === e.currentTarget && setUrlOpen(false)}>
          <div className="modal modal-sm">
            <div className="modal-head">
              <h2 className="modal-title">{t("documents.importUrl")}</h2>
              <IconButton label={t("app.close")} size="sm" onClick={() => setUrlOpen(false)}>
                <Trash2 size={16} />
              </IconButton>
            </div>
            <div className="modal-body">
              <input
                autoFocus
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder={t("documents.urlPlaceholder")}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && url.trim()) {
                    void importUrl(url.trim());
                    setUrlOpen(false);
                    setUrl("");
                  }
                }}
              />
            </div>
            <div className="modal-footer">
              <Button variant="ghost" onClick={() => setUrlOpen(false)}>
                {t("app.cancel")}
              </Button>
              <Button
                variant="primary"
                disabled={!url.trim()}
                onClick={() => {
                  void importUrl(url.trim());
                  setUrlOpen(false);
                  setUrl("");
                }}
              >
                {t("app.import")}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
