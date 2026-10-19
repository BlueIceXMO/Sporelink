import React, { useEffect, useMemo, useRef, useState } from "react";
import {
  Download,
  FilePlus2,
  FolderPlus,
  Languages,
  Palette,
  PanelLeft,
  Search,
  Settings,
  Sparkles,
  Upload,
  Waypoints,
} from "lucide-react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { THEMES, applyTheme } from "@/lib/themes";
import { cn } from "@/lib/utils";

type Command = {
  id: string;
  label: string;
  hint?: string;
  icon: React.ReactNode;
  run: () => void | Promise<void>;
};

export function CommandPalette() {
  const { locale, t } = useI18n();
  const open = useAppStore((s) => s.commandOpen);
  const setOpen = useAppStore((s) => s.setCommandOpen);
  const setNewProjectOpen = useAppStore((s) => s.setNewProjectOpen);
  const setNewFolderOpen = useAppStore((s) => s.setNewFolderOpen);
  const setSearchOpen = useAppStore((s) => s.setSearchOpen);
  const setSettingsOpen = useAppStore((s) => s.setSettingsOpen);
  const openSample = useAppStore((s) => s.openSample);
  const exportVault = useAppStore((s) => s.exportVault);
  const importVault = useAppStore((s) => s.importVault);
  const toggleSidebar = useAppStore((s) => s.toggleSidebar);
  const settings = useAppStore((s) => s.settings);
  const saveSettings = useAppStore((s) => s.saveSettings);
  const [value, setValue] = useState("");
  const [index, setIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setValue("");
      setIndex(0);
      window.setTimeout(() => inputRef.current?.focus(), 30);
    }
  }, [open]);

  const commands = useMemo<Command[]>(() => {
    const list: Command[] = [
      {
        id: "new-project",
        label: t("command.newProject"),
        hint: "Ctrl N",
        icon: <FilePlus2 size={16} />,
        run: () => setNewProjectOpen(true),
      },
      {
        id: "new-folder",
        label: t("command.newFolder"),
        icon: <FolderPlus size={16} />,
        run: () => setNewFolderOpen(true),
      },
      {
        id: "search",
        label: t("command.search"),
        hint: "Ctrl Shift F",
        icon: <Search size={16} />,
        run: () => setSearchOpen(true),
      },
      {
        id: "settings",
        label: t("command.settings"),
        hint: "Ctrl ,",
        icon: <Settings size={16} />,
        run: () => setSettingsOpen(true),
      },
      {
        id: "sample",
        label: t("command.sample"),
        icon: <Sparkles size={16} />,
        run: () => openSample(),
      },
      {
        id: "export-vault",
        label: t("command.exportVault"),
        icon: <Download size={16} />,
        run: () => exportVault(),
      },
      {
        id: "import-vault",
        label: t("command.importVault"),
        icon: <Upload size={16} />,
        run: async () => {
          const path = await openDialog({
            multiple: false,
            filters: [{ name: "Sporelink vault", extensions: ["json"] }],
          });
          if (typeof path === "string") await importVault(path);
        },
      },
      {
        id: "toggle-sidebar",
        label: t("command.toggleSidebar"),
        icon: <PanelLeft size={16} />,
        run: () => toggleSidebar(),
      },
      {
        id: "theme",
        label: t("command.theme"),
        icon: <Palette size={16} />,
        run: async () => {
          if (!settings) return;
          const idx = THEMES.findIndex((theme) => theme.id === settings.themeId);
          const next = THEMES[(idx + 1) % THEMES.length];
          applyTheme(next.id);
          await saveSettings({ ...settings, themeId: next.id });
        },
      },
      {
        id: "language",
        label: t("command.language"),
        icon: <Languages size={16} />,
        run: async () => {
          if (!settings) return;
          const order = ["zh-CN", "zh-TW", "en"] as const;
          const idx = order.indexOf(settings.locale);
          const next = order[(idx + 1) % order.length];
          await saveSettings({ ...settings, locale: next });
        },
      },
      {
        id: "tree",
        label: t("command.toggleTree"),
        icon: <Waypoints size={16} />,
        run: async () => {
          if (!settings) return;
          await saveSettings({
            ...settings,
            layout: { ...settings.layout, showTree: !settings.layout.showTree },
          });
        },
      },
    ];
    return list;
  }, [
    t,
    setNewProjectOpen,
    setNewFolderOpen,
    setSearchOpen,
    setSettingsOpen,
    openSample,
    exportVault,
    importVault,
    toggleSidebar,
    settings,
    saveSettings,
  ]);

  const filtered = commands.filter((command) =>
    command.label.toLowerCase().includes(value.toLowerCase().trim()),
  );

  if (!open) return null;

  const execute = (command: Command) => {
    setOpen(false);
    void command.run();
  };

  return (
    <div className="command-backdrop" onMouseDown={(e) => e.target === e.currentTarget && setOpen(false)}>
      <div className="command-modal">
        <input
          ref={inputRef}
          value={value}
          onChange={(e) => {
            setValue(e.target.value);
            setIndex(0);
          }}
          placeholder={t("command.placeholder")}
          onKeyDown={(e) => {
            if (e.key === "Escape") setOpen(false);
            if (e.key === "ArrowDown") {
              e.preventDefault();
              setIndex((i) => Math.min(filtered.length - 1, i + 1));
            }
            if (e.key === "ArrowUp") {
              e.preventDefault();
              setIndex((i) => Math.max(0, i - 1));
            }
            if (e.key === "Enter" && filtered[index]) execute(filtered[index]);
          }}
        />
        <div className="command-list">
          {filtered.map((command, i) => (
            <button
              key={command.id}
              type="button"
              className={cn("command-item", i === index && "is-active")}
              onMouseEnter={() => setIndex(i)}
              onClick={() => execute(command)}
            >
              <span className="command-icon">{command.icon}</span>
              <span className="command-label">{command.label}</span>
              {command.hint && <kbd>{command.hint}</kbd>}
            </button>
          ))}
          {filtered.length === 0 && <div className="command-empty">{t("command.noResults")}</div>}
        </div>
        <div className="command-foot">{locale.startsWith("zh") ? "↑↓ 选择 · Enter 执行 · Esc 关闭" : "↑↓ select · Enter run · Esc close"}</div>
      </div>
    </div>
  );
}
