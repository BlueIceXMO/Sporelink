# docs/

1. `00-alignment.md` — 技术栈、分层职责、验收口径
2. `ROADMAP.md` — 功能清单与 P0/P1/P2；其中 Electron、SQLite、`packages/*` 已被 00-alignment 取代
3. `feature-parity.md` — Explore 入口/行为到 Sporelink 实现的对照表与状态
4. `contracts.md` — Node/Edge 存储、IPC、事件、导出 schema
5. `architecture.md` — 分层、写入模型、上下文编译、安全边界
6. `manual-smoke.md` — 可复现的手动验收脚本
7. `../ref/reference/notes.md` — 设计 token，仅作参考

规矩：

- 功能范围看 ROADMAP 的 P0/P1/P2，不另立清单。
- 没有 `feature-parity.md` 条目的功能不进入实现。
- 契约（IPC、存储 schema、导出 schema）先改这里，再改代码。
- 不写产品定位与哲学叙事，那部分仍是占位。
- 用中文写。

底层统一模型：

- 业务内容一律是 `Node`，关系一律是 `Edge`。
- 不存 `root/dive/diverge/branch` 这类卡片类型字段；表层类型由 Rust 依据图拓扑与边关系派生。
- 新增“类型”前先问：它能否表达为节点 + 边？能，就不要加枚举。
