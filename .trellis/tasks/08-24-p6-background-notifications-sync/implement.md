# Implement: P6 后台刷新、通知与外部同步

1. [ ] 冻结后台 job、通知、SyncPort、队列、凭据和错误契约。
2. [ ] 将 due-feed 与 FreshRSS/Miniflux 逻辑下沉 Core，统一 change_log/游标/映射。
3. [ ] 用 Fake Provider 完成重放、冲突、ack、tombstone 和崩溃恢复测试。
4. [ ] 暴露 FRB 手动刷新/同步/状态 API，先完成 Flutter 前台闭环。
5. [ ] 接入 Keystore、WorkManager、通知权限/渠道/汇总与唯一任务。
6. [ ] 实现最小设置、错误重试、重置与清除数据 UI。
7. [ ] 运行全量门禁并执行断网、Doze、杀进程、重复唤醒和权限拒绝设备测试。

## Rollback

后台触发可独立关闭并保留手动同步；远端 Provider 可按 capabilities 禁用，不删除本地 change_log。
