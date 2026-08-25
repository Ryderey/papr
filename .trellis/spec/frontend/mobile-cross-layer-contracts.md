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
