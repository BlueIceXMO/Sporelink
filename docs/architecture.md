# Sporelink 架构（v0.1.0-alpha）

## 1. 形态

- 单 Tauri 2 桌面应用，不嵌 Node runtime，不起本地 HTTP 服务。
- 前端 React + TypeScript + Vite；后端 Rust；唯一业务存储 VelesDB；密钥 OS keychain。
- 代码在 `Sporelink/`；`ref/` 只读参考，不 import、不进发布物。
- 工具链在 `Sporelink_ws/.toolchain/`，不属于产品仓库。

```text
React (TS)
  │  tauri invoke / Channel
  ▼
Rust commands
  ├─ domain       Node/Edge 不变量、表层分类
  ├─ store        VelesDB + StoreWriter + 快照/导入导出
  ├─ context      线性路径、预算、裁剪、快照
  ├─ llm          provider 适配、SSE、取消、usage
  ├─ keychain     OS keychain（只进不出）
  └─ documents    PDF/DOCX/PPTX/HTML/MD 解析与分块
```

## 2. 图模型

### 2.1 一切皆节点

业务内容统一为 `Node`，关系统一为 `Edge`。不存卡片类型。

- 项目、文件夹、卡片、轮次、消息、文档、片段、知识、附件、设置、模型、运行记录都是节点。
- 结构包含、来源、分支、平行、引用、附件、总结都是边。
- 卡片表层类型由 Rust 的 `classify_card` 读图派生；DTO 中的 `surfaceKind` 只服务 UI。
- 删除是软删除；恢复只清 `deletedAt`。
- 图的父子关系是事实的一部分，但可由边的 payload 重建；没有第二套“类型表”。

### 2.2 卡片分类

```text
无 card->card contains 入边        => root
有 parallel_to 关联边              => diverge
有 branch_of 入边                  => branch
有 derives_from 入边               => dive
其余                               => child
```

分类不写库。导入/导出只搬运节点与边，打开项目时重新分类。

### 2.3 布局

- 自动布局读 `contains` 生成树，`parallel_to` 将节点放到同层侧边。
- 手动拖动写 `node.position`；有位置时优先手动位置。
- 3D 树是 SVG 数学投影：yaw/pitch 旋转、滚轮缩放、深度排序、点击命中；支持 2D 降级。

## 3. 存储

### 3.1 VelesDB

- `Database::open(vault_dir/db)`。
- 两个 metadata-only collection：`sporelink_nodes`、`sporelink_edges`。
- `Point::metadata_only(point_id, payload)`；payload 含字符串 ID、类型/关系、projectId、revision、updatedAt、data。
- numeric point id 由 Writer 单调分配；启动时 scroll 全量加载并建索引。
- 所有写入经单一 `StoreWriter` actor（mpsc + oneshot 回执）。
- 读命令拿 `RwLock<GraphState>` 的只读快照；写命令 await Writer。
- `flush_all()`：退出、导入、快照、批量写后。
- 不做 SQLite/FTS 第二套存储；搜索先在 Rust 内存索引上做，后续可加 VelesDB 向量/BM25。

### 3.2 数据目录

```text
~/.sporelink/                # 应用配置，不随工作区迁移
  config.json                # activeWorkspace、recentWorkspaces、主题/语言/布局、
                             # 模型 profiles、访问模式、ignore 规则
  logs/startup.log
  webview2/

<workspace>/                 # 默认 Documents/Sporelink，可配置
  .spore/
    workspace.json           # id、name、schemaVersion、createdAt
    db/                      # VelesDB
    snapshots/               # 全量 JSON 快照
    logs/
  attachments/               # 显式导入的工作区外文件
  papers/ ...                # 用户原始资料，原地保留
```

- 原始文档/附件留在工作区，不复制进 `.spore`；`.spore/db` 只存路径、hash、chunk 引用与本地搜索索引。
- `.spore` 与 `.sporelink` 在扫描、搜索、文件列表、AI 文件读取中硬过滤。
- 当前只启用“仅工作区读写”；`ro` / `full` 是配置里预留的后续模式。
- 路径逻辑边界：canonicalize + 工作区前缀 + 拒绝 `..`/symlink 逃逸；不修改系统 ACL。
- WebView2 profile 在 `~/.sporelink/webview2`，双击运行不依赖外部环境变量。

### 3.3 不变量

- 每个项目恰好一个 `card` 根节点（无 card contains 入边）。
- 所有非根卡片有且仅有一条 `contains` 父边。
- `branch_of` / `derives_from` / `parallel_to` 的源节点必须属于同一项目。
- 删除卡片不静默丢子节点：`tree` 模式软删除整棵子树；`node` 模式把子卡挂到最近存活祖先。
- 任何导出不含 Key、不含 `run` 节点。
- 每次写 +revision；更新带 expectedRevision；冲突返回 `revision_conflict`。

## 4. 上下文编译

1. 从当前卡沿 `contains` 入边向上收集路径。
2. 每张卡按 `contains` 边收集轮次/消息，按 `order` 排序。
3. 分支/深潜读取 `branch_of` / `derives_from` 的 `contextCutMessageId` / `sourceMessageId`，祖先只取到该消息。
4. 加入置顶消息、引用片段、知识、文档 Chunk。
5. 去重；按优先级计算 token；超预算按“文档 → 知识 → 引用 → 旧轮次”的顺序丢弃。
6. 最后一条用户消息永远保留；返回丢弃项与原因。
7. 发送前保存 `ContextSnapshot` 节点，回答可追溯。

token 估算：CJK 字符约 1 token，拉丁字符约 4 字符/token；结果标注为“估算”，不使用供应商 tokenizer。

## 5. 模型网关

- 适配器：`mock`、`openai_compatible`、`anthropic`、`gemini`。
- 统一接口：`stream_chat(request, sink, cancel) -> RunOutcome`。
- 事件：`started / delta / reasoning / usage / done / error / cancelled`。
- SSE 解析：OpenAI `data:`、Anthropic `event:` + `data:`、Gemini `alt=sse`。
- Key 只在 Rust 读；请求头/查询参数不写日志。
- 超时、429/5xx 退避一次；取消令牌；不跨 provider 自动回退。
- 每次运行创建 `run` 节点：provider、model、purpose、tokens、cost、latency、status、error。
- 静态价格表只用于估算；UI 标明估算。

## 6. 安全

- WebView 不直接 fetch、读文件、读 Key。
- CSP 仅允许 `self`、`data:`、`ipc:`；不加载远程脚本/字体。
- Keychain 命令：`secret_set / secret_status / secret_delete`；没有 getter。
- 导出器在序列化前剔除 `apiKey`、`authorization`、`secret` 字段，并有测试。
- 日志只记事件级信息，不记消息正文、请求头、Key。
- 自定义 Base URL 首次使用需确认；TLS 校验不可关闭。
- 无遥测、无远程配置、无自动更新检查。
- 网络目标在 Privacy 设置中可见；React 只调用 Rust 命令。

## 7. 前端

- Zustand 管 UI/投影状态；服务端状态以 Rust 返回为准。
- 所有 IPC 经 `src/lib/ipc.ts` 包装；错误统一转为 toast。
- 组件分层：shell / onboarding / sidebar / project / card / composer / tree / settings / search / documents / knowledge / favorites / common。
- 设计 token 用 CSS 变量；9 套原创主题；字体 Manrope + 系统中文字体；图标 lucide-react。
- Markdown：react-markdown + GFM + KaTeX + highlight.js。
- 3D 树纯 SVG，不引入 three.js。
- 动效克制；`prefers-reduced-motion` 与设置同时生效。

## 8. 构建

- Rust toolchain：`Sporelink_ws/.toolchain`（工作区外，产品仓库外）。
- 前端：npm + Vite；Tauri CLI 作为 devDependency。
- 验证：`npm run typecheck`、`npm run build`、`cargo test`、`cargo check`、`npm run tauri build`。
- 冒烟：启动 exe，WebView2 远程调试 + CDP；不可用时用 PowerShell 截图 + 手动清单。
- 许可证：GPLv3；VelesDB Core License 1.0 的归属与完整文本随发布物。
