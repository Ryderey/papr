# P6 后台刷新、通知与外部同步

## Goal

使用 Android 系统调度和通知完成尽力而为的后台刷新，并把 FreshRSS/Miniflux 协议下沉 Core，形成可重试、可恢复、凭据安全的同步闭环。

## Requirements

1. WorkManager 只负责唤醒；到期判断、条件请求、幂等刷新、并发与失败记录由 Core 负责，不承诺分钟级准时。
2. 新文章通知支持 Android 13+ 权限、渠道、开关、夜间免打扰、批量汇总；拒绝权限不影响刷新。
3. FreshRSS/Miniflux 同步订阅、文件夹、已读和收藏，支持手动触发与低频后台触发。
4. 定义 provider-neutral `SyncPort` 的 capabilities/pull/push/status，使用 Fake Provider 覆盖 cursor、ack、remote mapping、tombstone 和重试。
5. 本地未确认变化不可被远端拉取覆盖；进程终止后从持久化队列继续。
6. 同步凭据只存 Keystore；清除全部数据必须同时清数据库、缓存和 Papr 凭据。
7. 不建设托盘式常驻调度、桌面通知/Dock 角标、代理/并发高级面板或 Papr 云账号。

## Acceptance Criteria

- [ ] 后台任务重复运行与进程终止恢复均幂等，不产生重复文章或丢失本地变更。
- [ ] 通知权限拒绝、夜间免打扰和批量新文章行为符合设置。
- [ ] FreshRSS/Miniflux 在断网、部分失败、重复 pull/push 下保持本地意图。
- [ ] Fake Provider 覆盖 pull、push、ack、cursor、tombstone 与失败重试。
- [ ] 凭据泄漏与清除数据回归通过；全量阶段门禁和 Debug APK 通过。

## Dependencies / Out of Scope

依赖 P5 归档。桌面 `scheduler.rs`、`sync.rs`、`notify.rs` 只提供业务语义参考，不直接移植生命周期或凭据存储方式。
