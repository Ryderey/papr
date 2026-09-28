# Android RC scope matrix

Status as of 2026-09-28. “Implemented” means a code path exists; it does not imply the deferred acceptance checks passed. The product boundary follows [the phased plan](flutter-android-phased-development-plan.md) and the archived P0–P6 tasks.

| Area | Implemented mobile path | Android replacement / RC boundary | Evidence |
| --- | --- | --- | --- |
| Subscriptions | Feed and folder management, bundled directory discovery, OPML import/export, deep links, manual and due refresh | Storage Access Framework for OPML; WorkManager wakes Core for due refresh | `mobile/lib/ui/screens/feed_list_screen.dart`, `mobile/lib/services/background_refresh_service.dart`, `crates/papr-core/src/services/feed.rs` |
| Reading | Smart views, search, article state, sanitized HTML, full-text extraction, reading controls and system sharing | Flutter reader and Android share/browser intents replace desktop hover and child WebView | `mobile/lib/ui/screens/article_browser_screen.dart`, `mobile/lib/ui/screens/article_detail_screen.dart`, `crates/papr-core/src/services/article.rs` |
| Organization | Tags, rules, highlights and protected article state | Touch selection and mobile management screens replace drag/right-click workflows | `mobile/lib/ui/screens/organization_screen.dart`, `mobile/lib/ui/screens/highlights_screen.dart`, `crates/papr-core/src/db.rs` |
| AI and translation | Profile connection, streamed summaries, summary-local follow-up, LLM translation and successful-result caches | Credential material in Android Keystore; follow-up is limited to the current summary, with no global Ask/RAG | `mobile/lib/ui/screens/ai_summary_screen.dart`, `mobile/lib/ui/screens/ai_translation_screen.dart`, `crates/papr-core/src/services/ai.rs` |
| Audio | Podcast playback and controls | Media3 `MediaSessionService` and foreground media playback replace HTML audio | `mobile/android/app/src/main/kotlin/com/papr/papr_mobile/PlaybackService.kt`, `mobile/lib/ui/screens/playback_screen.dart` |
| External sync | FreshRSS/Miniflux GReader sync and durable Core change log | Manual and low-frequency WorkManager sync; request-time Keystore credential | `crates/papr-core/src/services/sync.rs`, `crates/papr-core/src/sync/greader.rs`, `mobile/lib/repositories/sync_repository.dart` |
| Settings | Language, theme, reading, refresh, notifications, preference reset and confirmed local-data clear | Android notification permission/channel; Android app-data clear | `mobile/lib/ui/screens/settings_screen.dart`, `mobile/lib/services/platform_service.dart` |

The following desktop capabilities remain excluded from this RC: tray/window/taskbar behavior, startup and in-app updater, desktop shortcuts/command palette, hover/right-click/desktop drag, focus mode, scroll-to-mark-read, embedded YouTube iframe, Send to Kindle, global Ask/RAG, Digest, Newsletter/IMAP, experimental deduplication, custom RSSHub instance setup, and advanced proxy/concurrency/storage maintenance panels. Basic RSSHub URL support remains. This is the exclusion decision in [the phased plan](flutter-android-phased-development-plan.md#11-明确不做), not a claim that every desktop command has been ported.

## Deferred acceptance

The checks below are tracked in [Android deferred acceptance](../.trellis/tasks/09-28-mobile-deferred-acceptance/prd.md). They do not block the current self-use internal RC.

- Localization keys are present in English, Chinese and Japanese; translated wording, TalkBack, font scaling, contrast, reduced motion, dark mode and touch targets still need review. The AI profile delete action now has a localized accessibility label and a narrow-screen Japanese Widget test.
- API 37 `Medium_Phone` simulator runs cover a test-package upgrade, cold start, retained language/theme and settings appearance. Android 10, an intermediate API level and a tablet simulator still need runs; physical-device testing is outside the current self-use scope.
- Cold start, large data, long article/audio and background recovery still need recorded results. Core v16 Alpha schema migration has an automated data-retention test; formal Alpha APK upgrade is outside scope.
- The internal RC APK build, matching-certificate version 1→2 emulator installation and cold launch are recorded in the [RC procedure](mobile-rc-release.md). Temporary AI profile deletion removed its private credential preference entry. In-app data clear ended the process and relaunched with default language and no AI profiles. Direct database-file and Keystore-alias checks, sync credential deletion and the full simulated flow remain unverified.
