# llm/

- 统一 `stream_chat`；事件 delta/reasoning/usage。
- 适配器：mock、openai_compatible、anthropic、gemini。
- Key 只在 Rust 读取；不记录请求头或 URL 里的 key。
- 不跨 provider 自动回退；错误映射到 AppError。
- 供应商 SSE 解析失败要返回可读错误，不要静默吞掉。
