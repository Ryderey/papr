# Mobile Cross-Layer Contracts

## 1. Scope / Trigger

Use this contract when a mobile subscription feature crosses SQLite, `papr-core`, Flutter Rust Bridge (FRB), Flutter repositories, and Android platform APIs. Business rules belong in `papr-core`; FRB and platform channels remain thin adapters.

## 2. Signatures

The P1 bridge surface is defined in `crates/papr-flutter-bridge/src/api.rs`:

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

Android channel: `com.papr.papr_mobile/platform`, with `getInitialDeepLink`, `openOpmlDocument`, `saveOpmlDocument`, and the pushed `deepLink` event.

## 3. Contracts

- `Feed` carries `folder_id`, `source_type`, `custom_title`, and `refresh_interval_min` across every layer; do not reconstruct them in Flutter.
- `Folder.position` is authoritative. Reordering must submit the complete folder ID set.
- Add-feed normalization, discovery, first fetch, parsing, classification, and persistence execute in Core. Feed, articles, and change-log rows commit in one transaction.
- Folder/feed writes use `Db::transact` and append `folder`/`feed` change-log entries in the same transaction.
- Deleting a folder preserves feeds with `folder_id = NULL`; deleting a feed relies on database cascades for its articles.
- OPML transport is text. Android uses Storage Access Framework document intents and requests no broad storage permission.
- A supported deep link is `papr://subscribe?url=<encoded>`; invalid or unsupported links return `None` and do not open the add-feed flow.

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

Flutter localizes stable codes in `mobile/lib/l10n/l10n.dart`; it must not parse human-readable `detail` text.

## 5. Good / Base / Bad Cases

- Good: add a web page containing a feed link; Core discovers it, writes the feed and initial articles atomically, and Flutter refreshes repository state.
- Base: delete a folder; its feeds remain visible as unclassified and folder ordering stays contiguous.
- Bad: first article insertion fails; the feed and change-log write roll back. Never persist a partial subscription.
- Bad: malformed OPML or a failed SAF write reports an error and leaves existing subscriptions unchanged.

## 6. Tests Required

- Core: CRUD validation, complete reorder set, cascade/set-null behavior, OPML deduplication and round trip, source discovery, and add-feed transaction rollback.
- Bridge: DTO conversion and stable error propagation; generated bindings must be reproducible.
- Flutter: phone `NavigationBar`, tablet `NavigationRail`, feed/folder actions, directory add flow, localized errors, and reader regressions.
- Android acceptance: cold and warm deep links, SAF import/export, process restart persistence, rotation/narrow layout, and TalkBack labels.
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
