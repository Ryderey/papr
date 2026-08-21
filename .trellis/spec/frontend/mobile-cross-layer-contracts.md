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

Flutter localizes stable codes in `mobile/lib/l10n/l10n.dart`; it must not parse human-readable `detail` text.

## 5. Good / Base / Bad Cases

- Good: add a web page containing a feed link; Core discovers it, writes the feed and initial articles atomically, and Flutter refreshes repository state.
- Base: delete a folder; its feeds remain visible as unclassified and folder ordering stays contiguous.
- Bad: first article insertion fails; the feed and change-log write roll back. Never persist a partial subscription.
- Bad: malformed OPML or a failed SAF write reports an error and leaves existing subscriptions unchanged.
- Good: the same article filter drives a 50-row page, its count, and mark-all-read; rows and count stay consistent after optimistic refresh.
- Base: extraction is unavailable offline; the reader keeps showing cached extracted/content HTML and local state writes continue to work.
- Bad: a punctuation-only search string becomes an empty safe query, not raw FTS syntax or an unbounded SQL fragment.

## 6. Tests Required

- Core: CRUD validation, complete reorder set, cascade/set-null behavior, OPML deduplication and round trip, source discovery, and add-feed transaction rollback.
- Bridge: DTO conversion and stable error propagation; generated bindings must be reproducible.
- Flutter: phone `NavigationBar`, tablet `NavigationRail`, feed/folder actions, directory add flow, localized errors, and reader regressions.
- Android acceptance: cold and warm deep links, SAF import/export, process restart persistence, rotation/narrow layout, and TalkBack labels.
- Reading Core: safe FTS terms, tag/feed/folder/smart filters, bounded paging and deterministic order, count parity, idempotent state/change-log writes, extraction cache invalidation, and reading-setting validation/persistence.
- Reading Flutter: smart-view counts, 50-row next-page offset, metadata/placeholder rendering, optimistic rollback, safe HTML/link behavior, and URL-dependent browser/share actions.
- Reading Android acceptance: browser/share intents, back navigation, rotation, process restore, and large-list scrolling.
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
