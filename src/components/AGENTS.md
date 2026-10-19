# src/components/

- 组件只消费 store / props；写操作调用 `state/store.ts` 的 action。
- 不把业务规则塞进组件；表层卡片类型用 Rust 返回的 `surfaceKind`。
- 弹窗、菜单、选择器要可键盘关闭（Esc）并有可读 label。
- 颜色、圆角、动效统一走 CSS 变量；避免硬编码。
- 大块 Markdown 用 `common/Markdown.tsx`。
