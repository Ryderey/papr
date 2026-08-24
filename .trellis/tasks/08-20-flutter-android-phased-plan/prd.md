# Flutter Android 精简分阶段开发（P0-P7）

## Goal

以 `docs/desktop-mobile-porting-feature-analysis.md` 为范围判断依据，按更新后的 `docs/flutter-android-phased-development-plan.md` 推进 P0→P7，把 Flutter Android 端从「可用性验证」提升到「可进入 Google Play 内测的 Release Candidate」，不追求桌面交互与半成品功能的数量对齐。

父任务持有源需求、子任务映射与跨阶段验收规则；实现落到各子任务，本任务不直接实现。

## Source Requirements

范围判断来源：`docs/desktop-mobile-porting-feature-analysis.md`。执行与验收来源：`docs/flutter-android-phased-development-plan.md`（2026-08-24 修订）。子任务不得重新纳入“明确不做”清单。

## 固定架构方向（每项功能必须遵守）

```text
桌面成熟逻辑 → papr-core → Tauri / FRB 薄适配器 → Flutter Repository/Riverpod → UI
```

- `papr-core` 是数据库、业务规则、网络协议、同步语义的唯一来源；不依赖 Tauri/Flutter/Dart/平台能力。
- Tauri 与 Flutter 适配器只做参数、DTO、错误、事件转换。
- Flutter Widget 只依赖 Repository/Provider，不直接调 FRB 生成 API。
- SQLite 与 HTTP 留在 Rust 层；Dart 不引入第二套数据库/网络业务实现。
- 新依赖仅用于 Android 原生能力：文档选择、系统分享/浏览器、Keystore、后台任务、通知、媒体会话。

## 子任务映射

| 子任务 | 标题 | 交付物 |
|--------|------|--------|
| P0 | 统一核心与正式数据基线 | 完整 schema 收敛、Alpha schema、类型化错误码、统一服务初始化、事务变更日志 |
| P1 | 应用框架、订阅源与文件夹 | 自适应导航、i18n/主题、文件夹与 Feed CRUD、OPML、发现与深链 |
| P2 | 文章列表、智能视图与完整阅读闭环 | 智能视图/计数/分页/搜索、已读/收藏/稍后读、全文提取、阅读设置、分享 |
| P3 | 标签、规则、高亮与个人整理 | 标签/规则/高亮 CRUD 与同步变更日志、文章清理保留策略 |
| P4 | 翻译与 AI 摘要 | AI/翻译/SSE 下沉 core、BYOK、Keystore 存 Key、摘要与翻译引擎 |
| P5 | Podcast 与 Android 原生播放 | Podcast 播放器、MediaSession、前台服务、锁屏/蓝牙控制 |
| P6 | 后台刷新、通知与外部同步 | WorkManager、通知、FreshRSS/Miniflux、SyncPort、凭据安全、最小重置/清除能力 |
| P7 | 功能矩阵验收与可发布 RC | 功能矩阵、无障碍/窄屏、多设备、Release AAB、隐私说明 |

P0–P2 已归档；P3 已提交主体实现但仍需补齐未解析锚点提示、Widget 测试矩阵和实体机证据。P4–P7 使用本轮新建子任务，执行顺序固定，每阶段独立验证。

## 明确排除

- 全部桌面窗口/托盘/快捷键/自动更新/子 WebView 能力。
- hover、右键、桌面拖拽、命令面板、专注模式、滚到底自动已读、YouTube iframe。
- Send to Kindle、Ask/RAG、Digest、Newsletter/IMAP、实验去重。
- RSSHub 自建配置与桌面式高级存储/网络运维面板。

## 跨阶段验收规则

- 每个实现任务先读本计划文档、相关代码、当前 Trellis 任务材料。
- 记录该任务修改的公共 DTO、数据库迁移、事件契约。
- 先 `papr-core` 完成业务能力，再暴露 FRB，再接 Flutter。
- 更新/新增最小但真实的回归测试。
- 重新生成并提交 FRB 绑定，检查无非预期生成差异。
- 运行第 7 节质量门禁；无法执行的检查必须在交接中说明原因，不能写成已通过。
- 高风险场景必须有回归：重复订阅、迁移中断、断网、部分刷新失败、长正文、恶意 HTML、同步重放、规则批量执行、AI 流中断、凭据删除、后台任务重复唤醒。

## 质量门禁（每个阶段）

```powershell
cargo test -p papr-core
cargo check -p papr-flutter-bridge

Set-Location mobile
flutter test
flutter analyze
flutter build apk --debug
```

涉及桌面适配时追加：`cargo test -p papr`、`pnpm test`、`pnpm build`。P6 起增加 Release AAB 构建与真实设备验证。

## Acceptance Criteria（父任务层面）

- [ ] P0–P7 精简范围的全部子任务完成并归档，每个子任务的退出条件都有代码/测试/真实设备证据。
- [ ] Rust、Flutter、FRB、桌面回归全部通过。
- [ ] 从正式 Alpha schema 连续升级到 RC，未再次依赖清库。
- [ ] 无阻塞级崩溃、数据丢失、密钥泄露、迁移问题。
- [ ] Release AAB 可安装、升级、启动并完成订阅到阅读全链路。

## Out of Scope

- `docs/desktop-mobile-porting-feature-analysis.md` 档位一、档位三中标记“砍/不照搬”的桌面能力，以及档位四低价值半成品。
- iOS 保持可移植性，但不进入本计划验收矩阵。
- Google Play 商店运营材料不属于本计划。
