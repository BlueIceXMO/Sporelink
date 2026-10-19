import { ArrowRight, FileText, Plus, Sparkles, Upload } from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { Button } from "@/components/common/IconButton";
import { relativeTime } from "@/lib/utils";

export function Welcome() {
  const { locale, t } = useI18n();
  const projects = useAppStore((s) => s.projects);
  const openSample = useAppStore((s) => s.openSample);
  const setNewProjectOpen = useAppStore((s) => s.setNewProjectOpen);
  const openProject = useAppStore((s) => s.openProject);
  const importProject = useAppStore((s) => s.importProject);

  const recent = [...projects]
    .filter((p) => !p.deletedAt)
    .sort((a, b) => b.updatedAt - a.updatedAt)
    .slice(0, 6);

  const onImport = async () => {
    const path = await openDialog({
      multiple: false,
      filters: [{ name: "Sporelink project", extensions: ["json"] }],
    });
    if (typeof path === "string") await importProject(path);
  };

  return (
    <div className="welcome-view">
      <div className="welcome-glow" />
      <div className="welcome-inner">
        <div className="welcome-kicker">
          <Sparkles size={14} /> {t("app.local")}
        </div>
        <h1>{t("welcome.title")}</h1>
        <p className="welcome-sub">{t("welcome.subtitle")}</p>

        <div className="welcome-actions">
          <button type="button" className="welcome-card welcome-primary" onClick={() => void openSample()}>
            <span className="welcome-icon">
              <Sparkles size={22} />
            </span>
            <span className="welcome-copy">
              <strong>{t("welcome.how")}</strong>
              <span>{t("welcome.howDesc")}</span>
            </span>
            <ArrowRight size={18} className="welcome-arrow" />
          </button>
          <button type="button" className="welcome-card" onClick={() => setNewProjectOpen(true)}>
            <span className="welcome-icon">
              <Plus size={22} />
            </span>
            <span className="welcome-copy">
              <strong>{t("welcome.new")}</strong>
              <span>{t("welcome.newDesc")}</span>
            </span>
            <ArrowRight size={18} className="welcome-arrow" />
          </button>
          <button type="button" className="welcome-card" onClick={() => void onImport()}>
            <span className="welcome-icon">
              <Upload size={22} />
            </span>
            <span className="welcome-copy">
              <strong>{t("welcome.import")}</strong>
              <span>{t("welcome.importDesc")}</span>
            </span>
            <ArrowRight size={18} className="welcome-arrow" />
          </button>
        </div>

        {recent.length > 0 && (
          <div className="welcome-recent">
            <div className="welcome-section-title">{t("welcome.recent")}</div>
            <div className="recent-grid">
              {recent.map((project) => (
                <button
                  key={project.id}
                  type="button"
                  className="recent-card"
                  onClick={() => void openProject(project.id)}
                >
                  <span className="recent-color" style={{ background: project.color || "var(--brand)" }} />
                  <span className="recent-name">{project.name}</span>
                  <span className="recent-meta">
                    {t("project.cards", { count: project.cardCount })} · {relativeTime(project.updatedAt, locale)}
                  </span>
                </button>
              ))}
            </div>
          </div>
        )}

        <div className="welcome-foot">
          <FileText size={14} />
          <span>{t("welcome.localNote")}</span>
        </div>
      </div>
    </div>
  );
}
