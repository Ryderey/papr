# Implement: P3 标签、规则与高亮

## A. Core

1. [x] Port tag CRUD, color, association, ordering, validation, and article counts to `papr-core`; add transaction-backed change-log writes.
2. [x] Port rule DTOs, validation, active-rule retrieval, Unicode matcher, preview, and protected existing-article application; connect enabled rules to ingestion.
3. [x] Port highlight CRUD, color/note edits, all-highlight list, pure anchor resolver, and stable error codes; add change-log transactions.
4. [x] Add focused Core tests for tag/rule/highlight contracts, no-half-write behavior, retained saved/highlighted articles, and anchor fallback.

## B. FRB

5. [x] Expose tags, rules, previews, highlights, resolved anchors, and mutations through typed DTOs and APIs.
6. [x] Regenerate Dart bindings; confirm a second generation is byte-identical and Bridge tests pass.

## C. Flutter

7. [x] Extend repositories/providers for tags, rules, highlights, preview/apply, and local state refresh.
8. [x] Add rule manager with preview samples and protected destructive-action confirmation; remove mobile tag-editing and tag-management entry points per the mobile scope decision.
9. [x] Add `SelectionArea` reader integration, create-highlight context action, color/note editor, per-article/global browsers, resolved inline marks, and unresolved-anchor notice.
10. [x] Add localized strings and Widget tests for selection setup, management flows, error rollback, and route behavior. The HTML renderer's real long-press selection is covered by Android hardware acceptance because the standard Widget-test binding cannot synthesize its selection geometry.

## D. Quality and Device Gate

11. [x] Run Core/Bridge/desktop tests, FRB idempotence, `flutter analyze`, `flutter test`, and Debug APK build.
12. [x] Verify star/read-later persistence, rules, selections/highlights, rotation, process restore, and large-body behavior on Android hardware.

Completed: localized strings; selection setup, management-flow, optimistic-rollback and route Widget regressions; resolved inline-mark and unresolved-anchor tests; star/read-later reopen persistence; rule list; global-highlight-to-reader routing; rule preview/apply confirmation; failed enable-state rollback; and article-view/count refresh after a rule applies. Android hardware acceptance passed for star/read-later persistence, rules, long-press highlight creation and reopening, rotation, process restore, and large-body highlighting.

## Rollback Points

- A: Core commit contains no migration and can be reverted independently.
- B: FRB commit is generated-only plus thin API mapping.
- C: Flutter commit retains P2 reader when P3 UI is reverted.

## Handoff

- Record public DTO/error/change-log contracts.
- Record verification counts and any renderer selection/inline-mark limitation discovered on device.
