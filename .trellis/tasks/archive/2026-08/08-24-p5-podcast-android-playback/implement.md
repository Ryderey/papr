# Implement: P5 Podcast 与 Android 原生播放

1. [x] 冻结 enclosure DTO、播放状态机、平台命令和错误码。（复用已存在的 `ArticleDetail.enclosures`；服务是唯一播放状态源，Media3 的 MediaSessionService 负责外部控制与通知。）
2. [ ] 补齐 Core/FRB 的 Podcast/enclosure 查询与测试。
3. [x] 选择最小播放器方案并实现 Android MediaSession/前台服务/通知。（采用官方 Media3 ExoPlayer + MediaSessionService；服务独占 Player、Audio Focus、系统媒体控制与默认媒体通知，Flutter 只发命令并订阅快照。）
4. [x] 实现 Flutter 文章入口、迷你播放器和完整控制页。（仅将 MIME 为 `audio/*` 或常见音频扩展名的 enclosure 作为 Podcast 入口；普通附件仍走原有外部打开。迷你播放器、全屏进度/跳转/倍速控制都订阅同一原生状态流。）
5. [ ] 覆盖 Audio Focus、noisy、网络失败、重连和进程恢复测试。
6. [x] 运行当前阶段的静态检查、Flutter 全量测试与 Debug APK 构建。
7. [ ] 用真实 Podcast 与蓝牙/锁屏完成设备验收。

## Rollback

先关闭 Flutter 播放入口，再移除服务注册和桥接；Core enclosure 数据保持兼容。
