# src-tauri/

Tauri 2 + Rust 后端。唯一业务存储是 VelesDB；Key 只在 Rust 进程内。

- `src/domain`：Node/Edge、DTO、不变量；不依赖 Tauri/React。
- `src/store`：VelesDB 集合、单一写锁、启动重建索引、快照/导入导出。
- `src/context`：上下文路径、预算、裁剪、快照条目。
- `src/llm`：Mock/OpenAI-compatible/Anthropic/Gemini；不把 Key 返回前端。
- `src/documents`：PDF/DOCX/PPTX/HTML/MD 解析与分块。
- `src/search`：内存全文搜索；不做后台索引。
- `src/export`：JSON/Markdown 导出与脱敏。
- `src/commands`：Tauri IPC；错误统一 `AppError`。
- 写入必须经 `Store` 的写锁；更新必须带 expected revision。
- 不在日志里打印消息正文、Authorization、API Key。
