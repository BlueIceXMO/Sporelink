# src/

React + TypeScript 前端。

- 只通过 `src/lib/ipc.ts` 调用 Rust；不直接 fetch、不读文件、不读 Key。
- 服务端状态以 Rust 返回为准；Zustand 只放投影与 UI 状态。
- 组件目录：`components/*`；共享逻辑：`lib/`；状态：`state/`。
- 主题 token 在 `lib/themes.ts`；不要在组件里硬编码颜色。
- 新增用户可见文案时同时补 `lib/i18n.ts` 的三种语言。
- 不要 import `../ref/`。
