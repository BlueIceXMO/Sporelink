# src/lib/

- `ipc.ts` 是唯一 IPC 边界；命令名/参数必须与 `docs/contracts.md` 一致。
- `types.ts` 镜像 Rust DTO；改 DTO 时同步两边。
- `themes.ts` 只放 token 与 apply；不 import 组件。
- `i18n.ts` 保持三语言键一致；新增键先加 zh-CN。
- 纯函数优先；不要在这里放 React 组件。
