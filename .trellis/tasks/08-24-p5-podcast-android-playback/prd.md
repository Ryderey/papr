# P5 Podcast 与 Android 原生播放

## Goal

为 Podcast enclosure 提供符合 Android 生命周期的音频体验，包括后台播放、MediaSession、前台服务、通知和锁屏/蓝牙控制。

## Requirements

1. 识别现有 Podcast 来源和 audio enclosure，普通 RSS 阅读不受播放器失败影响。
2. 支持播放/暂停、进度、拖动、后退 15 秒、前进 30 秒、0.75–2× 倍速和跨文章持续播放。
3. Android 原生层承接 MediaSession、前台服务、媒体通知、Audio Focus 和 noisy intent；Core 不调用 Android API。
4. 页面、迷你播放器、通知、锁屏和蓝牙按钮共享一个播放状态来源。
5. 覆盖断网、无效 URL、损坏媒体、来电/其他音频抢占和进程恢复。
6. 明确排除 Newsletter/IMAP、Send to Kindle 与 YouTube iframe。

## Acceptance Criteria

- [ ] 真实 Podcast 样本可发现 enclosure 并播放，损坏 enclosure 不阻塞 Feed。
- [ ] 锁屏、后台、通知和蓝牙控制状态一致，Audio Focus 行为符合 Android 规范。
- [ ] 文章切换、旋转、进程重建和网络恢复不产生双播放器或错误进度。
- [ ] Android 10、一个中间版本和当前稳定版本完成实体机/模拟器矩阵。
- [ ] Core/Bridge/Flutter 检查与 Debug APK 通过。

## Dependencies / Out of Scope

依赖 P4 归档。若需要播放器依赖，实施前必须证明现有依赖和原生标准 API 不足，并记录体积与维护成本。
