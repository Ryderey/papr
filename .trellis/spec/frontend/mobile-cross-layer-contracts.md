# Mobile Cross-Layer Contracts

## 1. Scope / Trigger

Use this contract when a mobile subscription or reading feature crosses SQLite, `papr-core`, Flutter Rust Bridge (FRB), Flutter repositories, and Android platform APIs. Business rules belong in `papr-core`; FRB and platform channels remain thin adapters.

## 2. Signatures

The bridge surface is defined in `crates/papr-flutter-bridge/src/api.rs`. P1 subscription APIs are:

```rust
add_feed(core, AddFeedInput { input }) -> Result<Feed, PaprBridgeError>
list_folders(core) -> Result<Vec<Folder>, PaprBridgeError>
create_folder(core, name) -> Result<i64, PaprBridgeError>
rename_folder(core, id, name) -> Result<(), PaprBridgeError>
delete_folder(core, id) -> Result<(), PaprBridgeError>
reorder_folders(core, ordered_ids) -> Result<(), PaprBridgeError>
delete_feed(core, id) -> Result<(), PaprBridgeError>
rename_feed(core, id, title) -> Result<(), PaprBridgeError>
move_feed(core, id, folder_id) -> Result<(), PaprBridgeError>
set_feed_refresh_interval(core, id, minutes) -> Result<(), PaprBridgeError>
refresh_feed(core, id, options) -> Result<RefreshReport, PaprBridgeError>
import_opml(core, text) -> Result<OpmlImportReport, PaprBridgeError>
export_opml(core) -> Result<String, PaprBridgeError>
search_directory(query, lang) -> Vec<DiscoveryResult>
parse_deep_link(url) -> Option<String>
```

P2 reading APIs are:

```rust
list_articles(core, ArticleFilter) -> Result<Vec<ArticleSummary>, PaprBridgeError>
count_articles(core, ArticleFilter) -> Result<i64, PaprBridgeError>
get_article_counts(core) -> Result<ArticleCounts, PaprBridgeError>
list_article_tags(core) -> Result<Vec<TagSummary>, PaprBridgeError>
set_article_read/core, set_article_starred, set_article_read_later(core, id, value)
mark_all_articles_read(core, ArticleFilter) -> Result<i64, PaprBridgeError>
extract_article_fulltext(core, id) -> Result<ArticleDetail, PaprBridgeError>
set_reading_settings(core, ReadingSettings) -> Result<(), PaprBridgeError>
```

P3 organization APIs are:

```rust
list_article_tags(core) -> Result<Vec<TagSummary>, PaprBridgeError>
create_tag(core, name) -> Result<i64, PaprBridgeError>
rename_tag(core, id, name) -> Result<(), PaprBridgeError>
set_tag_color(core, id, color) -> Result<(), PaprBridgeError>
reorder_tags(core, tag_ids) -> Result<(), PaprBridgeError>
delete_tag(core, id) -> Result<(), PaprBridgeError>
set_article_tag(core, article_id, tag_id, attached) -> Result<(), PaprBridgeError>
list_rules(core) -> Result<Vec<Rule>, PaprBridgeError>
create_rule/core, update_rule(core, RuleInput) -> Result<_, PaprBridgeError>
delete_rule(core, id) -> Result<(), PaprBridgeError>
preview_rule(core, RuleInput) -> Result<RulePreview, PaprBridgeError>
apply_rule_to_existing(core, RuleInput) -> Result<i64, PaprBridgeError>
list_highlights/core, list_all_highlights(core) -> Result<Vec<Highlight>, PaprBridgeError>
create_highlight(core, HighlightInput) -> Result<i64, PaprBridgeError>
update_highlight_note/core, set_highlight_color/core, delete_highlight(core, ...) -> Result<(), PaprBridgeError>
resolve_highlights(core, article_id, text) -> Result<Vec<ResolvedHighlight>, PaprBridgeError>
```

Android channel: `com.papr.papr_mobile/platform`, with `getInitialDeepLink`, `openOpmlDocument`, `saveOpmlDocument`, `openUrl`, `shareArticle`, and the pushed `deepLink` event.

## 3. Contracts

- `Feed` carries `folder_id`, `source_type`, `custom_title`, and `refresh_interval_min` across every layer; do not reconstruct them in Flutter.
- `Folder.position` is authoritative. Reordering must submit the complete folder ID set.
- Add-feed normalization, discovery, first fetch, parsing, classification, and persistence execute in Core. Feed, articles, and change-log rows commit in one transaction.
- Folder/feed writes use `Db::transact` and append `folder`/`feed` change-log entries in the same transaction.
- Deleting a folder preserves feeds with `folder_id = NULL`; deleting a feed relies on database cascades for its articles.
- OPML transport is text. Android uses Storage Access Framework document intents and requests no broad storage permission.
- A supported deep link is `papr://subscribe?url=<encoded>`; invalid or unsupported links return `None` and do not open the add-feed flow.
- `ArticleFilter` is the single query contract for list, count, and bulk-read operations. It carries exactly one view kind plus optional ID/search/unread/order/limit/offset values.
- Core bounds every article page to 1-200 rows (default 50), builds FTS terms from safe alphanumeric tokens, and applies deterministic effective-date then ID ordering.
- Article read/star/read-later writes and their `article` change-log rows commit in one transaction. Repeating the same value is idempotent.
- Fulltext extraction fetches and sanitizes in Core. Persisting extracted HTML updates the FTS row and lead image and clears stale translation fields atomically; a failed extraction preserves existing cached content.
- Reading settings are validated and persisted as one Core settings update. Flutter may preview controls locally, but Core remains authoritative after save or failure.
- Android `openUrl` and `shareArticle` accept only HTTP(S) article URLs. Flutter disables these actions when no usable URL exists.
- Tag positions are authoritative and a reorder submits the complete tag ID set. Tag, article-tag, rule, and highlight mutations append their change-log rows in the same Core transaction; repeating the same article-tag association is idempotent.
- Rule enable/disable keeps an in-screen optimistic override keyed by rule ID and disables that switch while its write is in flight. On success, invalidate and await `rulesProvider` reload; on failure, remove the override and show the localized stable error so no stale toggle remains visible.
- `apply_rule_to_existing` returns the number of articles whose state actually changed, while `preview_rule` returns all matches. A zero apply result can therefore mean every matching article already had the target state. Regardless of that count, a successful apply invalidates every `articlePageProvider` family instance plus `articleCountsProvider` and `articleCountProvider`, so Starred/Read Later lists cannot retain stale projections.
- The reader exposes no per-article tag editor. Its star icon maps to `isStarred` / the Starred smart view, and its bookmark icon maps to `readLater` / the Read Later smart view. After a successful state write, invalidate `articleDetailProvider(articleId)`, every `articlePageProvider` family instance, and the article count providers so reopening the article or subscription cannot reuse stale flags.
- Rule matching is centralized in Core and shared by ingestion, preview, and apply-to-existing. Matching is Unicode case-insensitive, comma-separated terms are ORed, SQL wildcard characters remain literal, and enabled rules run in position order.
- A `skip` rule may remove only disposable existing articles. Starred, read-later, or highlighted articles are retained; `read` and `star` actions use the same article-state/change-log transaction contract as direct writes.
- Highlight offsets use UTF-16 code units so Flutter selections and Core anchors agree. Resolution tries the stored offset first, then quote plus prefix/suffix context, and finally the first quote match. An unresolved anchor remains a valid editable record.
- Flutter must pass the reader's DOM text-node sequence to `resolve_highlights`, then render each returned resolved range by safely inserting `<mark class="papr-highlight" data-highlight-color="…">` into parsed HTML text nodes. Never use a regular expression to alter raw article HTML. Unresolved records stay in the highlight list and display a localized notice.

## Scenario: Android background refresh and new-article notification

### 1. Scope / Trigger

- Trigger: Android wakes Papr without a visible Flutter route to refresh due
  subscriptions and optionally notify about newly inserted articles.
- Why: feed timing, conditional fetches, deduplication, and partial-error
  handling must remain in Core; Android and Flutter only schedule and present.

### 2. Signatures

```rust
Db::feeds_due_for_refresh(global_min: i64) -> Result<Vec<FeedRefreshInfo>, CoreError>
refresh_feeds(core, RefreshOptions { feed_ids, force }) -> Result<RefreshReport, PaprBridgeError>
get_settings(core) -> Result<SettingsSnapshot, PaprBridgeError>
set_background_settings(core, refresh_interval_min,
                        notifications_enabled,
                        notification_quiet_hours) -> Result<(), PaprBridgeError>
```

The Android adapter registers unique periodic work named
`papr.background.refresh`; its top-level Dart callback invokes task
`papr.refresh.due` from a headless engine.

### 3. Contracts

- `force=true` means a user-initiated refresh and selects every requested
  feed. `force=false` means background work and selects only feeds due under
  the global/per-feed interval. Never reuse the background default in a manual
  refresh entry point.
- Core accepts persisted live intervals from 5 through 120 minutes for desktop
  compatibility and `525_600` as off. Android schedules at 15 through 120
  minutes, so a 5/10-minute Core interval is checked on a best-effort 15-minute
  wake-up rather than rejected or rewritten.
- The Worker requires connectivity, initializes the same FRB/Core database,
  and registers unique periodic work with update semantics. Turning automatic
  refresh off cancels that unique work; changing the interval never creates a
  second job.
- Background settings persist atomically. Flutter reconciles the Android job
  after a successful write; if persistence or scheduling fails, it restores
  the previous settings and schedule before restoring UI state.
- Notifications default off. Android 13+ permission is requested only after an
  explicit enable action. A completed report with zero new articles, disabled
  or revoked permission, or enabled quiet hours during 22:00–08:00 produces no
  notification. A normal run emits one localized count-only notification with
  stable ID `6101`; URLs, titles, bodies, credentials, and provider text never
  enter notification or log output.
- The default Android notification icon is an app-owned white silhouette in
  `res/drawable` (currently `ic_notification.xml`).
  `flutter_local_notifications` resolves `AndroidInitializationSettings` as a
  drawable resource, so a launcher icon that exists only under `res/mipmap`
  is not a valid substitute.

### 4. Validation & Error Matrix

| Condition | Stable result |
| --- | --- |
| Live interval below 5 or above 120 | `invalidRefreshInterval`; no settings change |
| Interval at/above `525_600` | normalized to off; unique periodic work cancelled |
| Worker setup/Core call fails before a report | WorkManager retry (`false`) |
| One feed fails but Core returns a report | completed work (`true`); other inserts remain |
| Notification permission denied or revoked | refresh still completes; no notification |
| Default notification icon is missing or is only a mipmap | initialization returns `invalid_icon`; preference and schedule stay unchanged |
| Scheduling fails after settings persist | restore previous Core settings/job/UI state |

### 5. Good / Base / Bad Cases

- Good: Android wakes one connected job, Core refreshes only due feeds, and one
  localized summary replaces the prior Papr summary.
- Base: a desktop database contains a 5-minute interval. Mobile preserves it,
  schedules Android's 15-minute minimum, and notification settings can still
  be changed atomically.
- Bad: a manual pull-to-refresh sends `force=false`, or Flutter implements its
  own `last_fetched_at` timing query. Both silently skip work the user asked
  for and split the timing contract across layers.

### 6. Tests Required

- Core: due selection covers never fetched, recent, boundary/overdue,
  per-feed override, off, corrupt global fallback, and newsletter exclusion;
  `force=true` and `force=false` have an explicit regression test.
- Flutter: scheduling helpers cover off and 15–120 bounds; notification
  eligibility covers zero, enablement, 22:00/08:00 boundaries, and localized
  count-only text; settings UI exposes rollback-safe controls. A resource
  regression must prove the configured icon name exists under `res/drawable`.
- Android acceptance: confirm a single unique job after restart/update/toggle,
  a successful background/headless run, permission denial, one allowed
  notification, quiet-hour suppression, and persistence after process reclaim;
  inspect the built APK resource table when changing any string-addressed
  notification resource.

### 7. Wrong vs Correct

#### Wrong

```dart
// Manual refresh accidentally applies the background due filter.
const RefreshOptions(feedIds: null, force: false);
```

#### Correct

```dart
// Foreground user action is forced; only WorkManager sends force: false.
const RefreshOptions(feedIds: null, force: true);
```

Keep due selection and feed error isolation in Core, and keep WorkManager and
notification permission behavior in the Android/Flutter adapter.

## Scenario: AI summary and LLM translation

### 1. Scope / Trigger

- Trigger: a reader feature uses an AI Profile and an article cache across
  Android Keystore, Flutter, FRB, Core HTTP/SSE, and SQLite.
- Why: Flutter must never own provider parsing, persistent credentials, or a
  partial translation cache; otherwise a route cancellation can corrupt a
  later reader session.

### 2. Signatures

```rust
stream_ai_summary(core, article_id, profile, credential, template, language, request_id, sink)
stream_ai_translation(core, article_id, profile, credential, language, request_id, sink)
cancel_ai_request(core, request_id) -> bool
```

- Translation persistence is `articles.translated_html` plus
  `articles.translated_lang`; Core writes both only through
  `Db::set_translation_cache` after every HTML batch succeeds.

### 3. Contracts

- Flutter reads a Keystore credential only at request time, passes it as the
  ephemeral `credential` argument, and never writes it to a DTO, provider, log,
  backup, or cache.
- `AiStreamEvent::Delta` belongs to summary/follow-up output. Translation
  emits only `Progress { completed, total }`, then one terminal `Completed` or
  `Error { code }` for the request ID.
- A provider may stream hidden reasoning before visible `content`. Core never
  renders or persists reasoning. For SenseNova 6.8 Flash Lite, Core uses an
  8,192-token output ceiling because the provider counts reasoning toward
  `max_tokens`; an otherwise valid stream with no visible content returns
  `aiNoVisibleOutput`, not `aiParse`.
- The Core translation service chooses extracted HTML when present, chunks it
  without splitting a block, sanitizes every returned fragment, and replaces
  the cache only after the complete result is valid.
- A Flutter translation route owns its request ID and cancels it on disposal.
  It reloads `ArticleDetail.translated_html` only after `Completed`; it does
  not synthesize translated HTML from stream payloads.
- Mobile permits at most one enabled AI Profile. If none is enabled, summary
  and translation show the configuration route and do not attempt a request.

### 4. Validation & Error Matrix

| Condition | Stable code / result |
| --- | --- |
| Article body is missing | `noArticleBody` |
| No enabled profile or no request-time credential | no request / `noAiCredential` |
| Provider rejects credentials | `aiAuth` |
| Provider/network/invalid stream failure | `aiRateLimited`, `aiNetwork`, or `aiParse` |
| Complete stream contains no visible content | `aiNoVisibleOutput` |
| Request cancelled, page left, or stream sink closes | `aiCancelled`; prior cache remains |
| All batches complete and sanitize successfully | `Completed`; HTML and language replace the prior cache together |

### 5. Good / Base / Bad Cases

- Good: a completed multi-block translation emits `0/N … N/N`, then displays
  the freshly re-read cached HTML with links and images intact.
- Base: a cache exists for Japanese and the user selects Chinese. The page
  keeps showing the Japanese cache until the user explicitly regenerates; a
  failed regeneration leaves it untouched.
- Bad: leaving the page during a request, an SSE parse failure, or a failed
  batch must not write a partial translation or overwrite a previous one.

### 6. Tests Required

- Core: assert block progress contains no delta events; completed translation
  writes sanitized HTML/language; provider failure and cancellation retain the
  prior cache; local SSE helpers signal readiness rather than relying on
  scheduler timing.
- Bridge: regenerate FRB bindings and cover request-ID cancellation/lease
  cleanup.
- Flutter: reader exposes the translation route; route localizes stable error
  codes, disables conflicting actions while active, and only displays cache
  re-read after completion.
- Android acceptance: check cached display offline, multi-block progress,
  cancel/back navigation, rotation/process restore, and that a request uses
  the sole enabled Profile.

### 7. Wrong vs Correct

#### Wrong

```dart
// Rebuilds a translation from streamed provider text and can persist a partial body.
stream.listen((event) => translatedHtml += event.text);
```

#### Correct

```dart
// Treat completion as a signal to read the atomically stored Core cache.
await articleRepository.getArticleDetail(articleId);
```

Core owns stream parsing, sanitizing, and the success-only write transaction;
Flutter owns only route state, progress presentation, and cancellation.

## Scenario: Android podcast playback

### 1. Scope / Trigger

- Trigger: an `ArticleDetail.enclosures` entry is playable audio and needs
  background playback, system media controls, and Flutter controls.
- Why: Flutter must not create a second audio player or derive an independent
  playback state from timers; Android owns the player lifecycle.

### 2. Signatures

Android method channel `com.papr.papr_mobile/platform` provides:

```text
startPlayback({ mediaId, url, title, source }) -> bool
playbackCommand({ command, positionMs?, speed? }) -> bool
getPlaybackState() -> PlaybackState
```

The `com.papr.papr_mobile/platform/playback` event channel emits:

```text
PlaybackState { mediaId, title, source, durationMs, positionMs, speed,
                playing, buffering, error? }
```

### 3. Contracts

- Flutter identifies an audio enclosure from `audio/*` MIME type or a known
  audio extension, then sends its URL only to `startPlayback`.
- `PlaybackService` owns the Media3 `ExoPlayer`, `MediaSession`, foreground
  notification, audio focus, and becoming-noisy handling. The full URL never
  appears in a playback event, Dart state, or a log message.
- `play`, `pause`, `skipBack`, `skipForward`, `stop`, `seekTo`, and
  `setSpeed` are idempotent commands. Seek values are non-negative and speed
  is bounded to 0.75-2.0.
- Flutter shares one broadcast event stream between the app shell and the
  control page. A service event, including the 500ms playing-position update,
  is the sole source for the mini-player, full controls, and system UI.

### 4. Validation & Error Matrix

| Condition | Stable code / result |
| --- | --- |
| URL is not HTTP(S), or has no host | `invalidPlaybackUrl` |
| Network connection, timeout, HTTP, or file load error | `playbackNetwork` |
| Player cannot be created or used | `playbackUnavailable` |
| Other player failure | `playbackFailed` |
| Invalid command, seek, or speed | rejected before crossing the channel |

### 5. Good / Base / Bad Cases

- Good: tapping an audio enclosure starts `PlaybackService`; leaving the
  article continues playback and the app-shell mini player follows the same
  state as the notification.
- Base: an enclosure lacks `audio/*` but ends in `.mp3`; Flutter still offers
  the podcast entry while non-audio attachments retain external open behavior.
- Bad: a media URL fails after start. The mini player and full control page
  show the same stable error, with no URL or provider response exposed.

### 6. Tests Required

- Flutter: verify platform command validation, audio-enclosure classification,
  mini-player/control-page state rendering, and unchanged ordinary attachment
  behavior.
- Android acceptance: verify background notification, lock-screen/Bluetooth
  controls, focus loss, headphone unplug, rotation, network failure, and one
  real podcast enclosure.

### 7. Wrong vs Correct

#### Wrong

```dart
// A second player drifts from the MediaSession notification and lock screen.
final localPlayer = AudioPlayer()..play(url);
```

#### Correct

```dart
await platformService.startPlayback(
  mediaId: mediaId,
  url: enclosure.url,
  title: detail.title,
  source: detail.feedTitle,
);
```

Keep `PlaybackService` as the only player and render the event-channel
snapshot everywhere in Flutter.

## 4. Validation & Error Matrix

| Condition | Stable code / result |
| --- | --- |
| Empty folder name | `emptyFolderName` |
| Case-insensitive duplicate folder | `folderNameExists` |
| Empty custom feed title | `emptyFeedTitle` |
| Empty feed input | `emptyFeedUrl` |
| Existing normalized feed URL | `feedAlreadyExists` |
| Feed ID not found | `feedNotFound` |
| Unsupported or malformed input URL | `invalidFeedUrl` |
| Fetch failure | category/code `network` |
| Invalid feed or no discovered feed | category `parse`, code `feedNotFound` or `parse` |
| User cancels document picker | `null` for open, `false` for save |
| Concurrent document request | `documentPickerBusy` |
| Article ID not found | `articleNotFound` |
| Invalid reading setting or article URL | category/code `validation` |
| Fulltext fetch failure | category/code `network` |
| Fulltext parse/extraction failure | category/code `parse` |
| Empty tag name / duplicate rename | `emptyTagName` / `tagNameExists` |
| Invalid tag color / incomplete order | `invalidTagColor` / `invalidTagOrder` |
| Missing tag / rule / highlight | `tagNotFound` / `ruleNotFound` / `highlightNotFound` |
| Empty rule name / query | `emptyRuleName` / `emptyRuleQuery` |
| Unsupported rule field / action | `invalidRuleField` / `invalidRuleAction` |
| Empty quote / negative highlight offset | `emptyHighlight` / `invalidHighlightOffset` |
| Invalid highlight color | `invalidHighlightColor` |

Flutter localizes stable codes in `mobile/lib/l10n/l10n.dart`; it must not parse human-readable `detail` text.

## 5. Good / Base / Bad Cases

- Good: add a web page containing a feed link; Core discovers it, writes the feed and initial articles atomically, and Flutter refreshes repository state.
- Base: delete a folder; its feeds remain visible as unclassified and folder ordering stays contiguous.
- Bad: first article insertion fails; the feed and change-log write roll back. Never persist a partial subscription.
- Bad: malformed OPML or a failed SAF write reports an error and leaves existing subscriptions unchanged.
- Good: the same article filter drives a 50-row page, its count, and mark-all-read; rows and count stay consistent after optimistic refresh.
- Base: extraction is unavailable offline; the reader keeps showing cached extracted/content HTML and local state writes continue to work.
- Bad: a punctuation-only search string becomes an empty safe query, not raw FTS syntax or an unbounded SQL fragment.
- Good: preview and apply receive the same `RuleInput`; their match counts agree and later ingestion evaluates the same matcher.
- Base: a star rule previews two matching articles after both are already starred; applying it returns zero changes, but the client still refreshes Starred/count projections from Core.
- Base: article text changed after a highlight was created; Core relocates the quote using its surrounding context and reports the new UTF-16 range.
- Bad: a skip rule matches a starred or highlighted article; Core retains it instead of deleting user-curated state.
- Good: reopening an article resolves a persisted quote and paints its mark in the original paragraph while keeping the same record in the list below.
- Base: a quote crosses an inline element such as `<strong>`; each intersected text node receives a mark while the original element structure is preserved.
- Bad: article text changed and Core returns null coordinates; do not inject a guessed mark or discard the note.

## 6. Tests Required

- Core: CRUD validation, complete reorder set, cascade/set-null behavior, OPML deduplication and round trip, source discovery, and add-feed transaction rollback.
- Bridge: DTO conversion and stable error propagation; generated bindings must be reproducible.
- Flutter: phone `NavigationBar`, tablet `NavigationRail`, feed/folder actions, directory add flow, localized errors, and reader regressions.
- Android acceptance: cold and warm deep links, SAF import/export, process restart persistence, rotation/narrow layout, and TalkBack labels.
- Reading Core: safe FTS terms, tag/feed/folder/smart filters, bounded paging and deterministic order, count parity, idempotent state/change-log writes, extraction cache invalidation, and reading-setting validation/persistence.
- Reading Flutter: smart-view counts, 50-row next-page offset, metadata/placeholder rendering, optimistic rollback, safe HTML/link behavior, and URL-dependent browser/share actions.
- Reading Android acceptance: browser/share intents, back navigation, rotation, process restore, and large-list scrolling.
- Organization Core: tag validation/order/association, rule matcher parity and protected skip, highlight CRUD and UTF-16/context anchor fallback, plus change-log transaction behavior.
- Organization Flutter: selection-menu highlight creation, tag/rule management, preview/apply confirmation, stable-code localization, route behavior, and failed mutation rollback.
- Rule manager Flutter: preview displays the Core match count and samples; `skip` application requires confirmation; an in-flight enable/disable change is visible immediately and reverts after a rejected write.
- Rule apply Flutter: applying an existing-article rule refreshes article pages and both count-provider families even when Core reports zero changed rows; regressions assert that the refreshed Starred projection is observed.
- Reader state Flutter: star and read-later writes survive closing and reopening the article, and the reader contains no per-article tag-edit action.
- Organization reader regression: resolved ranges render a color-coded `<mark>`, inline-element boundary selections retain valid HTML, and unresolved ranges render no guessed mark but do show the localized notice.
- Organization Android acceptance: long-press selection, tag/rule flows, highlight reopen/edit/delete, unresolved-anchor presentation, rotation, process restore, and large-body behavior.
- Full gate: `cargo test -p papr-core`, `cargo test -p papr-flutter-bridge`, `cargo test -p papr`, `flutter analyze`, `flutter test`, and `flutter build apk --debug`.

## 7. Wrong vs Correct

### Wrong

```dart
// Reimplements a Core rule and loses the stable error contract.
if (input.isEmpty) throw Exception('URL required');
```

### Correct

```dart
// Pass input through the repository and localize PaprBridgeError.code.
await repository.addFeed(input);
```

Keep validation at the Core boundary, transport typed DTOs through FRB, and limit Flutter to presentation and orchestration.

For optimistic article state, update the visible row immediately, call the repository, and on failure restore the old row before invalidating list/count/detail providers. Never hide a failed write behind a refresh-only fallback.

Do not implement a second rule matcher in Dart or use Dart string offsets as Rust byte offsets. Send the typed rule input unchanged and persist selection offsets as UTF-16 code units; Core owns both matching and anchor recovery.

For persisted highlights, resolve in Core and use a DOM parser to split only affected text nodes. Do not use `String.replaceAll` or raw-HTML regular expressions: they can match markup, damage entities, and paint the wrong occurrence.
