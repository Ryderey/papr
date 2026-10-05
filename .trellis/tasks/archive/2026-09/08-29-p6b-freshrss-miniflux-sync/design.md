# Design: P6B FreshRSS 与 Miniflux 同步

## Boundaries

- `papr-core` owns the sync state machine, durable outbox, cursors, conflict rules, and provider-neutral `SyncPort` contract.
- A GReader adapter implements `SyncPort` for FreshRSS and Miniflux. Provider selection changes only API-root normalization and compatible capability metadata.
- FRB moves typed, secret-free connection and status data. A secret is accepted only as a transient argument while connecting or syncing; it never returns in a DTO.
- Flutter owns form state and retrieves a selected secret from Android Keystore only for a single bridge call. Android owns encryption, alias lifecycle, and credential-reference validation.
- P6A's existing unique WorkManager task remains the sole scheduler. It may call the same sync use case only after Core says the configured low-frequency window is due.

## Sync Contract

`SyncPort` exposes four explicit operations: authenticate/validate, push an ordered change batch, pull a cursor-scoped snapshot or delta, and acknowledge a completed cursor. Its DTOs carry stable opaque remote IDs, entity kind, mutation kind (`upsert` / `tombstone`), cursor, and per-item outcome; they do not carry credentials.

Core writes every local read/starred mutation to one outbox derived from the existing transactional `change_log`; it does not retain the desktop-only `sync_queue` model. A mutation remains pending until the port acknowledges it. The algorithm is:

1. Load the secret by reference at the Flutter/Android edge and establish the port session.
2. Read pending local mutations in sequence order, push them, and atomically record acknowledgements. Failed or unacknowledged rows remain pending.
3. Pull remote folder/subscription and article-state changes from the stored cursor.
4. Apply remote data transactionally. A local entity with a pending conflicting mutation keeps the local value while accepting its remote ID mapping.
5. Persist the new cursor only after all prior steps complete. A retry therefore replays safely.

Remote subscription/folder additions are merged by stable remote ID with URL/name fallback. This release does not propagate destructive deletion to real GReader services: it retains local data and avoids automatic remote unsubscription. Tombstones are nevertheless first-class `SyncPort` messages and are covered by Fake Provider tests, so the future Papr provider does not require a second protocol.

## Credentials and Status

The existing Android encrypted preference/Keystore implementation becomes a generic private credential store with separate validation rules for `papr.ai.*` and `papr.sync.*`. A sync profile stores only a `credential_ref`; its secret value is removed during disconnect and data reset. Core stores no auth token.

`SyncStatus` includes connected provider, configured URL, last successful sync timestamp, in-progress state, retry eligibility, and a stable redacted error code. UI should never display raw endpoint responses, HTTP headers, passwords, token values, article body, or URLs embedded in errors.

## Scheduling and Failure Handling

Manual sync always calls the Core use case. Background refresh queries a Core-maintained sync-due timestamp after feed refresh and skips sync when not due. An unavailable credential, user disconnect, or failed authentication finishes feed refresh normally and updates only sync status. Network and provider failures leave the outbox/cursor unchanged and return a retryable sync failure; the existing WorkManager policy owns later retries without adding another periodic task.

## Verification Strategy

Core tests use an in-memory Fake Provider with controllable acknowledgements, deltas, cursors, tombstones, and failure injection. Flutter tests cover alias validation, secret absence, optimistic status rollback, and the visible status actions. Android tests/acceptance cover Keystore isolation and one-worker scheduling. A real FreshRSS or Miniflux endpoint is an optional device acceptance layer after fake-provider contracts pass.
