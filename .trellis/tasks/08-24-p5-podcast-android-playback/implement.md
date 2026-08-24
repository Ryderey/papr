# Implement: P5 Podcast 与 Android 原生播放

1. [ ] 冻结 enclosure DTO、播放状态机、平台命令和错误码。
2. [ ] 补齐 Core/FRB 的 Podcast/enclosure 查询与测试。
3. [ ] 选择最小播放器方案并实现 Android MediaSession/前台服务/通知。
4. [ ] 实现 Flutter 文章入口、迷你播放器和完整控制页。
5. [ ] 覆盖 Audio Focus、noisy、网络失败、重连和进程恢复测试。
6. [ ] 运行全量阶段门禁并构建 Debug APK。
7. [ ] 用真实 Podcast 与蓝牙/锁屏完成设备验收。

## Rollback

先关闭 Flutter 播放入口，再移除服务注册和桥接；Core enclosure 数据保持兼容。
