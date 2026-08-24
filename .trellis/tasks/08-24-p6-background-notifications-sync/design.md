# Design: P6 后台刷新、通知与外部同步

## Boundary

Core 拥有 due-feed 计算、刷新、同步协议、队列、冲突策略和状态 DTO。Android WorkManager/通知/Keystore 只负责系统集成；Flutter 提供设置、手动动作和可观察状态。

## Job Contract

平台传入 job ID、触发原因和约束快照；Core 返回结构化 refresh/sync report。相同 job 或队列记录可重放。WorkManager 使用唯一周期任务和合理 backoff，不用 60 秒常驻 tick。

## Sync Contract

`SyncPort` 固定 capabilities/pull/push/status。业务写入继续使用 P0 change_log；不得启用另一条互不一致的 `sync_queue` 真相源。凭据以 opaque ref 穿过 Core。

## Failure and Rollback

刷新、通知、同步互相隔离：通知失败不回滚内容，远端失败不覆盖本地待推送状态。先提供手动入口再启用后台触发；可禁用 Worker 而保留同步数据。
