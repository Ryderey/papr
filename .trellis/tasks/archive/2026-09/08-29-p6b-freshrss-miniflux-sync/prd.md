# P6B FreshRSS 与 Miniflux 同步

## Goal

在 Android 上以 FreshRSS 与 Miniflux 共用的 GReader 协议同步订阅、文件夹、已读和星标状态；同步逻辑归 Core 所有，凭据仅保存在 Android Keystore。

## Requirements

1. Core 定义 provider-neutral `SyncPort`、稳定同步 DTO 与 Fake Provider；FreshRSS 和 Miniflux 仅通过各自的 GReader API 根路径差异实现该端口。
2. 同步连接配置只持久化服务类型、服务器 URL、用户名、凭据引用、最近成功时间和稳定错误码；密码、应用密码和长期 token 不得写入 SQLite、日志、FRB DTO 或用户可见错误。
3. Android 凭据使用既有 Keystore 机制的独立 `papr.sync.*` 命名空间；AI 的 `papr.ai.*` 凭据保持兼容且不能互相读取或删除。
4. Flutter 设置页提供 FreshRSS/Miniflux 选择、连接检测、连接/断开、立即同步、重试与最近状态。连接成功后才保存非秘密配置；失败时不替换既有连接。
5. 同步顺序为：确认连接 → 推送并确认本地已读/星标队列 → 拉取远端文件夹和订阅 → 拉取文章状态。推送或拉取失败时保留待重试本地变更；未获确认的本地变更不得被远端状态覆盖。
6. 订阅和文件夹采用非破坏性合并：缺失项可补入对端；不因远端缺失自动删除本地内容，也不因本地删除自动取消远端订阅。本阶段仅在 `SyncPort`/Fake Provider 中验证 tombstone 与确认语义，为未来 Papr 多端同步保留合同。
7. 手动同步是首个用户入口。后台刷新只在存在有效连接且达到低频同步窗口时调用同一 Core 用例；不得在每次前台刷新后立即同步，也不得创建第二个 Android Worker。
8. 连接断开、无网络、认证失效、部分 provider 失败和重复执行必须有稳定结果，且不丢失本地已读/星标操作。

## Out of Scope

- Papr 账号、云服务、OAuth、WebDAV、OPML 云同步、协作冲突 UI、桌面高级同步面板。
- 无确认的双向删除、同步标签/规则/高亮/文章正文、分钟级自动同步、把密码或 token 迁入数据库。

## Acceptance Criteria

- [x] Fake Provider 覆盖 pull、push、ack、cursor、tombstone、断网和部分失败重试；同一批次重复运行保持幂等。
- [x] Core 队列在 push 失败后保持可重试；远端 pull 不覆盖未确认的本地已读/星标变更。
- [x] FreshRSS 与 Miniflux 的 URL 规范化、认证、订阅/文件夹合并和文章状态同步共享同一 `SyncPort` 合同。
- [x] Flutter 可安全连接、显示状态、手动同步、重试与断开；无连接或凭据丢失时给出可操作提示。
- [x] Android Keystore 中的同步凭据不会与 AI 凭据冲突；断开连接删除对应同步凭据，数据库只保留非秘密状态。
- [x] P6A 的唯一 WorkManager 任务在启用同步后仍只有一个，且低频同步不影响普通 Feed 刷新和通知。
- [x] Core、FRB、Flutter 测试与 Debug APK 构建通过；至少使用 Fake Provider 完成一次可重复的端到端同步验收。
