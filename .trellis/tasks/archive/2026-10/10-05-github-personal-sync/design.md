# Design

The approved protocol design is `docs/github-personal-sync-design-2026-10-05.md`. It is the source of truth for dataset identity, 64 shards, device sequence watermarks, source generations, atomic tree/commit/non-force-ref publication, durable attempts, retention, and excluded fields.

## Integration

Create an isolated managed worktree from optimize-bugfix and merge feat/flutter-android-rearchitecture. Preserve both branches' behavior; resolve the AGENTS conflict by retaining the common rules and Trellis block. Move the existing Release profile to the workspace root so Cargo honors it. Validate before completing the merge. Advance optimize-bugfix by fast-forward, then create codex/github-personal-sync from the integrated baseline. Do not modify the original checkout's uncommitted Android release work.

## Sync implementation

Implement the shared GitHub module alongside GReader. Shared SQL helpers operate on the adapters' existing Connection/Transaction; no second desktop database handle. Capture subscription/folder/state mutations transactionally, preserve in-flight edits, import metadata-only catalog entries, and hydrate bodies without overwriting flags. Credentials are transient transport inputs; profiles store references only. Windows and Flutter supply credential access and scheduling.

## Compatibility and verification

Append migrations after Core v17. Test upgrades of populated v15 desktop and v16 Android databases. Keep archived source rows for protected articles after unsubscribe. Do not open migrated databases with older binaries. Tests use temporary databases and fake transport/local HTTP endpoints. Real private repository/device checks require dedicated resources and are reported separately.

## Implementation refinements

The user supplied private repository Ryderey/papr-sync, ID 1405209831, default main. Follow docs/github-sync-implementation-2026-10-05.md for setup/evidence and current limits. A derived manifest file_hashes inventory validates exact non-manifest content without a self/commit reference; empty shards remain sparse. Imports vacate folder names transactionally to handle final-name swaps/reuse. Local wipe disconnects before clearing GitHub metadata, without cloud deletion. Secure sequence checkpoints complement installation/path binding. Generic failures back off at least 60 seconds; fatal credential/integrity/history errors stop foreground automatic retries.
