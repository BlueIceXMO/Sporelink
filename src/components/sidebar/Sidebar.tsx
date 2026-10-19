import { useMemo, useState } from "react";
import {
  Archive,
  BookOpen,
  Copy,
  Download,
  FileText,
  Folder,
  FolderPlus,
  MoreHorizontal,
  Pencil,
  Plus,
  Search,
  Settings,
  Sparkles,
  Star,
  Trash2,
  Upload,
  UserRound,
  PanelLeftClose,
  PanelLeftOpen,
} from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { cn, relativeTime } from "@/lib/utils";
import { IconButton, Button } from "@/components/common/IconButton";
import type { ProjectSummary } from "@/lib/types";

function ProjectRow({
  project,
  onOpen,
}: {
  project: ProjectSummary;
  onOpen: (id: string) => void;
}) {
  const { locale, t } = useI18n();
  const currentProject = useAppStore((s) => s.currentProject);
  const [menu, setMenu] = useState(false);
  const favoriteProject = useAppStore((s) => s.favoriteProject);
  const archiveProject = useAppStore((s) => s.archiveProject);
  const duplicateProject = useAppStore((s) => s.duplicateProject);
  const deleteProject = useAppStore((s) => s.deleteProject);
  const exportProject = useAppStore((s) => s.exportProject);

  return (
    <div className={cn("project-row", currentProject?.project.id === project.id && "is-active")}>
      <button type="button" className="project-main" onClick={() => onOpen(project.id)}>
        <span className="project-dot" style={{ background: project.color || "var(--brand)" }} />
        <span className="project-name">{project.name || t("project.untitled")}</span>
        {project.favorite && <Star size={12} className="project-star" />}
      </button>
      <button
        type="button"
        className="project-more"
        title={t("app.more")}
        onClick={() => setMenu((v) => !v)}
      >
        <MoreHorizontal size={14} />
      </button>
      {menu && (
        <>
          <div className="menu-backdrop" onClick={() => setMenu(false)} />
          <div className="project-menu">
            <button
              type="button"
              onClick={() => {
                setMenu(false);
                void favoriteProject(project.id, !project.favorite);
              }}
            >
              <Star size={14} /> {project.favorite ? t("card.unfavorite") : t("card.favorite")}
            </button>
            <button
              type="button"
              onClick={() => {
                setMenu(false);
                void duplicateProject(project.id);
              }}
            >
              <Copy size={14} /> {t("project.duplicate")}
            </button>
            <button
              type="button"
              onClick={() => {
                setMenu(false);
                void exportProject(project.id, "json");
              }}
            >
              <Download size={14} /> {t("project.exportJson")}
            </button>
            <button
              type="button"
              onClick={() => {
                setMenu(false);
                void archiveProject(project.id, !project.archived);
              }}
            >
              <Archive size={14} /> {project.archived ? t("project.unarchive") : t("project.archive")}
            </button>
            <button
              type="button"
              className="danger"
              onClick={() => {
                setMenu(false);
                if (window.confirm(t("project.deleteConfirm", { name: project.name }))) {
                  void deleteProject(project.id);
                }
              }}
            >
              <Trash2 size={14} /> {t("project.delete")}
            </button>
            <div className="menu-meta">
              {relativeTime(project.updatedAt, locale)} · {t("project.cards", { count: project.cardCount })}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

export function Sidebar() {
  const { t } = useI18n();
  const projects = useAppStore((s) => s.projects);
  const folders = useAppStore((s) => s.folders);
  const currentProject = useAppStore((s) => s.currentProject);
  const view = useAppStore((s) => s.view);
  const sidebarCollapsed = useAppStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useAppStore((s) => s.toggleSidebar);
  const openProject = useAppStore((s) => s.openProject);
  const setView = useAppStore((s) => s.setView);
  const setSearchOpen = useAppStore((s) => s.setSearchOpen);
  const setSettingsOpen = useAppStore((s) => s.setSettingsOpen);
  const setNewProjectOpen = useAppStore((s) => s.setNewProjectOpen);
  const setNewFolderOpen = useAppStore((s) => s.setNewFolderOpen);
  const importProject = useAppStore((s) => s.importProject);

  const rootProjects = useMemo(
    () => projects.filter((p) => !p.archived && !p.deletedAt && !p.folderId),
    [projects],
  );
  const archived = useMemo(
    () => projects.filter((p) => p.archived && !p.deletedAt),
    [projects],
  );

  const onImport = async () => {
    const path = await openDialog({
      multiple: false,
      filters: [{ name: "Sporelink project", extensions: ["json"] }],
    });
    if (typeof path === "string") await importProject(path);
  };

  return (
    <aside className="sidebar">
      <div className="sidebar-head">
        <button
          type="button"
          className="brand"
          onClick={() => {
            setView("welcome");
          }}
          title="Sporelink"
        >
          <span className="brand-mark">
            <span />
            <span />
            <span />
          </span>
          {!sidebarCollapsed && <span className="brand-word">Sporelink</span>}
        </button>
        <IconButton
          label={sidebarCollapsed ? t("sidebar.expand") : t("sidebar.collapse")}
          size="sm"
          onClick={() => void toggleSidebar()}
        >
          {sidebarCollapsed ? <PanelLeftOpen size={17} /> : <PanelLeftClose size={17} />}
        </IconButton>
      </div>

      <div className="sidebar-actions">
        <button type="button" className="sidebar-action" onClick={() => setNewFolderOpen(true)} title={t("sidebar.newFolder")}>
          <FolderPlus size={18} />
          {!sidebarCollapsed && <span>{t("sidebar.newFolder")}</span>}
        </button>
        <button type="button" className="sidebar-action" onClick={() => setNewProjectOpen(true)} title={t("sidebar.newProject")}>
          <Plus size={18} />
          {!sidebarCollapsed && <span>{t("sidebar.newProject")}</span>}
        </button>
        <button
          type="button"
          className="sidebar-action"
          onClick={() => {
            setView("documents");
            void useAppStore.getState().loadDocuments();
          }}
          title={t("sidebar.documents")}
        >
          <FileText size={18} />
          {!sidebarCollapsed && <span>{t("sidebar.documents")}</span>}
        </button>
        <button type="button" className="sidebar-action" onClick={() => void onImport()} title={t("sidebar.importProject")}>
          <Upload size={18} />
          {!sidebarCollapsed && <span>{t("sidebar.importProject")}</span>}
        </button>
      </div>

      <button type="button" className="sidebar-search" onClick={() => setSearchOpen(true)}>
        <Search size={16} />
        {!sidebarCollapsed && <span>{t("sidebar.search")}</span>}
        {!sidebarCollapsed && <kbd>Ctrl K</kbd>}
      </button>

      <nav className="sidebar-scroll">
        <div className="sidebar-section">
          <div className="sidebar-section-title">{t("sidebar.local")}</div>
          {projects.length === 0 && !sidebarCollapsed && (
            <div className="sidebar-empty">{t("sidebar.noProjects")}</div>
          )}
          {folders.map((folder) => {
            const children = projects.filter(
              (p) => p.folderId === folder.id && !p.archived && !p.deletedAt,
            );
            if (children.length === 0) return null;
            return (
              <div key={folder.id} className="folder-group">
                <div className="folder-row">
                  <Folder size={14} />
                  {!sidebarCollapsed && <span>{folder.name}</span>}
                </div>
                {children.map((project) => (
                  <ProjectRow key={project.id} project={project} onOpen={(id) => void openProject(id)} />
                ))}
              </div>
            );
          })}
          {rootProjects.map((project) => (
            <ProjectRow key={project.id} project={project} onOpen={(id) => void openProject(id)} />
          ))}
          {archived.length > 0 && (
            <div className="folder-group">
              <div className="folder-row muted">
                <Archive size={14} />
                {!sidebarCollapsed && <span>{t("sidebar.archived")}</span>}
              </div>
              {archived.map((project) => (
                <ProjectRow key={project.id} project={project} onOpen={(id) => void openProject(id)} />
              ))}
            </div>
          )}
        </div>

        <div className="sidebar-section">
          <button
            type="button"
            className={cn("sidebar-link", view === "favorites" && "is-active")}
            onClick={() => setView("favorites")}
          >
            <Star size={17} />
            {!sidebarCollapsed && <span>{t("sidebar.favorites")}</span>}
          </button>
          <button
            type="button"
            className={cn("sidebar-link", view === "knowledge" && "is-active")}
            onClick={() => {
              setView("knowledge");
              void useAppStore.getState().loadKnowledge();
            }}
          >
            <BookOpen size={17} />
            {!sidebarCollapsed && <span>{t("sidebar.knowledge")}</span>}
          </button>
          <button
            type="button"
            className={cn("sidebar-link", view === "trash" && "is-active")}
            onClick={() => setView("trash")}
          >
            <Trash2 size={17} />
            {!sidebarCollapsed && <span>{t("sidebar.trash")}</span>}
          </button>
        </div>
      </nav>

      <div className="sidebar-foot">
        <button type="button" className="sidebar-link" onClick={() => setSettingsOpen(true)}>
          <Settings size={18} />
          {!sidebarCollapsed && <span>{t("sidebar.settings")}</span>}
        </button>
        <button type="button" className="sidebar-link" disabled title={t("sidebar.accountLocal")}>
          <UserRound size={18} />
          {!sidebarCollapsed && (
            <span className="account-label">
              {t("sidebar.account")}
              <small>{t("sidebar.accountLocal")}</small>
            </span>
          )}
        </button>
        {!sidebarCollapsed && currentProject && (
          <div className="sidebar-project-meta">
            <Sparkles size={13} />
            <span>{t("project.cards", { count: currentProject.project.cardCount })}</span>
          </div>
        )}
        {!sidebarCollapsed && (
          <Button
            variant="ghost"
            size="sm"
            className="sidebar-import"
            onClick={() => void onImport()}
            title={t("app.import")}
          >
            <Upload size={13} /> {t("app.import")}
          </Button>
        )}
      </div>
    </aside>
  );
}
