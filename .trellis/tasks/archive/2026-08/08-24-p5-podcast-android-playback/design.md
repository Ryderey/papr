# Design: P5 Podcast 与 Android 原生播放

## Boundary

现有 `ArticleDetail.enclosures` 已经由 Core/FRB 传递给 Flutter，P5 不增加数据库迁移或新的 enclosure DTO。Flutter 只从文章详情选择 HTTP(S) 音频 enclosure；单一 Android 播放服务拥有播放器、MediaSession、Audio Focus 与媒体通知，Flutter 不创建第二个播放器。

Android 使用官方 AndroidX Media3 `media3-exoplayer` 与 `media3-session`（当前稳定版 1.10.1）。平台 `MediaPlayer` 虽可播放单一 URL，但不能以同等、跨版本的方式提供 `MediaSessionService`、外部控制器连接和自动媒体通知；不引入 Dart 播放器依赖，也不改变 Core。

## State Model

状态至少包含 media ID、标题、来源、duration、position、speed、playing、buffering 与 error。完整 URL 只在 Android 服务内存中使用，不出现在状态事件、日志或错误文字中。服务在播放期间每 500ms 发布一次位置快照；命令幂等，Flutter 服务重连后先返回权威快照；UI 不自行推算第二份长期状态。

Flutter 通过 `com.papr.papr_mobile/platform` 发送 `startPlayback`、`play`、`pause`、`seekTo`、`skipBack`、`skipForward`、`setSpeed`、`stop` 和 `getPlaybackState`。服务经独立的 playback 事件通道推送权威快照；每次 Flutter 订阅先收到当前快照。稳定错误码为 `invalidPlaybackUrl`、`playbackUnavailable`、`playbackNetwork` 与 `playbackFailed`。

## Lifecycle and Security

只有用户发起播放后启动 `MediaSessionService`；停止并清空队列后释放 Player/Session。Manifest 声明 `FOREGROUND_SERVICE`、`FOREGROUND_SERVICE_MEDIA_PLAYBACK` 和服务的 `mediaPlayback` 类型。仅允许 HTTP(S) 音频 URL，不记录带敏感查询参数的完整 URL。Android 13 的通知授权不阻止服务启动，服务仍遵循系统前台服务通知要求。

## Rollback

移动 UI 可退回外部播放器打开；不需要数据库迁移回滚。
