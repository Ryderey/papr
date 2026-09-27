# Implement: P6 后台刷新、通知与外部同步

1. [x] 冻结后台 job、通知、SyncPort、队列、凭据和错误契约。
2. [x] 将 due-feed 与 FreshRSS/Miniflux 逻辑下沉 Core，统一 change_log/游标/映射。
3. [x] 用 Fake Provider 完成重放、冲突、ack、tombstone 和崩溃恢复测试。
4. [x] 暴露 FRB 手动刷新/同步/状态 API，先完成 Flutter 前台闭环。
5. [x] 接入 Keystore、WorkManager、通知权限/渠道/汇总与唯一任务。
6. [x] 实现最小设置、错误重试、重置与清除数据 UI。
7. [ ] 运行全量门禁并执行断网、Doze、杀进程、重复唤醒和权限拒绝设备测试。

## Progress — 2026-09-27

- P6A and P6B are archived. Their code supplies the Core due-refresh and
  GReader sync paths, one Android Worker, notification controls, isolated
  Keystore aliases, and Fake Provider replay tests.
- Settings now has confirmed preference reset and confirmed Android app-data
  clear. Preference reset uses existing typed setters; app-data clear delegates
  to `ActivityManager.clearApplicationUserData()` so the system owns database,
  cache, preference, and credential cleanup.
- Checks: Core 101 tests, FRB cargo check, Flutter 32 tests, Flutter analyzer,
  and Debug APK build passed. The bridge still emits existing `frb_expand`
  warnings.
- After the channel error regression was added, the full Flutter suite passed
  32 tests and `flutter analyze --no-pub` remained clean. A Release AAB build
  also passed at `mobile/build/app/outputs/bundle/release/app-release.aab`
  (93.2 MB, SHA-256 `AD77EE7848B6423C020F08166E7EE3E87634CAF20DBA9F076CCD2ACD82BB361D`).
  The current Gradle release configuration still signs with debug keys, so
  this is a build gate only, not a distributable RC artifact.
- The Debug APK installed over the existing `emulator-5554` data, launched as
  process 8041, and JobScheduler still showed one Papr `BackgroundWorker` job.
- With explicit user authorization, Settings > Clear all data was confirmed on
  `emulator-5554`. Before clear, `papr.db` was 8,122,368 bytes, there were 358
  articles, and `papr_ai_credentials.xml` existed. The process exited and both
  the database and `shared_prefs` directories disappeared. After relaunch the
  article count was zero, a fresh database was created, the credential file
  remained absent, language and refresh interval reverted to English/30 min,
  and notification app-op was `ignore`. No prior app cache survived; fresh
  Flutter/runtime cache files were created at launch.
- Preference reset was also exercised on the emulator: Dark theme and disabled
  background refresh returned to System and enabled after confirmation.
- A live IT Home RSS subscription fetched 60 articles on the emulator after
  network access returned. Denying the Android notification prompt kept the
  notification switch off and showed the localized denial message; granting
  it on the next attempt enabled the switch and quiet-hours control while
  background refresh stayed on. The interval was set to 15 minutes for a
  subsequent due-feed device run.
- After the foreground process was killed, a forced run of the namespaced
  JobScheduler entry started Papr in a new process. WorkManager logged
  `Worker result SUCCESS` at 04:48:09 and registered one next-period job.
  The feed was added at 04:36, so this run was before its 15-minute due time;
  it does not prove new-article notification or quiet-hour suppression.
- A second forced JobScheduler run after the interval elapsed completed with
  `SUCCESS` at 05:05:43 and again left one next-period job. The article list
  still held 60 items, and no Papr notification was active, so the required
  nonzero-new-article notification and quiet-hour suppression remain unproven.
- Remaining acceptance: observe a due new-article notification and quiet-hour
  suppression after process reclaim; exercise live FreshRSS and Miniflux
  accounts when credentials are available. Keep this parent task open until
  device and release checks have evidence.

## Rollback

后台触发可独立关闭并保留手动同步；远端 Provider 可按 capabilities 禁用，不删除本地 change_log。
