# src-tauri/src/

后端模块规则：

- 业务内容统一为 `Node` + `Edge`；不存卡片类型。
- 卡片表层类型只由 `service::card_view` 派生。
- Store 写操作串行；读操作拿 `RwLock` 快照。
- 流式助手消息按检查点落盘；取消/失败保留已生成内容。
- 新增命令先更新 `docs/contracts.md`，并在 `lib.rs` 注册。
- 错误信息不得包含消息正文、请求头、API Key。
