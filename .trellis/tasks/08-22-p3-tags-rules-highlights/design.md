# Design: P3 标签、规则与高亮

## 1. Boundary

`papr-core` owns tag/rule/highlight DTOs, validation, matching, SQLite writes, transaction boundaries, and change-log entries. FRB exposes typed operations only; Flutter repositories coordinate provider invalidation and optimistic UI state. No migration is needed because P0 already includes the required tables.

## 2. Core Contracts

- `Tag` adds `position` and `article_count`; create returns the existing ID for a case-insensitive duplicate, while a rename collision returns `tagNameExists`.
- `Rule` has `id`, `name`, `enabled`, optional `feed_id`, `field`, comma-separated `query`, `action`, and `position`; `RulePreview` returns a count plus five recent titles.
- `Highlight` stores `quote`, `prefix`, `suffix`, `text_offset`, color, note, and creation timestamp. Core exposes a pure anchor resolver returning a range or unresolved state.
- Valid tag/highlight colors are the existing desktop palette keys. Invalid fields/actions/colors and empty names/queries/quotes return typed stable errors.
- Every tag/rule/highlight mutation, article-tag association, and bulk rule application executes through `Db::transact` and appends the corresponding `change_log` record in the same transaction. No half-write is observable.

## 3. Rule Semantics

- One shared, Unicode case-folded matcher powers incoming ingestion, preview, and existing-article application. `%`, `_`, and `\\` are treated as literal query characters in SQL preview/apply.
- Rules are evaluated in position order. A matching `skip` wins immediately; matching `read` and `star` set their respective state.
- Applying `skip` to existing records deletes only disposable matches: it must retain articles that are starred, read-later, or have highlights. FTS and dependent rows remain consistent through existing database triggers/cascades.
- Disabled rules are excluded from ingestion; preview remains available for a draft regardless of enable state.

## 4. Highlighter UX

1. Wrap the existing `HtmlWidget` in Flutter `SelectionArea`; the installed renderer participates in Flutter selection.
2. Capture the selected plain text from `onSelectionChanged`, compute its first matching offset in the same reader plain-text basis, and capture 32-character prefix/suffix context.
3. Add an "Add highlight" context-menu action that opens a color/note sheet and persists the anchor through the repository. Empty selections never show the action.
4. On reload, use the pure resolver (exact offset, context-scored quote match, then first quote match). Show a per-article highlight list, colors, notes, and an unresolved notice; unresolved records remain editable/deletable.
5. Use `HtmlWidget`'s supported `SelectionArea` integration; do not introduce a WebView or a separate HTML renderer. Persistent inline painting is limited to renderer-supported safe `<mark>` styling, while the canonical user-visible, persisted representation is the resolved highlight list and selected-text workflow.

## 5. Flutter Surfaces

- Article detail: persistent star/read-later actions, selection context menu, highlight edit/delete sheet, and resolved/unresolved list. Per-article tag editing is deferred and has no reader UI.
- Settings/organization: tag manager and rule manager with reorder, validation, preview sample list, and an explicit confirmation before applying a destructive skip rule to existing articles.
- Global highlight browser: route/sheet grouped by article, opening the associated reader.
- All mutations update the relevant tag/article/detail/rule/highlight providers optimistically and restore prior state on failure.

## 6. Verification and Rollback

- Core tests cover validation, matching parity, protected skip, change-log atomicity, tag ordering/association, and anchor resolution.
- Widget tests cover context-menu availability, management forms, preview/apply confirmation, and failed optimistic mutations.
- Regenerate FRB deterministically, then run the project-wide gate and Debug APK build.
- Commit Core, FRB, Flutter, and docs separately. The feature can roll back at any layer without schema rollback.
