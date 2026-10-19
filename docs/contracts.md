# Sporelink 契约 v1（v0.1.0-alpha）

> 先改本文，再改代码。所有时间戳为 Unix 毫秒；所有 ID 为字符串 UUID。

## 1. 统一图模型

### 1.1 Node

```ts
type Node = {
  id: string;
  nodeType:
    | "project" | "folder" | "card" | "turn" | "message"
    | "document" | "chunk" | "knowledge" | "attachment"
    | "settings" | "model_profile" | "run";
  projectId: string | null;
  title: string;
  body: string;
  data: unknown;
  position: [number, number, number] | null;
  favorite: boolean;
  locked: boolean;
  archived: boolean;
  deletedAt: number | null;
  createdAt: number;
  updatedAt: number;
  revision: number;
};
```

- `body` 只放可读正文；结构化字段放 `data`。
- `deletedAt` 是软删除；所有普通查询排除非空值。
- `revision` 每次写 +1；更新必须带 `expectedRevision`。
- 不存 `cardType` / `surfaceKind` / `root|dive|diverge|branch`。

### 1.2 Edge

```ts
type Edge = {
  id: string;
  projectId: string | null;
  from: string;
  to: string;
  relation:
    | "contains"       // 结构包含：project/folder -> card, card -> turn, turn -> message, document -> chunk
    | "derives_from"   // 新卡 -> 来源消息；知识 -> 消息/片段
    | "branch_of"      // 新卡 -> 分支点消息
    | "parallel_to"    // 发散卡 -> 源卡
    | "cites"          // 消息 -> 文档/片段/知识
    | "references"     // 通用引用
    | "attachment_of"  // 附件 -> 消息
    | "mentions"       // 消息 -> 知识/文档
    | "summarizes";    // 摘要知识 -> 卡片/项目
  order: number;
  metadata: {
    sourceMessageId?: string;
    contextCutMessageId?: string;
    label?: string;
    [key: string]: unknown;
  };
  createdAt: number;
  updatedAt: number;
  revision: number;
};
```

### 1.3 表层卡片类型派生

Rust 的 `classify_card(card_id)` 只读图：

1. 若没有来自其他 `card` 的 `contains` 入边 → `root`。
2. 否则若有 `parallel_to` 关联边 → `diverge`。
3. 否则若有 `branch_of` 入边 → `branch`。
4. 否则若有 `derives_from` 入边 → `dive`。
5. 否则 → `child`。

`surfaceKind` 只存在于 DTO，不入库、不进导出。前端只用它决定标签、颜色和布局提示。

### 1.4 上下文路径

- 从当前 `card` 沿 `contains` 入边向上到根。
- 每张祖先卡按 `turn` / `message` 的 `contains` 边、`order` 排序收集。
- 当前卡若有 `branch_of` 或 `derives_from` 边，读取 `metadata.contextCutMessageId` / `sourceMessageId`；祖先卡只取到该消息。
- `parallel_to` 不改变路径，只改变 3D 布局与表层标签。
- 当前卡自己的消息永远包含；最后一条用户消息永远保留。

## 2. 存储

- VelesDB `velesdb-core` 6.x，metadata-only collections：
  - `sporelink_nodes`
  - `sporelink_edges`
- 每个 point 的 numeric id 由 `StoreWriter` 单调分配；payload 内保留字符串 `id`。
- `payload = { id, nodeType | relation, projectId, revision, updatedAt, data }`。
- 启动时 scroll 全量重建内存索引：`nodesById`、`edgesById`、`outgoing`、`incoming`、`byProject`、`byType`。
- 写入只经单一 `StoreWriter`；读走 `RwLock<GraphState>` 快照。
- 流式助手消息按 `checkpointMs` 检查点落盘；取消/失败保留已生成内容。
- 退出、导入、快照前 `flush_all()`。
- 附件文件放 `attachments/<sha256>`；附件元数据是 `attachment` 节点。

## 3. IPC 命令

命名统一 `snake_case`。所有命令返回 `Result<T, AppError>`。

### 3.1 启动与设置

| 命令 | 入参 | 返回 |
| --- | --- | --- |
| `bootstrap` | 无 | `Bootstrap` |
| `settings_get` | 无 | `Settings` |
| `settings_update` | `{ patch, expectedRevision }` | `Settings` |
| `diagnostics` | 无 | `Diagnostics` |
| `open_data_dir` | 无 | `void` |

`Bootstrap`：`version`、`vault`、`settings`、`projects`、`folders`、`models`、`counts`、`keychainAvailable`。

### 3.2 项目 / 文件夹

| 命令 | 入参 | 返回 |
| --- | --- | --- |
| `project_list` | `{ includeArchived? }` | `ProjectSummary[]` |
| `project_get` | `{ id }` | `ProjectDetail` |
| `project_create` | `{ name, description?, color?, folderId? }` | `ProjectDetail` |
| `project_update` | `{ id, patch, expectedRevision }` | `ProjectDetail` |
| `project_delete` | `{ id }` | `void` |
| `project_restore` | `{ id }` | `ProjectDetail` |
| `project_duplicate` | `{ id, name? }` | `ProjectDetail` |
| `project_summary` | `{ id, modelProfileId? }` | `KnowledgeDto` |
| `project_export` | `{ id, path, format: "json" \| "markdown" }` | `{ path }` |
| `project_import` | `{ path, folderId? }` | `ProjectDetail` |
| `folder_list` | 无 | `FolderSummary[]` |
| `folder_create` | `{ name, parentId? }` | `FolderSummary` |
| `folder_update` | `{ id, patch, expectedRevision }` | `FolderSummary` |
| `folder_delete` | `{ id }` | `void` |

### 3.3 图与卡片

| 命令 | 入参 | 返回 |
| --- | --- | --- |
| `graph_get` | `{ projectId }` | `GraphView` |
| `card_get` | `{ id }` | `CardDetail` |
| `card_create` | `CardCreateInput` | `CardDetail` |
| `card_update` | `{ id, patch, expectedRevision }` | `CardDetail` |
| `card_delete` | `{ id, mode: "tree" \| "node" }` | `void` |
| `card_restore` | `{ id }` | `CardDetail` |
| `card_move` | `{ id, position }` | `CardDetail` |
| `card_favorite` | `{ id, value }` | `CardDetail` |
| `card_lock` | `{ id, value }` | `CardDetail` |

`CardCreateInput`：

```ts
{
  projectId: string;
  parentCardId: string;
  relation: "dive" | "diverge" | "branch" | "child";
  sourceMessageId?: string;
  title?: string;
  initialPrompt?: string;
}
```

- `dive`：`contains(parent -> new)` + `derives_from(new -> sourceMessage)`。
- `diverge`：`contains(parent -> new)` + `derives_from(new -> sourceMessage)` + `parallel_to(new -> sourceCard)`。
- `branch`：`contains(parent -> new)` + `branch_of(new -> sourceMessage)`，`contextCutMessageId = sourceMessageId`。
- `child`：仅 `contains`。

### 3.4 轮次 / 消息 / 对话

| 命令 | 入参 | 返回 |
| --- | --- | --- |
| `turn_update` | `{ id, patch, expectedRevision }` | `TurnDto` |
| `turn_delete` | `{ id }` | `void` |
| `turn_restore` | `{ id }` | `TurnDto` |
| `message_update` | `{ id, patch, expectedRevision }` | `MessageDto` |
| `message_delete` | `{ id }` | `void` |
| `message_restore` | `{ id }` | `MessageDto` |
| `message_favorite` | `{ id, value }` | `MessageDto` |
| `chat_send` | `ChatSendInput` + `Channel<ChatEvent>` | `ChatStart` |
| `chat_regenerate` | `ChatRegenerateInput` + `Channel<ChatEvent>` | `ChatStart` |
| `chat_cancel` | `{ runId }` | `void` |
| `context_preview` | `ContextPreviewInput` | `ContextPreview` |
| `context_snapshot_get` | `{ id }` | `ContextSnapshotDto` |

`ChatSendInput`：

```ts
{
  projectId: string;
  cardId: string;
  content: string;
  modelProfileId: string;
  attachments?: string[];
  context: {
    excludedItemIds?: string[];
    pinnedMessageIds?: string[];
    knowledgeIds?: string[];
    documentChunkIds?: string[];
  };
}
```

### 3.5 搜索 / 收藏

| 命令 | 入参 | 返回 |
| --- | --- | --- |
| `search` | `{ query, projectId?, types?, favoriteOnly?, limit? }` | `SearchResults` |
| `favorites_export` | `{ path, format: "json" \| "markdown" }` | `{ path }` |

### 3.6 文档

| 命令 | 入参 | 返回 |
| --- | --- | --- |
| `document_import` | `{ path, projectId? }` | `DocumentDetail` |
| `document_import_url` | `{ url, projectId? }` | `DocumentDetail` |
| `document_list` | `{ projectId? }` | `DocumentSummary[]` |
| `document_get` | `{ id }` | `DocumentDetail` |
| `document_delete` | `{ id }` | `void` |
| `document_search` | `{ id, query }` | `ChunkMatch[]` |

### 3.7 知识

| 命令 | 入参 | 返回 |
| --- | --- | --- |
| `knowledge_list` | `{ projectId?, query?, tag? }` | `KnowledgeDto[]` |
| `knowledge_get` | `{ id }` | `KnowledgeDto` |
| `knowledge_create` | `{ projectId?, title, body, category?, confidence?, tags?, sourceRefs? }` | `KnowledgeDto` |
| `knowledge_update` | `{ id, patch, expectedRevision }` | `KnowledgeDto` |
| `knowledge_delete` | `{ id }` | `void` |
| `knowledge_from_message` | `{ messageId, selection, title? }` | `KnowledgeDto` |
| `knowledge_check` | `{ id, modelProfileId? }` | `{ feedback, modelProfileId }` |

### 3.8 模型 / 密钥

| 命令 | 入参 | 返回 |
| --- | --- | --- |
| `model_profile_list` | 无 | `ModelProfileDto[]` |
| `model_profile_create` | `ModelProfileInput` | `ModelProfileDto` |
| `model_profile_update` | `{ id, patch, expectedRevision }` | `ModelProfileDto` |
| `model_profile_delete` | `{ id }` | `void` |
| `model_test` | `{ id }` | `ModelTestResult` |
| `model_list_remote` | `{ id }` | `string[]` |
| `secret_set` | `{ profileId, key }` | `{ ok: true }` |
| `secret_status` | `{ profileId }` | `{ configured: boolean }` |
| `secret_delete` | `{ profileId }` | `{ ok: true }` |

### 3.9 数据 / 示例

| 命令 | 入参 | 返回 |
| --- | --- | --- |
| `vault_export` | `{ path }` | `{ path }` |
| `vault_import` | `{ path, mode: "merge" \| "replace" }` | `ImportResult` |
| `vault_clear` | `{ confirm: string }` | `void` |
| `snapshot_create` | `{ label? }` | `SnapshotSummary` |
| `snapshot_list` | 无 | `SnapshotSummary[]` |
| `snapshot_restore` | `{ id }` | `void` |
| `sample_install` | `{ locale }` | `ProjectDetail` |

## 4. 事件

```ts
type ChatEvent =
  | { type: "started"; runId; turnId; userMessageId; assistantMessageId; contextSnapshotId }
  | { type: "delta"; runId; messageId; text }
  | { type: "reasoning"; runId; messageId; text }
  | { type: "usage"; runId; inputTokens; outputTokens; cost; latencyMs }
  | { type: "checkpoint"; runId; messageId; chars }
  | { type: "done"; runId; messageId; status: "complete" }
  | { type: "cancelled"; runId; messageId }
  | { type: "error"; runId; code; message; hint?; retryable };
```

- 后端是唯一写者；前端收到 delta 只更新内存显示。
- `done`/`error`/`cancelled` 后，前端重新拉取 `card_get` 以对齐落盘内容。

## 5. 错误

```ts
type AppError = {
  code:
    | "not_found" | "revision_conflict" | "invalid_input" | "store_error"
    | "model_not_configured" | "key_missing" | "keychain_unavailable"
    | "provider_auth" | "provider_rate_limit" | "provider_unavailable"
    | "provider_timeout" | "context_overflow" | "cancelled"
    | "parse_error" | "io_error" | "unsupported";
  message: string;
  hint?: string;
  retryable: boolean;
};
```

错误信息不得包含消息正文、Authorization、API Key、完整请求 URL 中的 key 查询参数。

## 6. 导出 schema

### 6.1 全 Vault

```json
{
  "format": "sporelink.vault",
  "schemaVersion": 1,
  "appVersion": "0.1.0-alpha",
  "exportedAt": 0,
  "vault": { "id": "...", "name": "..." },
  "nodes": [],
  "edges": [],
  "sanitized": true
}
```

### 6.2 项目

```json
{
  "format": "sporelink.project",
  "schemaVersion": 1,
  "appVersion": "0.1.0-alpha",
  "exportedAt": 0,
  "projectId": "...",
  "nodes": [],
  "edges": [],
  "sanitized": true
}
```

- 导出包含 `model_profile` 节点，但绝不含 Key。
- 导入时若 ID 冲突，重新分配节点/边 ID 并重建内部引用。
- 导入先校验格式与版本，再写临时图，最后原子替换/合并；失败不破坏现有数据。
- 不导入 `run`、`settings` 节点（运行时/本机设置）。
