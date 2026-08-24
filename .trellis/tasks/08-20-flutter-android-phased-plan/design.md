# Design: Flutter Android 精简分阶段开发

## 1. 范围决策

使用三类边界管理后续工作：平台无对应物与低价值半成品直接排除；数据/内容能力下沉 `papr-core`；后台、媒体、通知与凭据改用 Android 原生机制。桌面端只作为业务语义参考，不作为 UI 或运行时实现模板。

## 2. 阶段结构

- P0–P2 保留已归档结果。
- P3 保留当前实现并只完成差距与设备验收，不扩展桌面式高亮交互。
- P4 只交付翻译和 AI 摘要，排除 Ask/RAG/Digest。
- P5 只交付 Podcast 与原生后台播放，排除 Newsletter/IMAP。
- P6 交付 WorkManager 刷新、通知、FreshRSS/Miniflux 与最小安全运维能力。
- P7 按移动端范围矩阵和 Google Play 内测条件收口。

## 3. 架构边界

业务、协议、SQLite 和事务留在 `papr-core`；FRB/Tauri 保持薄适配；Flutter 通过 Repository/Provider 使用能力；Android 平台层只承接 Keystore、WorkManager、通知、MediaSession、SAF、分享与深链。

## 4. 范围防回流

每个子任务 PRD 必须引用本父任务的“明确排除”清单。发现桌面能力时先分类为直接排除、Android 重建或 Core 下沉；未经父任务范围修订，不得临时加入 RC。

## 5. 兼容与回滚

保持现有 schema 的编号迁移和 FRB 生成契约。P4、P5、P6 分别以 Core、桥接、Flutter/Android 三层提交，任何阶段可按层回滚，不回退 P0–P3 数据。
