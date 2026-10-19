# domain/

- `model.rs` 只有 Node/Edge/枚举；DTO 在 `dto.rs`。
- 不新增 `CardType`；表层类型用 `CardSurface`（派生值）。
- 字段用 camelCase 序列化；时间戳为 Unix 毫秒。
- 改字段时同步 `src/lib/types.ts` 与 `docs/contracts.md`。
