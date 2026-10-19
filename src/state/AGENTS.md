# src/state/

- 单一 Zustand store；action 内调用 `ipc`。
- 流式事件按 `messageId` 更新当前卡片；切卡后不要写错卡片。
- 错误统一走 `toast`；revision 冲突提示刷新。
- 不要在这里放 JSX。
