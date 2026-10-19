import { useEffect, useState } from "react";
import { RotateCcw, Trash2 } from "lucide-react";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { ipc } from "@/lib/ipc";
import type { TrashItem } from "@/lib/types";
import { EmptyState } from "@/components/common/Primitives";
import { Button } from "@/components/common/IconButton";
import { formatDateTime } from "@/lib/utils";

const TYPE_LABEL: Record<string, string> = {
  project: "project",
  card: "card",
  turn: "turn",
  message: "message",
  document: "document",
  knowledge: "knowledge",
};

export function TrashView() {
  const { locale, t } = useI18n();
  const [items, setItems] = useState<TrashItem[]>([]);
  const refreshProjects = useAppStore((s) => s.refreshProjects);
  const refreshGraph = useAppStore((s) => s.refreshGraph);
  const toast = useAppStore((s) => s.toast);

  const load = () => {
    void ipc.trashList().then(setItems).catch(() => setItems([]));
  };

  useEffect(load, []);

  const restore = async (item: TrashItem) => {
    try {
      await ipc.trashRestore(item.id);
      await refreshProjects();
      await refreshGraph();
      load();
      toast("success", t("trash.restored"));
    } catch (error) {
      const e = error as { message?: string; hint?: string };
      toast("error", e.message ?? "Restore failed", e.hint);
    }
  };

  return (
    <div className="favorites-view">
      <header className="page-head">
        <div>
          <h1>{t("trash.title")}</h1>
          <p>{t("trash.subtitle")}</p>
        </div>
      </header>
      {items.length === 0 ? (
        <EmptyState icon={<Trash2 size={28} />} title={t("trash.empty")} />
      ) : (
        <div className="favorites-list" style={{ maxWidth: 1000, margin: "0 auto" }}>
          {items.map((item) => (
            <div key={item.id} className="favorite-row">
              <span className="favorite-icon">
                <Trash2 size={16} />
              </span>
              <span className="favorite-main">
                <strong>{item.title || item.id}</strong>
                <small>
                  {TYPE_LABEL[item.nodeType] ?? item.nodeType}
                  {item.projectName ? ` · ${item.projectName}` : ""} ·{" "}
                  {formatDateTime(item.deletedAt, locale)}
                </small>
              </span>
              <Button size="sm" onClick={() => void restore(item)}>
                <RotateCcw size={14} /> {t("trash.restore")}
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
