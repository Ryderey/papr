# Design: P5 Podcast 与 Android 原生播放

## Boundary

Core 提供 enclosure/来源 DTO 与持久化阅读数据；Flutter 持有展示状态；单一 Android 播放服务拥有播放器、MediaSession 和通知。Flutter 与服务通过一个明确的命令/状态通道通信。

## State Model

状态至少包含 media ID、URL、标题、来源、duration、position、speed、playing、buffering 与 error。命令幂等，服务重连后先返回权威快照；UI 不自行推算第二份长期状态。

## Lifecycle and Security

只有用户发起播放后启动前台服务；停止并清空队列后退出。仅允许 HTTP(S) 音频 URL，不记录带敏感查询参数的完整 URL。

## Rollback

移动 UI 可退回外部播放器打开；不需要数据库迁移回滚。
