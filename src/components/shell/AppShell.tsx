import { Sidebar } from "@/components/sidebar/Sidebar";
import { Welcome } from "@/components/project/Welcome";
import { ProjectView } from "@/components/project/ProjectView";
import { DocumentView } from "@/components/documents/DocumentView";
import { KnowledgeView } from "@/components/knowledge/KnowledgeView";
import { FavoritesView } from "@/components/favorites/FavoritesView";
import { TrashView } from "@/components/favorites/TrashView";
import { CardTree3D } from "@/components/tree/CardTree3D";
import { useAppStore } from "@/state/store";

export function AppShell() {
  const view = useAppStore((s) => s.view);
  const currentProject = useAppStore((s) => s.currentProject);
  const settings = useAppStore((s) => s.settings);
  const sidebarCollapsed = useAppStore((s) => s.sidebarCollapsed);

  const showTree =
    Boolean(currentProject) &&
    view === "project" &&
    Boolean(settings?.layout.showTree);

  return (
    <div className={`app-shell ${sidebarCollapsed ? "sidebar-collapsed" : ""} ${showTree ? "with-tree" : ""}`}>
      <Sidebar />
      <main className="app-main">
        {view === "welcome" || !currentProject ? <Welcome /> : null}
        {view === "project" && currentProject ? <ProjectView /> : null}
        {view === "documents" ? <DocumentView /> : null}
        {view === "knowledge" ? <KnowledgeView /> : null}
        {view === "favorites" ? <FavoritesView /> : null}
        {view === "trash" ? <TrashView /> : null}
      </main>
      {showTree && <CardTree3D />}
    </div>
  );
}
