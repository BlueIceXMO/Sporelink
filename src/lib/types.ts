export type NodeType =
  | "project"
  | "folder"
  | "card"
  | "turn"
  | "message"
  | "document"
  | "chunk"
  | "knowledge"
  | "attachment"
  | "settings"
  | "model_profile"
  | "run";

export type EdgeRelation =
  | "contains"
  | "derives_from"
  | "branch_of"
  | "parallel_to"
  | "cites"
  | "references"
  | "attachment_of"
  | "mentions"
  | "summarizes";

export type CardSurface = "root" | "dive" | "diverge" | "branch" | "child" | "note";

export type AppError = {
  code: string;
  message: string;
  hint?: string | null;
  retryable: boolean;
};

export type OnboardingState = {
  completed: boolean;
  themeDone: boolean;
  languageDone: boolean;
  modelDone: boolean;
};

export type LayoutSettings = {
  density: "comfortable" | "compact";
  cardWidth: number;
  showTree: boolean;
  sidebarCollapsed: boolean;
  reducedMotion: boolean;
  force2dTree: boolean;
};

export type ModelAssignments = {
  chat?: string | null;
  reasoning?: string | null;
  title?: string | null;
  summary?: string | null;
};

export type AutoBehavior = {
  autoTitle: boolean;
  checkpointMs: number;
  confirmDelete: boolean;
  defaultSearch: boolean;
};

export type Settings = {
  schemaVersion: number;
  themeId: string;
  locale: "zh-CN" | "zh-TW" | "en";
  onboarding: OnboardingState;
  layout: LayoutSettings;
  modelAssignments: ModelAssignments;
  autoBehavior: AutoBehavior;
  editPermissions: { readOnly: boolean };
  privacy: { networkTargetsVisible: boolean; customBaseUrlAck: string[] };
  shortcuts: Record<string, string>;
};

export type SettingsState = { settings: Settings; revision: number };

export type ModelCapabilities = {
  vision: boolean;
  tools: boolean;
  json: boolean;
  reasoning: boolean;
};

export type ModelPricing = {
  inputPerMillion: number;
  outputPerMillion: number;
  currency: string;
};

export type ModelProfileData = {
  provider: "mock" | "openai_compatible" | "anthropic" | "gemini";
  label: string;
  baseUrl: string;
  model: string;
  contextWindow: number;
  maxOutput: number;
  capabilities: ModelCapabilities;
  pricing: ModelPricing;
  roles: string[];
  isBuiltin: boolean;
};

export type ModelProfile = {
  id: string;
  revision: number;
  data: ModelProfileData;
};

export type ProjectSummary = {
  id: string;
  name: string;
  description: string;
  color: string;
  folderId: string | null;
  favorite: boolean;
  archived: boolean;
  deletedAt: number | null;
  createdAt: number;
  updatedAt: number;
  revision: number;
  rootCardId: string | null;
  cardCount: number;
  messageCount: number;
};

export type ProjectDetail = {
  project: ProjectSummary;
  rootCardId: string;
  defaultModelProfileId: string | null;
  readOnly: boolean;
};

export type FolderSummary = {
  id: string;
  name: string;
  parentId: string | null;
  order: number;
  createdAt: number;
  updatedAt: number;
  revision: number;
};

export type CardView = {
  id: string;
  title: string;
  surfaceKind: CardSurface;
  position: [number, number, number] | null;
  favorite: boolean;
  locked: boolean;
  archived: boolean;
  parentId: string | null;
  sourceCardId: string | null;
  sourceMessageId: string | null;
  branchPointMessageId: string | null;
  childCount: number;
  messageCount: number;
  createdAt: number;
  updatedAt: number;
  revision: number;
};

export type EdgeView = {
  id: string;
  from: string;
  to: string;
  relation: EdgeRelation;
  order: number;
  metadata: Record<string, unknown>;
};

export type NodeRef = {
  id: string;
  nodeType: NodeType;
  title: string;
  favorite: boolean;
  updatedAt: number;
};

export type GraphView = {
  projectId: string;
  rootCardId: string | null;
  cards: CardView[];
  edges: EdgeView[];
  documents: NodeRef[];
  knowledge: NodeRef[];
};

export type CardPathItem = {
  id: string;
  title: string;
  surfaceKind: CardSurface;
};

export type Citation = {
  sourceType: string;
  sourceId: string;
  title: string;
  locator?: string | null;
};

export type Usage = {
  inputTokens: number;
  outputTokens: number;
  cost: number;
  currency: string;
  latencyMs: number;
  estimated: boolean;
};

export type AttachmentDto = {
  id: string;
  fileName: string;
  mime: string;
  size: number;
  hash: string;
};

export type MessageDto = {
  id: string;
  turnId: string;
  cardId: string;
  role: "user" | "assistant" | "system" | "tool";
  content: string;
  reasoning: string;
  citations: Citation[];
  attachments: AttachmentDto[];
  status: string;
  favorite: boolean;
  editedFrom: string | null;
  regeneratedFrom: string | null;
  usage: Usage | null;
  createdAt: number;
  updatedAt: number;
  revision: number;
};

export type TurnDto = {
  id: string;
  cardId: string;
  index: number;
  collapsed: boolean;
  modelProfileId: string | null;
  contextSnapshotId: string | null;
  messages: MessageDto[];
  createdAt: number;
  updatedAt: number;
  revision: number;
};

export type CardDetail = {
  card: CardView;
  path: CardPathItem[];
  turns: TurnDto[];
  defaultModelProfileId: string | null;
};

export type ContextOptions = {
  excludedItemIds: string[];
  pinnedMessageIds: string[];
  knowledgeIds: string[];
  documentChunkIds: string[];
};

export type ContextItem = {
  id: string;
  kind: string;
  sourceId: string;
  title: string;
  tokens: number;
  reason: string;
  included: boolean;
  droppedReason?: string | null;
  path?: string | null;
};

export type ContextPreview = {
  items: ContextItem[];
  totalTokens: number;
  budget: number;
  droppedCount: number;
  modelProfileId: string;
  messageCount: number;
  systemPromptTokens: number;
};

export type ChatStart = {
  runId: string;
  turnId: string;
  userMessageId: string;
  assistantMessageId: string;
  contextSnapshotId: string;
};

export type ChatSendInput = {
  projectId: string;
  cardId: string;
  content: string;
  modelProfileId: string;
  attachments?: string[];
  context: ContextOptions;
};

export type ChatRegenerateInput = {
  messageId: string;
  modelProfileId?: string;
};

export type SearchInput = {
  query: string;
  projectId?: string | null;
  types?: string[];
  favoriteOnly?: boolean;
  limit?: number;
};

export type ChunkMatch = {
  chunkId: string;
  page: number | null;
  snippet: string;
  score: number;
};

export type ChatEvent =
  | {
      type: "started";
      runId: string;
      turnId: string;
      userMessageId: string;
      assistantMessageId: string;
      contextSnapshotId: string;
    }
  | { type: "delta"; runId: string; messageId: string; text: string }
  | { type: "reasoning"; runId: string; messageId: string; text: string }
  | {
      type: "usage";
      runId: string;
      inputTokens: number;
      outputTokens: number;
      cost: number;
      currency: string;
      latencyMs: number;
      estimated: boolean;
    }
  | { type: "checkpoint"; runId: string; messageId: string; chars: number }
  | { type: "done"; runId: string; messageId: string; status: string }
  | { type: "cancelled"; runId: string; messageId: string }
  | {
      type: "error";
      runId: string;
      code: string;
      message: string;
      hint?: string | null;
      retryable: boolean;
    };

export type SearchHit = {
  nodeId: string;
  nodeType: NodeType;
  projectId: string | null;
  projectName: string;
  cardId: string | null;
  title: string;
  snippet: string;
  score: number;
  updatedAt: number;
};

export type SearchResults = {
  query: string;
  hits: SearchHit[];
  tookMs: number;
};

export type DocumentSummary = {
  id: string;
  projectId: string | null;
  title: string;
  fileName: string;
  path: string;
  mime: string;
  pages: number;
  chunkCount: number;
  createdAt: number;
  updatedAt: number;
};

export type ChunkDto = {
  id: string;
  index: number;
  page: number | null;
  heading: string | null;
  text: string;
};

export type DocumentDetail = {
  document: DocumentSummary;
  chunks: ChunkDto[];
};

export type SourceRef = {
  sourceType: string;
  sourceId: string;
  title: string;
  locator?: string | null;
};

export type KnowledgeDto = {
  id: string;
  projectId: string | null;
  title: string;
  body: string;
  category: string;
  confidence: number;
  tags: string[];
  sourceRefs: SourceRef[];
  createdAt: number;
  updatedAt: number;
  revision: number;
};

export type Counts = {
  projects: number;
  cards: number;
  messages: number;
  documents: number;
  knowledge: number;
  nodes: number;
  edges: number;
};

export type VaultInfo = {
  id: string;
  name: string;
  path: string;
};

export type WorkspaceInfo = {
  id: string;
  name: string;
  root: string;
  internal: string;
  access: "rw" | "ro" | "full" | string;
  createdAt: number;
};

export type WorkspaceFile = {
  path: string;
  name: string;
  size: number;
  isDir: boolean;
  modifiedAt: number;
};

export type Diagnostics = {
  version: string;
  platform: string;
  dataDir: string;
  dbPath: string;
  counts: Counts;
  keychainAvailable: boolean;
  modelProfiles: number;
  recentError: string | null;
};

export type Bootstrap = {
  version: string;
  vault: VaultInfo;
  workspace: WorkspaceInfo;
  settings: Settings;
  settingsRevision: number;
  projects: ProjectSummary[];
  folders: FolderSummary[];
  models: ModelProfile[];
  counts: Counts;
  keychainAvailable: boolean;
  initError: AppError | null;
};

export type ImportResult = {
  format: string;
  importedProjects: number;
  importedNodes: number;
  importedEdges: number;
  remapped: boolean;
};

export type SnapshotSummary = {
  id: string;
  label: string;
  createdAt: number;
  nodeCount: number;
  edgeCount: number;
  path: string;
};

export type TrashItem = {
  id: string;
  nodeType: NodeType;
  title: string;
  deletedAt: number;
  projectId: string | null;
  projectName: string;
  cardId: string | null;
};

export type ModelTestResult = {
  ok: boolean;
  latencyMs: number;
  message: string;
  model: string;
  modelCount: number | null;
};

export type KnowledgeCheckResult = {
  feedback: string;
  modelProfileId: string;
};

export type CardCreateRelation = "dive" | "diverge" | "branch" | "child";

export type Toast = {
  id: string;
  kind: "info" | "success" | "error";
  message: string;
  detail?: string;
};
