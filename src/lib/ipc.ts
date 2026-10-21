import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AppError,
  Bootstrap,
  CardCreateRelation,
  CardDetail,
  CardView,
  ChatEvent,
  ChatRegenerateInput,
  ChatSendInput,
  ChatStart,
  ChunkMatch,
  ContextOptions,
  ContextPreview,
  Diagnostics,
  DocumentDetail,
  DocumentSummary,
  FolderSummary,
  GraphView,
  ImportResult,
  KnowledgeCheckResult,
  KnowledgeDto,
  ModelProfile,
  ModelProfileData,
  ModelTestResult,
  ProjectDetail,
  ProjectSummary,
  SearchInput,
  SearchResults,
  Settings,
  SettingsState,
  SnapshotSummary,
  TrashItem,
  WorkspaceFile,
  WorkspaceInfo,
} from "./types";

export const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function normalizeError(error: unknown): AppError {
  if (typeof error === "object" && error !== null && "code" in error && "message" in error) {
    const e = error as AppError;
    return {
      code: String(e.code),
      message: String(e.message),
      hint: e.hint ?? null,
      retryable: Boolean(e.retryable),
    };
  }
  if (error instanceof Error) {
    return { code: "unknown", message: error.message, hint: null, retryable: true };
  }
  return { code: "unknown", message: String(error), hint: null, retryable: true };
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeError(error);
  }
}

function channel<T>(onMessage: (message: T) => void): Channel<T> {
  const ch = new Channel<T>();
  ch.onmessage = onMessage;
  return ch;
}

export const ipc = {
  bootstrap: () => call<Bootstrap>("bootstrap"),
  settingsGet: () => call<SettingsState>("settings_get"),
  settingsUpdate: (settings: Settings, expectedRevision: number) =>
    call<SettingsState>("settings_update", { settings, expectedRevision }),
  diagnostics: () => call<Diagnostics>("diagnostics"),
  openDataDir: () => call<void>("open_data_dir"),
  workspaceInfo: () => call<WorkspaceInfo>("workspace_info"),
  workspaceOpen: (path: string) => call<Bootstrap>("workspace_open", { path }),
  workspaceDefault: () => call<Bootstrap>("workspace_default"),
  workspaceFileList: () => call<WorkspaceFile[]>("workspace_file_list"),
  workspaceReadFile: (path: string, startLine?: number, endLine?: number) =>
    call<string>("workspace_read_file", { path, startLine, endLine }),

  projectList: (includeArchived = false) =>
    call<ProjectSummary[]>("project_list", { includeArchived }),
  projectGet: (id: string) => call<ProjectDetail>("project_get", { id }),
  projectCreate: (input: {
    name: string;
    description?: string;
    color?: string;
    folderId?: string | null;
  }) => call<ProjectDetail>("project_create", { input }),
  projectUpdate: (id: string, patch: Record<string, unknown>, expectedRevision: number) =>
    call<ProjectDetail>("project_update", { id, patch, expectedRevision }),
  projectDelete: (id: string) => call<void>("project_delete", { id }),
  projectRestore: (id: string) => call<ProjectDetail>("project_restore", { id }),
  projectDuplicate: (id: string, name?: string) =>
    call<ProjectDetail>("project_duplicate", { id, name }),
  projectSummary: (id: string, modelProfileId?: string) =>
    call<KnowledgeDto>("project_summary", { id, modelProfileId }),
  projectExport: (id: string, path: string, format: "json" | "markdown") =>
    call<{ path: string }>("project_export", { id, path, format }),
  projectImport: (path: string, folderId?: string | null) =>
    call<ProjectDetail>("project_import", { path, folderId }),

  folderList: () => call<FolderSummary[]>("folder_list"),
  folderCreate: (name: string, parentId?: string | null) =>
    call<FolderSummary>("folder_create", { name, parentId }),
  folderUpdate: (id: string, patch: Record<string, unknown>, expectedRevision: number) =>
    call<FolderSummary>("folder_update", { id, patch, expectedRevision }),
  folderDelete: (id: string) => call<void>("folder_delete", { id }),

  graphGet: (projectId: string) => call<GraphView>("graph_get", { projectId }),
  cardGet: (id: string) => call<CardDetail>("card_get", { id }),
  cardCreate: (input: {
    projectId: string;
    parentCardId: string;
    relation: CardCreateRelation;
    sourceMessageId?: string;
    title?: string;
  }) => call<CardDetail>("card_create", { input }),
  cardUpdate: (id: string, patch: Record<string, unknown>, expectedRevision: number) =>
    call<CardDetail>("card_update", { id, patch, expectedRevision }),
  cardDelete: (id: string, mode: "tree" | "node") =>
    call<void>("card_delete", { id, mode }),
  cardRestore: (id: string) => call<CardDetail>("card_restore", { id }),
  cardMove: (id: string, position: [number, number, number]) =>
    call<CardDetail>("card_move", { id, position }),
  cardFavorite: (id: string, value: boolean) =>
    call<CardDetail>("card_favorite", { id, value }),
  cardLock: (id: string, value: boolean) => call<CardDetail>("card_lock", { id, value }),

  turnUpdate: (id: string, patch: Record<string, unknown>, expectedRevision: number) =>
    call("turn_update", { id, patch, expectedRevision }),
  turnDelete: (id: string) => call<void>("turn_delete", { id }),
  turnRestore: (id: string) => call("turn_restore", { id }),

  messageUpdate: (id: string, patch: Record<string, unknown>, expectedRevision: number) =>
    call("message_update", { id, patch, expectedRevision }),
  messageDelete: (id: string) => call<void>("message_delete", { id }),
  messageRestore: (id: string) => call("message_restore", { id }),
  messageFavorite: (id: string, value: boolean) =>
    call("message_favorite", { id, value }),

  chatSend: (input: ChatSendInput, onEvent: (e: ChatEvent) => void) =>
    call<ChatStart>("chat_send", { input, onEvent: channel(onEvent) }),
  chatRegenerate: (input: ChatRegenerateInput, onEvent: (e: ChatEvent) => void) =>
    call<ChatStart>("chat_regenerate", { input, onEvent: channel(onEvent) }),
  chatCancel: (runId: string) => call<void>("chat_cancel", { runId }),

  contextPreview: (input: {
    cardId: string;
    draftContent: string;
    modelProfileId: string;
    context: ContextOptions;
  }) => call<ContextPreview>("context_preview", { input }),
  contextSnapshotGet: (id: string) => call("context_snapshot_get", { id }),

  search: (input: SearchInput) => call<SearchResults>("search", { input }),
  favoritesExport: (path: string, format: "json" | "markdown") =>
    call<{ path: string }>("favorites_export", { path, format }),

  documentImport: (path: string, projectId?: string | null) =>
    call<DocumentDetail>("document_import", { path, projectId }),
  documentImportUrl: (url: string, projectId?: string | null) =>
    call<DocumentDetail>("document_import_url", { url, projectId }),
  documentList: (projectId?: string | null) =>
    call<DocumentSummary[]>("document_list", { projectId }),
  documentGet: (id: string) => call<DocumentDetail>("document_get", { id }),
  documentDelete: (id: string) => call<void>("document_delete", { id }),
  documentSearch: (id: string, query: string) =>
    call<ChunkMatch[]>("document_search", { id, query }),

  knowledgeList: (projectId?: string | null, query?: string, tag?: string) =>
    call<KnowledgeDto[]>("knowledge_list", { projectId, query, tag }),
  knowledgeGet: (id: string) => call<KnowledgeDto>("knowledge_get", { id }),
  knowledgeCreate: (input: {
    projectId?: string | null;
    title: string;
    body: string;
    category?: string;
    confidence?: number;
    tags?: string[];
    sourceRefs?: unknown[];
  }) => call<KnowledgeDto>("knowledge_create", { input }),
  knowledgeUpdate: (id: string, patch: Record<string, unknown>, expectedRevision: number) =>
    call<KnowledgeDto>("knowledge_update", { id, patch, expectedRevision }),
  knowledgeDelete: (id: string) => call<void>("knowledge_delete", { id }),
  knowledgeFromMessage: (messageId: string, selection: string, title?: string) =>
    call<KnowledgeDto>("knowledge_from_message", { messageId, selection, title }),
  knowledgeCheck: (id: string, modelProfileId?: string) =>
    call<KnowledgeCheckResult>("knowledge_check", { input: { id, modelProfileId } }),

  modelProfileList: () => call<ModelProfile[]>("model_profile_list"),
  modelProfileCreate: (input: ModelProfileData) =>
    call<ModelProfile>("model_profile_create", { input }),
  modelProfileUpdate: (id: string, patch: Record<string, unknown>, expectedRevision: number) =>
    call<ModelProfile>("model_profile_update", { id, patch, expectedRevision }),
  modelProfileDelete: (id: string) => call<void>("model_profile_delete", { id }),
  modelTest: (id: string) => call<ModelTestResult>("model_test", { id }),
  modelListRemote: (id: string) => call<string[]>("model_list_remote", { id }),

  secretSet: (profileId: string, key: string) =>
    call<{ ok: true }>("secret_set", { profileId, key }),
  secretStatus: (profileId: string) =>
    call<{ configured: boolean }>("secret_status", { profileId }),
  secretDelete: (profileId: string) => call<{ ok: true }>("secret_delete", { profileId }),

  vaultExport: (path: string) => call<{ path: string }>("vault_export", { path }),
  vaultImport: (path: string, mode: "merge" | "replace") =>
    call<ImportResult>("vault_import", { path, mode }),
  vaultClear: (confirm: string) => call<void>("vault_clear", { confirm }),
  snapshotCreate: (label?: string) => call<SnapshotSummary>("snapshot_create", { label }),
  snapshotList: () => call<SnapshotSummary[]>("snapshot_list"),
  snapshotRestore: (id: string) => call<void>("snapshot_restore", { id }),
  trashList: () => call<TrashItem[]>("trash_list"),
  trashRestore: (id: string) => call<void>("trash_restore", { id }),
  sampleInstall: (locale: string) => call<ProjectDetail>("sample_install", { locale }),
};

export type { CardView, ContextOptions };
