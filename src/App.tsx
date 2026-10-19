import { useEffect } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { AppShell } from "@/components/shell/AppShell";
import { Toasts } from "@/components/common/Toasts";
import { Button } from "@/components/common/IconButton";
import { Onboarding } from "@/components/onboarding/Onboarding";
import { SettingsModal } from "@/components/settings/SettingsModal";
import { SearchModal } from "@/components/search/SearchModal";
import { CommandPalette } from "@/components/search/CommandPalette";
import { NewProjectDialog } from "@/components/project/NewProjectDialog";
import { NewFolderDialog } from "@/components/sidebar/NewFolderDialog";
import { ContextDrawer } from "@/components/composer/ContextDrawer";
import { applyTheme } from "@/lib/themes";
import { isEditableTarget } from "@/lib/utils";

export default function App() {
  const initialize = useAppStore((s) => s.initialize);
  const loading = useAppStore((s) => s.loading);
  const boot = useAppStore((s) => s.boot);
  const fatalError = useAppStore((s) => s.fatalError);
  const settings = useAppStore((s) => s.settings);
  const onboardingOpen = useAppStore((s) => s.onboardingOpen);
  const contextOpen = useAppStore((s) => s.contextOpen);
  const streaming = useAppStore((s) => s.streaming);
  const cancelStream = useAppStore((s) => s.cancelStream);
  const setCommandOpen = useAppStore((s) => s.setCommandOpen);
  const setSearchOpen = useAppStore((s) => s.setSearchOpen);
  const setSettingsOpen = useAppStore((s) => s.setSettingsOpen);
  const setNewProjectOpen = useAppStore((s) => s.setNewProjectOpen);
  const openCard = useAppStore((s) => s.openCard);
  const graph = useAppStore((s) => s.graph);
  const currentCard = useAppStore((s) => s.currentCard);
  const { t } = useI18n();

  useEffect(() => {
    void initialize();
  }, [initialize]);

  useEffect(() => {
    if (settings) {
      applyTheme(settings.themeId);
      document.documentElement.lang = settings.locale;
    }
  }, [settings]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const mod = event.ctrlKey || event.metaKey;
      if (mod && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setCommandOpen(true);
        return;
      }
      if (mod && event.shiftKey && event.key.toLowerCase() === "f") {
        event.preventDefault();
        setSearchOpen(true);
        return;
      }
      if (mod && event.key === ",") {
        event.preventDefault();
        setSettingsOpen(true);
        return;
      }
      if (mod && !event.shiftKey && event.key.toLowerCase() === "n") {
        event.preventDefault();
        setNewProjectOpen(true);
        return;
      }
      if (event.key === "Escape" && streaming) {
        event.preventDefault();
        void cancelStream();
        return;
      }
      if (event.altKey && graph && !isEditableTarget(event.target)) {
        const cards = graph.cards;
        const index = cards.findIndex((c) => c.id === currentCard?.card.id);
        if (event.key === "ArrowRight" || event.key === "ArrowDown") {
          event.preventDefault();
          const next = cards[Math.min(cards.length - 1, index + 1)];
          if (next) void openCard(next.id);
        } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
          event.preventDefault();
          const prev = cards[Math.max(0, index - 1)];
          if (prev) void openCard(prev.id);
        } else if (event.key === "Home" && graph.rootCardId) {
          event.preventDefault();
          void openCard(graph.rootCardId);
        }
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [
    streaming,
    cancelStream,
    setCommandOpen,
    setSearchOpen,
    setSettingsOpen,
    setNewProjectOpen,
    graph,
    currentCard,
    openCard,
  ]);

  if (loading) {
    return (
      <div className="splash">
        <div className="splash-mark">
          <span className="splash-dot" />
          <span className="splash-dot" />
          <span className="splash-dot" />
        </div>
        <div className="splash-title">Sporelink</div>
        <div className="splash-sub">{t("app.loading")}</div>
      </div>
    );
  }

  if (fatalError || !boot) {
    return (
      <div className="splash">
        <div className="fatal-card">
          <AlertTriangle size={28} />
          <h1>{t("app.error")}</h1>
          <p>{fatalError?.message ?? "Unknown error"}</p>
          {fatalError?.hint && <p className="muted">{fatalError.hint}</p>}
          <Button variant="primary" onClick={() => void initialize()}>
            <RefreshCw size={16} /> {t("app.retry")}
          </Button>
        </div>
      </div>
    );
  }

  return (
    <>
      <AppShell />
      {onboardingOpen && <Onboarding />}
      <SettingsModal />
      <SearchModal />
      <CommandPalette />
      <NewProjectDialog />
      <NewFolderDialog />
      {contextOpen && <ContextDrawer />}
      <Toasts />
    </>
  );
}
