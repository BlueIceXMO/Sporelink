# context/

- 从当前卡沿 `contains` 入边向上收集路径。
- 深潜/分支读取 `derives_from` / `branch_of` 的 cut message。
- 优先级：置顶 > 最近路径 > 引用 > 知识 > 文档。
- 最后一条用户消息始终保留；超预算显示丢弃原因。
- 预览与发送必须用同一个 `compile`，避免两套逻辑漂移。
