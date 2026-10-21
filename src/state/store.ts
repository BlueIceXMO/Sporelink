import { create } from "zustand";
import { ipc, normalizeError } from "@/lib/ipc";
import type {
  AppError,
  Bootstrap,
  CardCreateRelation,
  CardDetail,
  ChatEvent,
  ChatStart,
  ContextOptions,
  ContextPreview,
  DocumentDetail,
  DocumentSummary,
  FolderSummary,
  GraphView,
  KnowledgeDto,
  ModelProfile,
  ProjectDetail,
  ProjectSummary,
  SearchHit,
  Settings,
  SnapshotSummary,
  Toast,
} from "@/lib/types";
import { applyTheme } from "@/lib/themes";
import { newLocalId, truncate } from "@/lib/utils";

export type View =
  | "welcome"
  | "project"
  | "documents"
  | "knowledge"
  | "favorites"
  | "trash";

type StreamState = {
  runId: string;
  messageId: string;
  cardId: string;
  turnId: string;
  active: boolean;
};

type AppState = {
  boot: Bootstrap | null;
  loading: boolean;
  fatalError: AppError | null;
  settings: Settings | null;
  settingsRevision: number;
  projects: ProjectSummary[];
  folders: FolderSummary[];
  models: ModelProfile[];
  currentProject: ProjectDetail | null;
  graph: GraphView | null;
  currentCard: CardDetail | null;
  selectedCardId: string | null;
  view: View;
  documents: DocumentSummary[];
  currentDocument: DocumentDetail | null;
  knowledge: KnowledgeDto[];
  searchQuery: string;
  searchType: string;
  searchFavoriteOnly: boolean;
  searchResults: SearchHit[];
  searchTookMs: number;
  searchOpen: boolean;
  commandOpen: boolean;
  settingsOpen: boolean;
  contextOpen: boolean;
  onboardingOpen: boolean;
  newProjectOpen: boolean;
  newFolderOpen: boolean;
  busy: Record<string, boolean>;
  streaming: StreamState | null;
  contextPreview: ContextPreview | null;
  contextOptions: ContextOptions;
  contextDraft: string;
  toasts: Toast[];
  snapshots: SnapshotSummary[];
  themePreview: string | null;
  sidebarCollapsed: boolean;
  tree2d: boolean;
  composerModelId: string | null;
  draftToInsert: string | null;
  lastError: string | null;

  initialize: () => Promise<void>;
  refreshBootstrap: () => Promise<void>;
  openWorkspace: (path: string) => Promise<boolean>;
  openDefaultWorkspace: () => Promise<boolean>;
  setView: (view: View) => void;
  openProject: (id: string) => Promise<void>;
  closeProject: () => void;
  refreshProjects: () => Promise<void>;
  refreshFolders: () => Promise<void>;
  refreshModels: () => Promise<void>;
  openCard: (id: string) => Promise<void>;
  refreshGraph: () => Promise<void>;
  refreshCard: () => Promise<void>;
  createProject: (input: {
    name: string;
    description?: string;
    color?: string;
    folderId?: string | null;
  }) => Promise<ProjectDetail | null>;
  updateProject: (
    id: string,
    patch: Record<string, unknown>,
    expectedRevision: number,
  ) => Promise<ProjectDetail | null>;
  deleteProject: (id: string) => Promise<void>;
  duplicateProject: (id: string) => Promise<void>;
  archiveProject: (id: string, archived: boolean) => Promise<void>;
  favoriteProject: (id: string, favorite: boolean) => Promise<void>;
  createFolder: (name: string) => Promise<void>;
  deleteFolder: (id: string) => Promise<void>;
  createCard: (
    relation: CardCreateRelation,
    sourceMessageId?: string,
    title?: string,
  ) => Promise<CardDetail | null>;
  updateCard: (patch: Record<string, unknown>, expectedRevision: number) => Promise<void>;
  deleteCard: (mode: "tree" | "node") => Promise<void>;
  favoriteCard: (value: boolean) => Promise<void>;
  lockCard: (value: boolean) => Promise<void>;
  toggleTurn: (turnId: string, collapsed: boolean) => Promise<void>;
  deleteTurn: (turnId: string) => Promise<void>;
  updateMessage: (
    messageId: string,
    patch: Record<string, unknown>,
    expectedRevision: number,
  ) => Promise<void>;
  deleteMessage: (messageId: string) => Promise<void>;
  favoriteMessage: (messageId: string, value: boolean) => Promise<void>;
  sendMessage: (content: string, attachments?: string[]) => Promise<boolean>;
  regenerate: (messageId: string) => Promise<void>;
  cancelStream: () => Promise<void>;
  openContext: (draft: string) => Promise<void>;
  closeContext: () => void;
  refreshContext: (draft: string) => Promise<void>;
  toggleContextItem: (itemId: string) => void;
  setContextPinned: (messageId: string, pinned: boolean) => void;
  toggleKnowledgeContext: (knowledgeId: string) => void;
  saveSettings: (settings: Settings) => Promise<boolean>;
  previewTheme: (themeId: string) => void;
  cancelThemePreview: () => void;
  setLocale: (locale: string) => Promise<void>;
  setOnboardingOpen: (open: boolean) => void;
  setNewProjectOpen: (open: boolean) => void;
  setNewFolderOpen: (open: boolean) => void;
  setSettingsOpen: (open: boolean) => void;
  setSearchOpen: (open: boolean) => void;
  setCommandOpen: (open: boolean) => void;
  search: (query: string, type?: string, favoriteOnly?: boolean) => Promise<void>;
  openSearchHit: (hit: SearchHit) => Promise<void>;
  loadDocuments: (projectId?: string | null) => Promise<void>;
  openDocument: (id: string) => Promise<void>;
  importDocument: (path: string) => Promise<void>;
  importUrl: (url: string) => Promise<void>;
  deleteDocument: (id: string) => Promise<void>;
  loadKnowledge: (projectId?: string | null, query?: string) => Promise<void>;
  createKnowledge: (input: {
    projectId?: string | null;
    title: string;
    body: string;
    category?: string;
    confidence?: number;
    tags?: string[];
    sourceRefs?: unknown[];
  }) => Promise<KnowledgeDto | null>;
  updateKnowledge: (
    id: string,
    patch: Record<string, unknown>,
    expectedRevision: number,
  ) => Promise<void>;
  deleteKnowledge: (id: string) => Promise<void>;
  checkKnowledge: (id: string) => Promise<string | null>;
  saveKnowledgeFromMessage: (
    messageId: string,
    selection: string,
    title?: string,
  ) => Promise<void>;
  openSample: () => Promise<void>;
  exportProject: (id: string, format: "json" | "markdown") => Promise<void>;
  importProject: (path: string) => Promise<void>;
  exportVault: () => Promise<void>;
  importVault: (path: string) => Promise<void>;
  loadSnapshots: () => Promise<void>;
  createSnapshot: () => Promise<void>;
  restoreSnapshot: (id: string) => Promise<void>;
  toggleSidebar: () => Promise<void>;
  setTree2d: (value: boolean) => Promise<void>;
  setComposerModelId: (id: string | null) => void;
  insertDraft: (text: string) => void;
  consumeDraft: () => void;
  toast: (kind: Toast["kind"], message: string, detail?: string) => void;
  dismissToast: (id: string) => void;
  setBusy: (key: string, value: boolean) => void;
  clearFatal: () => void;
};

const emptyContext: ContextOptions = {
  excludedItemIds: [],
  pinnedMessageIds: [],
  knowledgeIds: [],
  documentChunkIds: [],
};

function errorToast(error: unknown): { message: string; detail?: string } {
  const e = normalizeError(error);
  return { message: e.message, detail: e.hint ?? e.code };
}

function defaultModelId(models: ModelProfile[], settings: Settings | null): string | null {
  const assigned = settings?.modelAssignments.chat;
  if (assigned && models.some((m) => m.id === assigned)) return assigned;
  const mock = models.find((m) => m.data.provider === "mock");
  return mock?.id ?? models[0]?.id ?? null;
}

function updateMessageInCard(
  card: CardDetail,
  messageId: string,
  updater: (message: CardDetail["turns"][number]["messages"][number]) => CardDetail["turns"][number]["messages"][number],
): CardDetail {
  return {
    ...card,
    turns: card.turns.map((turn) => ({
      ...turn,
      messages: turn.messages.map((m) => (m.id === messageId ? updater(m) : m)),
    })),
  };
}

export const useAppStore = create<AppState>((set, get) => ({
  boot: null,
  loading: true,
  fatalError: null,
  settings: null,
  settingsRevision: 0,
  projects: [],
  folders: [],
  models: [],
  currentProject: null,
  graph: null,
  currentCard: null,
  selectedCardId: null,
  view: "welcome",
  documents: [],
  currentDocument: null,
  knowledge: [],
  searchQuery: "",
  searchType: "all",
  searchFavoriteOnly: false,
  searchResults: [],
  searchTookMs: 0,
  searchOpen: false,
  commandOpen: false,
  settingsOpen: false,
  contextOpen: false,
  onboardingOpen: false,
  newProjectOpen: false,
  newFolderOpen: false,
  busy: {},
  streaming: null,
  contextPreview: null,
  contextOptions: emptyContext,
  contextDraft: "",
  toasts: [],
  snapshots: [],
  themePreview: null,
  sidebarCollapsed: false,
  tree2d: false,
  composerModelId: null,
  draftToInsert: null,
  lastError: null,

  initialize: async () => {
    set({ loading: true, fatalError: null });
    try {
      const boot = await ipc.bootstrap();
      const settings = boot.settings;
      applyTheme(settings.themeId);
      set({
        boot,
        settings,
        settingsRevision: boot.settingsRevision,
        projects: boot.projects,
        folders: boot.folders,
        models: boot.models,
        composerModelId: defaultModelId(boot.models, settings),
        sidebarCollapsed: settings.layout.sidebarCollapsed,
        tree2d: settings.layout.force2dTree,
        onboardingOpen: !settings.onboarding.completed,
        loading: false,
        fatalError: boot.initError ?? null,
        lastError: boot.initError?.message ?? null,
      });
    } catch (error) {
      const e = normalizeError(error);
      set({ loading: false, fatalError: e, lastError: e.message });
    }
  },

  refreshBootstrap: async () => {
    const boot = await ipc.bootstrap();
    applyTheme(boot.settings.themeId);
    set({
      boot,
      settings: boot.settings,
      settingsRevision: boot.settingsRevision,
      projects: boot.projects,
      folders: boot.folders,
      models: boot.models,
      sidebarCollapsed: boot.settings.layout.sidebarCollapsed,
      tree2d: boot.settings.layout.force2dTree,
    });
  },

  openWorkspace: async (path) => {
    set({ busy: { ...get().busy, workspace: true } });
    try {
      const boot = await ipc.workspaceOpen(path);
      applyTheme(boot.settings.themeId);
      set({
        boot,
        settings: boot.settings,
        settingsRevision: boot.settingsRevision,
        projects: boot.projects,
        folders: boot.folders,
        models: boot.models,
        composerModelId: defaultModelId(boot.models, boot.settings),
        sidebarCollapsed: boot.settings.layout.sidebarCollapsed,
        tree2d: boot.settings.layout.force2dTree,
        onboardingOpen: !boot.settings.onboarding.completed,
        view: "welcome",
        currentProject: null,
        graph: null,
        currentCard: null,
        selectedCardId: null,
        contextPreview: null,
        streaming: null,
        documents: [],
        currentDocument: null,
        knowledge: [],
        snapshots: [],
        searchResults: [],
        searchTookMs: 0,
        loading: false,
        fatalError: boot.initError ?? null,
      });
      get().toast(
        "success",
        get().settings?.locale === "en" ? "Workspace opened" : "工作区已打开",
        boot.workspace.root,
      );
      return true;
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
      return false;
    } finally {
      set({ busy: { ...get().busy, workspace: false } });
    }
  },

  openDefaultWorkspace: async () => {
    set({ busy: { ...get().busy, workspace: true } });
    try {
      const boot = await ipc.workspaceDefault();
      applyTheme(boot.settings.themeId);
      set({
        boot,
        settings: boot.settings,
        settingsRevision: boot.settingsRevision,
        projects: boot.projects,
        folders: boot.folders,
        models: boot.models,
        composerModelId: defaultModelId(boot.models, boot.settings),
        sidebarCollapsed: boot.settings.layout.sidebarCollapsed,
        tree2d: boot.settings.layout.force2dTree,
        onboardingOpen: !boot.settings.onboarding.completed,
        view: "welcome",
        currentProject: null,
        graph: null,
        currentCard: null,
        selectedCardId: null,
        contextPreview: null,
        streaming: null,
        documents: [],
        currentDocument: null,
        knowledge: [],
        snapshots: [],
        loading: false,
        fatalError: boot.initError ?? null,
      });
      return true;
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
      return false;
    } finally {
      set({ busy: { ...get().busy, workspace: false } });
    }
  },

  setView: (view) => set({ view }),

  openProject: async (id) => {
    set({ busy: { ...get().busy, project: true } });
    try {
      const detail = await ipc.projectGet(id);
      const rootId = detail.rootCardId;
      set({
        currentProject: detail,
        view: "project",
        selectedCardId: rootId,
        graph: null,
        currentCard: null,
        composerModelId:
          detail.defaultModelProfileId ??
          defaultModelId(get().models, get().settings) ??
          get().composerModelId,
      });
      await Promise.all([get().refreshGraph(), get().openCard(rootId)]);
    } catch (error) {
      get().toast("error", errorToast(error).message, errorToast(error).detail);
    } finally {
      set({ busy: { ...get().busy, project: false } });
    }
  },

  closeProject: () =>
    set({ currentProject: null, graph: null, currentCard: null, selectedCardId: null, view: "welcome" }),

  refreshProjects: async () => {
    try {
      const projects = await ipc.projectList(true);
      set({ projects });
    } catch (error) {
      get().toast("error", errorToast(error).message);
    }
  },

  refreshFolders: async () => {
    try {
      const folders = await ipc.folderList();
      set({ folders });
    } catch (error) {
      get().toast("error", errorToast(error).message);
    }
  },

  refreshModels: async () => {
    try {
      const models = await ipc.modelProfileList();
      set({ models });
    } catch (error) {
      get().toast("error", errorToast(error).message);
    }
  },

  openCard: async (id) => {
    set({ selectedCardId: id });
    try {
      const card = await ipc.cardGet(id);
      set({ currentCard: card });
    } catch (error) {
      get().toast("error", errorToast(error).message);
    }
  },

  refreshGraph: async () => {
    const project = get().currentProject;
    if (!project) return;
    try {
      const graph = await ipc.graphGet(project.project.id);
      set({ graph });
    } catch (error) {
      get().toast("error", errorToast(error).message);
    }
  },

  refreshCard: async () => {
    const id = get().selectedCardId;
    if (!id) return;
    try {
      const card = await ipc.cardGet(id);
      set({ currentCard: card });
    } catch {
      // A soft-deleted card can disappear; keep the previous view.
    }
  },

  createProject: async (input) => {
    try {
      const detail = await ipc.projectCreate(input);
      await get().refreshProjects();
      set({ newProjectOpen: false });
      await get().openProject(detail.project.id);
      get().toast("success", get().settings?.locale === "en" ? "Project created" : "项目已创建");
      return detail;
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
      return null;
    }
  },

  updateProject: async (id, patch, expectedRevision) => {
    try {
      const detail = await ipc.projectUpdate(id, patch, expectedRevision);
      set({ currentProject: detail });
      await get().refreshProjects();
      return detail;
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
      return null;
    }
  },

  deleteProject: async (id) => {
    try {
      await ipc.projectDelete(id);
      if (get().currentProject?.project.id === id) get().closeProject();
      await get().refreshProjects();
      get().toast("success", get().settings?.locale === "en" ? "Project deleted" : "项目已删除");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  duplicateProject: async (id) => {
    try {
      const detail = await ipc.projectDuplicate(id);
      await get().refreshProjects();
      await get().openProject(detail.project.id);
      get().toast("success", get().settings?.locale === "en" ? "Project duplicated" : "项目已复制");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  archiveProject: async (id, archived) => {
    try {
      const current = get().projects.find((p) => p.id === id);
      if (!current) return;
      await ipc.projectUpdate(id, { archived }, current.revision);
      await get().refreshProjects();
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  favoriteProject: async (id, favorite) => {
    try {
      const current = get().projects.find((p) => p.id === id);
      if (!current) return;
      await ipc.projectUpdate(id, { favorite }, current.revision);
      await get().refreshProjects();
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  createFolder: async (name) => {
    try {
      await ipc.folderCreate(name);
      await get().refreshFolders();
      set({ newFolderOpen: false });
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  deleteFolder: async (id) => {
    try {
      await ipc.folderDelete(id);
      await get().refreshFolders();
      await get().refreshProjects();
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  createCard: async (relation, sourceMessageId, title) => {
    const project = get().currentProject;
    const card = get().currentCard;
    if (!project || !card) return null;
    try {
      const created = await ipc.cardCreate({
        projectId: project.project.id,
        parentCardId: card.card.id,
        relation,
        sourceMessageId,
        title,
      });
      await get().refreshGraph();
      await get().refreshProjects();
      await get().openCard(created.card.id);
      return created;
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
      return null;
    }
  },

  updateCard: async (patch, expectedRevision) => {
    const card = get().currentCard;
    if (!card) return;
    try {
      const updated = await ipc.cardUpdate(card.card.id, patch, expectedRevision);
      set({ currentCard: updated });
      await get().refreshGraph();
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  deleteCard: async (mode) => {
    const card = get().currentCard;
    if (!card) return;
    try {
      await ipc.cardDelete(card.card.id, mode);
      await get().refreshGraph();
      const root = get().currentProject?.rootCardId;
      if (root) await get().openCard(root);
      get().toast("success", get().settings?.locale === "en" ? "Card deleted" : "卡片已删除");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  favoriteCard: async (value) => {
    const card = get().currentCard;
    if (!card) return;
    try {
      const updated = await ipc.cardFavorite(card.card.id, value);
      set({ currentCard: updated });
      await get().refreshGraph();
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  lockCard: async (value) => {
    const card = get().currentCard;
    if (!card) return;
    try {
      const updated = await ipc.cardLock(card.card.id, value);
      set({ currentCard: updated });
      await get().refreshGraph();
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  toggleTurn: async (turnId, collapsed) => {
    const card = get().currentCard;
    const turn = card?.turns.find((t) => t.id === turnId);
    if (!card || !turn) return;
    try {
      await ipc.turnUpdate(turnId, { collapsed }, turn.revision);
      set({
        currentCard: {
          ...card,
          turns: card.turns.map((t) => (t.id === turnId ? { ...t, collapsed } : t)),
        },
      });
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  deleteTurn: async (turnId) => {
    try {
      await ipc.turnDelete(turnId);
      await get().refreshCard();
      await get().refreshGraph();
      get().toast("success", get().settings?.locale === "en" ? "Round deleted" : "轮次已删除");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  updateMessage: async (messageId, patch, expectedRevision) => {
    try {
      await ipc.messageUpdate(messageId, patch, expectedRevision);
      await get().refreshCard();
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  deleteMessage: async (messageId) => {
    try {
      await ipc.messageDelete(messageId);
      await get().refreshCard();
      await get().refreshGraph();
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  favoriteMessage: async (messageId, value) => {
    try {
      await ipc.messageFavorite(messageId, value);
      const card = get().currentCard;
      if (card) {
        set({
          currentCard: updateMessageInCard(card, messageId, (m) => ({ ...m, favorite: value })),
        });
      }
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  sendMessage: async (content, attachments = []) => {
    const project = get().currentProject;
    const card = get().currentCard;
    const models = get().models;
    const settings = get().settings;
    if (!project || !card || !content.trim()) return false;
    if (project.readOnly || card.card.locked) {
      get().toast("error", project.readOnly ? "项目只读" : "卡片已锁定");
      return false;
    }
    const modelProfileId = get().composerModelId ?? defaultModelId(models, settings);
    if (!modelProfileId) {
      get().toast("error", settings?.locale === "en" ? "Configure a model first" : "请先配置一个模型");
      return false;
    }
    set({ busy: { ...get().busy, chat: true } });
    let start: ChatStart | null = null;
    try {
      const onEvent = (event: ChatEvent) => {
        const state = get();
        if (event.type === "delta") {
          if (state.selectedCardId !== card.card.id) return;
          set((s) =>
            s.currentCard
              ? {
                  currentCard: updateMessageInCard(s.currentCard, event.messageId, (m) => ({
                    ...m,
                    content: m.content + event.text,
                    status: "streaming",
                  })),
                }
              : s,
          );
        } else if (event.type === "reasoning") {
          if (state.selectedCardId !== card.card.id) return;
          set((s) =>
            s.currentCard
              ? {
                  currentCard: updateMessageInCard(s.currentCard, event.messageId, (m) => ({
                    ...m,
                    reasoning: m.reasoning + event.text,
                  })),
                }
              : s,
          );
        } else if (event.type === "usage") {
          if (state.selectedCardId !== card.card.id) return;
          set((s) =>
            s.currentCard
              ? {
                  currentCard: updateMessageInCard(s.currentCard, start?.assistantMessageId ?? "", (m) => ({
                    ...m,
                    usage: {
                      inputTokens: event.inputTokens,
                      outputTokens: event.outputTokens,
                      cost: event.cost,
                      currency: event.currency,
                      latencyMs: event.latencyMs,
                      estimated: event.estimated,
                    },
                  })),
                }
              : s,
          );
        } else if (event.type === "done") {
          set({ streaming: null, busy: { ...get().busy, chat: false } });
          void get().refreshCard();
          void get().refreshGraph();
        } else if (event.type === "cancelled") {
          set({ streaming: null, busy: { ...get().busy, chat: false } });
          get().toast("info", get().settings?.locale === "en" ? "Generation stopped" : "已停止生成");
          void get().refreshCard();
        } else if (event.type === "error") {
          set({ streaming: null, busy: { ...get().busy, chat: false } });
          get().toast("error", event.message, event.hint ?? event.code);
          void get().refreshCard();
        }
      };
      start = await ipc.chatSend(
        {
          projectId: project.project.id,
          cardId: card.card.id,
          content: content.trim(),
          modelProfileId,
          attachments,
          context: get().contextOptions,
        },
        onEvent,
      );
      set({
        streaming: {
          runId: start.runId,
          messageId: start.assistantMessageId,
          cardId: card.card.id,
          turnId: start.turnId,
          active: true,
        },
        contextOptions: emptyContext,
      });
      await get().refreshCard();
      await get().refreshGraph();
      return true;
    } catch (error) {
      set({ busy: { ...get().busy, chat: false }, streaming: null });
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
      return false;
    }
  },

  regenerate: async (messageId) => {
    const card = get().currentCard;
    if (!card) return;
    set({ busy: { ...get().busy, chat: true } });
    try {
      const onEvent = (event: ChatEvent) => {
        if (event.type === "delta") {
          set((s) =>
            s.currentCard
              ? {
                  currentCard: updateMessageInCard(s.currentCard, event.messageId, (m) => ({
                    ...m,
                    content: m.content + event.text,
                    status: "streaming",
                  })),
                }
              : s,
          );
        } else if (event.type === "done" || event.type === "cancelled" || event.type === "error") {
          set({ streaming: null, busy: { ...get().busy, chat: false } });
          if (event.type === "error") get().toast("error", event.message, event.hint ?? event.code);
          void get().refreshCard();
          void get().refreshGraph();
        }
      };
      const start = await ipc.chatRegenerate(
        { messageId, modelProfileId: get().composerModelId ?? defaultModelId(get().models, get().settings) ?? undefined },
        onEvent,
      );
      set({
        streaming: {
          runId: start.runId,
          messageId: start.assistantMessageId,
          cardId: card.card.id,
          turnId: start.turnId,
          active: true,
        },
      });
      await get().refreshCard();
    } catch (error) {
      set({ busy: { ...get().busy, chat: false }, streaming: null });
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  cancelStream: async () => {
    const runId = get().streaming?.runId;
    if (!runId) return;
    try {
      await ipc.chatCancel(runId);
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  openContext: async (draft) => {
    set({ contextOpen: true, contextDraft: draft });
    await get().refreshContext(draft);
  },

  closeContext: () => set({ contextOpen: false }),

  refreshContext: async (draft) => {
    const card = get().currentCard;
    const modelId = get().composerModelId ?? defaultModelId(get().models, get().settings);
    if (!card || !modelId) {
      set({ contextPreview: null });
      return;
    }
    try {
      const preview = await ipc.contextPreview({
        cardId: card.card.id,
        draftContent: draft,
        modelProfileId: modelId,
        context: get().contextOptions,
      });
      set({ contextPreview: preview });
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  toggleContextItem: (itemId) => {
    const options = get().contextOptions;
    const excluded = new Set(options.excludedItemIds);
    if (excluded.has(itemId)) excluded.delete(itemId);
    else excluded.add(itemId);
    set({ contextOptions: { ...options, excludedItemIds: [...excluded] } });
    void get().refreshContext(get().contextDraft);
  },

  setContextPinned: (messageId, pinned) => {
    const options = get().contextOptions;
    const setIds = new Set(options.pinnedMessageIds);
    if (pinned) setIds.add(messageId);
    else setIds.delete(messageId);
    set({ contextOptions: { ...options, pinnedMessageIds: [...setIds] } });
    void get().refreshContext(get().contextDraft);
  },

  toggleKnowledgeContext: (knowledgeId) => {
    const options = get().contextOptions;
    const ids = new Set(options.knowledgeIds);
    if (ids.has(knowledgeId)) ids.delete(knowledgeId);
    else ids.add(knowledgeId);
    set({ contextOptions: { ...options, knowledgeIds: [...ids] } });
    if (get().contextOpen) void get().refreshContext(get().contextDraft);
  },

  saveSettings: async (settings) => {
    try {
      const state = await ipc.settingsUpdate(settings, get().settingsRevision);
      set({
        settings: state.settings,
        settingsRevision: state.revision,
        sidebarCollapsed: state.settings.layout.sidebarCollapsed,
        tree2d: state.settings.layout.force2dTree,
        themePreview: null,
        onboardingOpen: !state.settings.onboarding.completed,
      });
      applyTheme(state.settings.themeId);
      get().toast("success", get().settings?.locale === "en" ? "Settings saved" : "设置已保存");
      return true;
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
      return false;
    }
  },

  previewTheme: (themeId) => {
    applyTheme(themeId);
    set({ themePreview: themeId });
  },

  cancelThemePreview: () => {
    const themeId = get().settings?.themeId ?? "ink";
    applyTheme(themeId);
    set({ themePreview: null });
  },

  setLocale: async (locale) => {
    const settings = get().settings;
    if (!settings) return;
    await get().saveSettings({ ...settings, locale: locale as Settings["locale"] });
  },

  setOnboardingOpen: (open) => set({ onboardingOpen: open }),
  setNewProjectOpen: (open) => set({ newProjectOpen: open }),
  setNewFolderOpen: (open) => set({ newFolderOpen: open }),
  setSettingsOpen: (open) => set({ settingsOpen: open }),
  setSearchOpen: (open) => set({ searchOpen: open }),
  setCommandOpen: (open) => set({ commandOpen: open }),

  search: async (query, type = get().searchType, favoriteOnly = get().searchFavoriteOnly) => {
    set({ searchQuery: query, searchType: type, searchFavoriteOnly: favoriteOnly });
    try {
      const result = await ipc.search({
        query,
        types: type && type !== "all" ? [type] : undefined,
        favoriteOnly,
        limit: 80,
      });
      set({ searchResults: result.hits, searchTookMs: result.tookMs });
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  openSearchHit: async (hit) => {
    set({ searchOpen: false });
    if (hit.projectId) {
      if (get().currentProject?.project.id !== hit.projectId) {
        await get().openProject(hit.projectId);
      }
      if (hit.cardId) {
        await get().openCard(hit.cardId);
        set({ view: "project" });
      } else if (hit.nodeType === "document") {
        await get().openDocument(hit.nodeId);
        set({ view: "documents" });
      } else if (hit.nodeType === "knowledge") {
        set({ view: "knowledge" });
      }
    }
  },

  loadDocuments: async (projectId = get().currentProject?.project.id) => {
    try {
      const documents = await ipc.documentList(projectId);
      set({ documents });
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  openDocument: async (id) => {
    try {
      const detail = await ipc.documentGet(id);
      set({ currentDocument: detail, view: "documents" });
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  importDocument: async (path) => {
    set({ busy: { ...get().busy, document: true } });
    try {
      const detail = await ipc.documentImport(path, get().currentProject?.project.id);
      set({ currentDocument: detail, view: "documents" });
      await get().loadDocuments();
      get().toast(
        "success",
        get().settings?.locale === "en" ? "Document imported" : "文档已导入",
        detail.document.path,
      );
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    } finally {
      set({ busy: { ...get().busy, document: false } });
    }
  },

  importUrl: async (url) => {
    set({ busy: { ...get().busy, document: true } });
    try {
      const detail = await ipc.documentImportUrl(url, get().currentProject?.project.id);
      set({ currentDocument: detail, view: "documents" });
      await get().loadDocuments();
      get().toast("success", get().settings?.locale === "en" ? "Document imported" : "文档已导入");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    } finally {
      set({ busy: { ...get().busy, document: false } });
    }
  },

  deleteDocument: async (id) => {
    try {
      await ipc.documentDelete(id);
      set({ currentDocument: null });
      await get().loadDocuments();
      get().toast("success", get().settings?.locale === "en" ? "Document deleted" : "文档已删除");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  loadKnowledge: async (projectId = get().currentProject?.project.id, query) => {
    try {
      const knowledge = await ipc.knowledgeList(projectId, query);
      set({ knowledge });
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  createKnowledge: async (input) => {
    try {
      const knowledge = await ipc.knowledgeCreate(input);
      set({ knowledge: [knowledge, ...get().knowledge] });
      get().toast("success", get().settings?.locale === "en" ? "Knowledge saved" : "知识已保存");
      return knowledge;
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
      return null;
    }
  },

  updateKnowledge: async (id, patch, expectedRevision) => {
    try {
      const knowledge = await ipc.knowledgeUpdate(id, patch, expectedRevision);
      set({ knowledge: get().knowledge.map((k) => (k.id === id ? knowledge : k)) });
      get().toast("success", get().settings?.locale === "en" ? "Knowledge saved" : "知识已保存");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  deleteKnowledge: async (id) => {
    try {
      await ipc.knowledgeDelete(id);
      set({ knowledge: get().knowledge.filter((k) => k.id !== id) });
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  checkKnowledge: async (id) => {
    try {
      const result = await ipc.knowledgeCheck(
        id,
        defaultModelId(get().models, get().settings) ?? undefined,
      );
      return result.feedback;
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
      return null;
    }
  },

  saveKnowledgeFromMessage: async (messageId, selection, title) => {
    try {
      const knowledge = await ipc.knowledgeFromMessage(messageId, selection, title);
      set({ knowledge: [knowledge, ...get().knowledge] });
      get().toast("success", get().settings?.locale === "en" ? "Saved as knowledge" : "已存为知识");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  openSample: async () => {
    try {
      const detail = await ipc.sampleInstall(get().settings?.locale ?? "zh-CN");
      await get().refreshProjects();
      set({ onboardingOpen: false });
      await get().openProject(detail.project.id);
      get().toast(
        "success",
        get().settings?.locale === "en" ? "Sample project opened" : "示例项目已打开",
      );
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  exportProject: async (id, format) => {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const path = await save({
        defaultPath: `${truncate(get().projects.find((p) => p.id === id)?.name ?? "sporelink", 40)}.${format === "json" ? "sporelink.json" : "md"}`,
        filters: [
          format === "json"
            ? { name: "Sporelink project", extensions: ["json"] }
            : { name: "Markdown", extensions: ["md"] },
        ],
      });
      if (!path) return;
      await ipc.projectExport(id, path, format);
      get().toast("success", get().settings?.locale === "en" ? "Project exported" : "项目已导出");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  importProject: async (path) => {
    try {
      const detail = await ipc.projectImport(path);
      await get().refreshProjects();
      await get().openProject(detail.project.id);
      get().toast("success", get().settings?.locale === "en" ? "Project imported" : "项目已导入");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  exportVault: async () => {
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const path = await save({
        defaultPath: `sporelink-vault-${new Date().toISOString().slice(0, 10)}.json`,
        filters: [{ name: "Sporelink vault", extensions: ["json"] }],
      });
      if (!path) return;
      await ipc.vaultExport(path);
      get().toast("success", get().settings?.locale === "en" ? "Export complete" : "导出完成");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  importVault: async (path) => {
    try {
      await ipc.vaultImport(path, "merge");
      await get().refreshBootstrap();
      get().toast("success", get().settings?.locale === "en" ? "Import complete" : "导入完成");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  loadSnapshots: async () => {
    try {
      const snapshots = await ipc.snapshotList();
      set({ snapshots });
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  createSnapshot: async () => {
    try {
      const snap = await ipc.snapshotCreate();
      set({ snapshots: [snap, ...get().snapshots] });
      get().toast("success", get().settings?.locale === "en" ? "Snapshot created" : "快照已创建");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  restoreSnapshot: async (id) => {
    try {
      await ipc.snapshotRestore(id);
      await get().refreshBootstrap();
      if (get().currentProject) await get().openProject(get().currentProject!.project.id);
      get().toast("success", get().settings?.locale === "en" ? "Snapshot restored" : "快照已恢复");
    } catch (error) {
      const e = errorToast(error);
      get().toast("error", e.message, e.detail);
    }
  },

  toggleSidebar: async () => {
    const settings = get().settings;
    if (!settings) return;
    const next = !get().sidebarCollapsed;
    set({ sidebarCollapsed: next });
    await get().saveSettings({
      ...settings,
      layout: { ...settings.layout, sidebarCollapsed: next },
    });
  },

  setTree2d: async (value) => {
    const settings = get().settings;
    if (!settings) return;
    set({ tree2d: value });
    await get().saveSettings({
      ...settings,
      layout: { ...settings.layout, force2dTree: value },
    });
  },

  setComposerModelId: (id) => set({ composerModelId: id }),

  insertDraft: (text) => set({ draftToInsert: text }),
  consumeDraft: () => set({ draftToInsert: null }),

  toast: (kind, message, detail) => {
    const toast: Toast = { id: newLocalId(), kind, message, detail };
    set({ toasts: [...get().toasts, toast] });
    const ttl = kind === "error" ? 7000 : 3500;
    window.setTimeout(() => get().dismissToast(toast.id), ttl);
  },

  dismissToast: (id) => set({ toasts: get().toasts.filter((t) => t.id !== id) }),

  setBusy: (key, value) => set({ busy: { ...get().busy, [key]: value } }),

  clearFatal: () => set({ fatalError: null }),
}));
