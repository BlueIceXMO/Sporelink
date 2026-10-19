import { useEffect, useState } from "react";
import { BookOpen, FileText, Star } from "lucide-react";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { ipc } from "@/lib/ipc";
import type { SearchHit } from "@/lib/types";
import { EmptyState } from "@/components/common/Primitives";
import { Button } from "@/components/common/IconButton";
import { formatDate } from "@/lib/utils";

export function FavoritesView() {
  const { locale, t } = useI18n();
  const [hits, setHits] = useState<SearchHit[]>([]);
  const openSearchHit = useAppStore((s) => s.openSearchHit);
  const favoritesExport = useAppStore((s) => s.toast);
  useEffect(() => {
    void ipc.search({ query: "", favoriteOnly: true, limit: 120 }).then((r) => setHits(r.hits));
  }, []);

  const groups = [
    { id: "project", label: t("favorites.projects") },
    { id: "card", label: t("favorites.cards") },
    { id: "message", label: t("favorites.messages") },
  ];

  const exportFavorites = async () => {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const path = await save({
      defaultPath: "sporelink-favorites.md",
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (!path) return;
    try {
      await ipc.favoritesExport(path, "markdown");
      favoritesExport("success", t("favorites.exported"));
    } catch (error) {
      const e = error as { message?: string };
      favoritesExport("error", e.message ?? "Export failed");
    }
  };

  return (
    <div className="favorites-view">
      <header className="page-head">
        <div>
          <h1>{t("favorites.title")}</h1>
          <p>{t("favorites.subtitle")}</p>
        </div>
        <Button onClick={() => void exportFavorites()}>
          <Star size={15} /> {t("favorites.export")}
        </Button>
      </header>
      {hits.length === 0 ? (
        <EmptyState
          icon={<Star size={28} />}
          title={t("favorites.empty")}
          body={t("favorites.emptyHint")}
        />
      ) : (
        <div className="favorites-groups">
          {groups.map((group) => {
            const items = hits.filter((hit) => hit.nodeType === group.id);
            if (items.length === 0) return null;
            return (
              <section key={group.id} className="favorites-group">
                <h2>{group.label}</h2>
                <div className="favorites-list">
                  {items.map((hit) => (
                    <button
                      key={`${hit.nodeType}:${hit.nodeId}`}
                      type="button"
                      className="favorite-row"
                      onClick={() => void openSearchHit(hit)}
                    >
                      <span className="favorite-icon">
                        {hit.nodeType === "project" ? <BookOpen size={16} /> : <FileText size={16} />}
                      </span>
                      <span className="favorite-main">
                        <strong>{hit.title}</strong>
                        <small>{hit.snippet}</small>
                      </span>
                      <span className="favorite-meta">
                        {hit.projectName} · {formatDate(hit.updatedAt, locale)}
                      </span>
                    </button>
                  ))}
                </div>
              </section>
            );
          })}
        </div>
      )}
    </div>
  );
}
