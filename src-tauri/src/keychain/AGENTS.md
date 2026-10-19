# keychain/

- 只提供 set/status/delete/get-internal；没有把 Key 返回前端的命令。
- service `com.sporelink.app`，account `provider:<profileId>`。
- 不写明文 fallback；keychain 不可用就报错。
- 不记录 Key。
