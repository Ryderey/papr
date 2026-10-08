# Desktop State Write Contracts

## 1. Scope / Trigger

Apply when changing desktop article flags, optimistic query caches, or GitHub
projection cancellation. Reads and writes can overlap while the SQLite writer
is busy; cached boolean equality does not establish mutation ownership.

## 2. Signatures

```ts
writeArticleFlag(
  qc: QueryClient, id: number, field: keyof ArticleFlags, value: boolean,
  write: () => Promise<unknown>,
): Promise<void>;
```

```rust
storage::finish(conn, id, lease, remote, cancel: &Cancellation) -> Result<(), CoreError>;
```

Tauri `mark_read`, `mark_starred`, and `mark_read_later` retain their existing
ID/boolean payloads. No new schema, protocol fields, or environment settings.

## 3. Contracts

- Reader, Article List, and shortcuts share pending writes through QueryClient.
  Serialize backend calls for the same article and field; different fields and
  articles may progress independently. Render the latest intent immediately.
- Update the confirmed value only after a successful write. When the final
  queued write fails, restore the last confirmed value, preserving subsequent
  pending intents and other fields. Errors still reach the hook's error toast.
- Treat article-cache snapshots as immutable. Flag updates produce new arrays
  and rows, including when input pages/rows are frozen. Desktop keeps current
  reading-list rows in place and invalidates membership for the next fetch;
  do not copy Android's mutable-list removal into this cache path.
- Cancel stale detail/list/search reads without reverting other optimistic
  changes. Queries started during a pending write must also retain pending flags
  while allowing fresh content and unrelated server state to update.
- Release cache subscriptions and queue entries after writes drain. Subsequent
  cloud state remains authoritative once there are no pending local writes.
- GitHub import and acknowledgement remain one transaction. Check cancellation
  during identity loading, entity projection, and version persistence, and
  after file/acknowledgement writes before returning success. An error rolls back
  data, applying, versions, files, outbox acknowledgement, and cursor together;
  the existing failure path records the code and releases the lease.

## 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Old query returns after a click | Pending flags remain visible; successful write remains confirmed |
| Two same-field writes both fail | Restore original confirmed value, not an earlier optimistic value |
| Earlier write fails with a later intent queued | Keep the latest intent visible |
| Latest write fails after an earlier success | Restore that success |
| Another field succeeds | A failure must not roll it back |
| Cancel in final article/version/file/ack phase | `githubSyncCancelled`, atomic rollback, lease released |

## 5. Good / Base / Bad Cases

- Good: star true/false/true remains true when the first request fails and the
  following requests succeed, even when issued by different hook instances.
- Base: one successful mark-read updates paginated, search, and detail caches.
- Bad: comparing the current boolean to a failed request's boolean can roll
  back a different request with the same value. Checking cancellation only
  every 64 article positions misses the final batch and persistence tail.

## 6. Tests Required

- Actual hook tests with deferred queries/writes cover stale detail and list
  reads, overlapping failures, latest-intent visibility, different articles,
  and independent fields. Pure patch-helper tests cannot prove write ordering.
- Actual hook tests also cover mark-read, unstar and remove-read-later with
  frozen cached pages/rows, asserting successful persistence calls, updated
  list/detail flags, unchanged input snapshots and no error callback.
- SQL-trigger fixtures request cancellation in the final article, version,
  file, and acknowledgement stages. Assert error, unchanged data/cursor/outbox/
  file contents/applying, and failure-path lease release.
- Run existing frontend tests/build and serial Rust workspace tests. Local
  fixtures do not establish acceptance on another Windows installation.

## 7. Wrong vs Correct

Wrong: capture all cached flags before a request, then restore them after any
failure; or return success after a cancel requested in the last SQL statement.

Correct: queue same-field writes with confirmed/latest values, overlay pending
flags on incoming queries, and call `cancel.check()` at the transaction tail
so the caller commits only a successful, uncancelled projection.
