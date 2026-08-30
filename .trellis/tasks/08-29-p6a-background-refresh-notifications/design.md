# Design: P6A 后台刷新与新文章通知

## Boundaries

- `papr-core` owns due-feed selection, conditional refresh, deduplication, persisted settings, per-feed failure recording, and the refresh report.
- FRB only converts the typed settings and refresh DTOs.
- A headless Dart background entry point initializes plugins, FRB, and `PaprCore` with the same application documents path as the foreground process, then invokes `refresh_feeds(force: false)`.
- AndroidX WorkManager scheduling is reached through the Flutter `workmanager` adapter. Local notification display and Android 13+ permission are reached through `flutter_local_notifications`.
- Flutter settings own user intent and reschedule/cancel the unique periodic task after Core confirms a settings write.

## Core Contracts

`Db::feeds_due_for_refresh(global_min)` returns non-newsletter feeds whose effective interval has elapsed. `REFRESH_OFF_MINUTES = 525_600` is shared by global and per-feed settings. Invalid stored global values fall back to 30 minutes; live values are clamped to 5–120 for compatibility with desktop data.

`IngestionService::refresh_feeds` selects feeds as follows:

- `force=true`: all feeds, optionally filtered by `feed_ids`.
- `force=false`: due feeds, optionally filtered by `feed_ids`.

The existing `RefreshReport { total_feeds, new_articles, errors }` is sufficient for P6A. WorkManager provides run identity/retry state, and article/feed uniqueness already provides persistent idempotency, so P6A does not add a second job table or lease protocol.

`SettingsSnapshot` gains typed background settings:

- `refresh_interval_min`: existing global setting; the off sentinel means disabled.
- `notifications_enabled`: default `false` on mobile.
- `notification_quiet_hours`: default `false`, fixed 22:00–08:00 window.

Settings writes are validated and persisted by Core. Notification permission state remains platform-owned and is not stored as a Core preference.

## Background Data Flow

1. Foreground startup initializes WorkManager and reads Core settings.
2. If auto refresh is enabled, Flutter registers one periodic task named `papr.background.refresh` with frequency `max(interval, 15 minutes)`, network connectivity required, and update semantics. If disabled, it cancels that unique task.
3. When Android wakes the task, the background dispatcher registers plugins, initializes `RustLib`, builds the same Core config, and opens the same SQLite database.
4. It calls Core with `RefreshOptions(feed_ids: null, force: false)`.
5. A successful Core report with `new_articles > 0` is checked against typed notification preferences, platform permission, and local quiet hours. The adapter displays one summary notification when all checks pass.
6. Network/platform setup failures request WorkManager retry; completed Core reports, including per-feed errors, finish successfully so one bad feed does not create an endless global retry loop.

## Concurrency and Idempotency

WorkManager unique periodic work prevents duplicate schedules. Foreground and background Core instances may briefly overlap, but SQLite WAL plus the existing `(feed_id, guid)` article uniqueness prevents duplicate articles, while conditional requests reduce redundant payloads. A persistent cross-instance refresh lease is intentionally deferred: it adds crash-expiry and ownership complexity without protecting user data beyond existing database invariants. Add one only if device logs show harmful concurrent refresh load.

## Notifications

- Channel ID: `papr_new_articles`; channel importance is default when sound is enabled and low when disabled only if Android channel behavior permits a stable choice. Because Android channel sound is user-controlled after creation, P6A exposes notification enablement and quiet hours, not an app-level sound toggle.
- Summary ID is stable so a later batch replaces the previous Papr summary instead of stacking many notifications.
- Text comes from existing Flutter localization resources.
- Tapping the notification opens the application; notification actions and deep routing are out of scope.

## Compatibility and Rollback

- Existing databases require no schema migration because settings remain in the existing key/value table and feed timing columns already exist.
- Existing manual refresh calls already send `force=true` and must remain unchanged.
- The WorkManager and notification adapters can be disabled or removed without changing stored articles or subscription settings.
- Removing P6A requires cancelling `papr.background.refresh`; Core due selection can remain because it is backward-compatible and testable independently.

## Dependency Decision

Add `workmanager` and `flutter_local_notifications`. The first is the maintained Flutter wrapper around AndroidX WorkManager and supplies the headless Dart isolate needed to reuse FRB/Core. The second is required because `MainActivity` method channels are unavailable in that headless engine. A custom JNI Worker was rejected because it would add a second Rust entry ABI, JNI string/error plumbing, and a separate Tokio runtime for behavior already supported by these adapters.
