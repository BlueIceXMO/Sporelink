# commands/

- 每个命令只做参数校验 + 调用 service/模块；业务逻辑不要堆在命令层。
- 命令名/参数与 `src/lib/ipc.ts`、`docs/contracts.md` 一致。
- 返回 `AppResult<T>`；错误用 `AppError`。
- 流式命令用 `Channel`，不要把原始 SSE 透传给前端。
- 新命令必须在 `src-tauri/src/lib.rs` 注册。
