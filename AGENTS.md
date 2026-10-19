# Sporelink/

本地优先桌面应用。产品代码在这里；`../ref/` 只读，不属于本仓库。

## 先读

- `docs/AGENTS.md`
- `docs/00-alignment.md`、`docs/contracts.md`、`docs/feature-parity.md`
- `README.md`

## 硬约束

- 栈：Tauri 2 / Rust / TS / React / Vite / VelesDB / OS keychain。
- 业务内容一律是 `Node`，关系一律是 `Edge`；不新增 `cardType` 枚举。
- 卡片表层类型由 Rust 依据图拓扑派生，只在 DTO 出现。
- API Key 只在 Rust 进程内；不得进前端、日志、导出。
- 改 IPC、存储 schema、导出 schema 前先改 `docs/contracts.md`。
- 不假设未验证的命令可用；构建/测试命令以 README 与实际验证为准。
- 每个代码目录有自己的 `AGENTS.md`。
