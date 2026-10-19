# store/

- 两个 metadata collection：`sporelink_nodes`、`sporelink_edges`。
- 所有写操作经 `write_lock`；payload 是事实来源。
- 启动时 scroll 全量重建索引；增删改后同步内存索引。
- 流式检查点、导入、快照前 flush。
- 不做 SQLite / FTS 第二套业务存储。
