// Shared article mutations. Both the reading pane and keyboard shortcuts go
// through this so optimistic cache patching stays consistent everywhere.

import { useQueryClient } from "@tanstack/react-query";
import * as api from "../api";
import { errorText } from "../lib/errors";
import { patchArticleFlags, writeArticleFlag, type ArticleFlags } from "../lib/articleCache";
import { refreshArticleQueries } from "../lib/readingQueries";

type Patch = Partial<ArticleFlags>;


/**
 * Shared article mutations. `onError` (when supplied) is called with a
 * localized message if a mutation fails — callers fire these actions without
 * awaiting, so without it a failure would be a silent unhandled rejection.
 */
export function useArticleActions(onError?: (msg: string) => void) {
  const qc = useQueryClient();

  const patch = (id: number, p: Patch) => patchArticleFlags(qc, id, p);

  const refreshLists = () => {
    qc.invalidateQueries({ queryKey: ["counts"] });
    qc.invalidateQueries({ queryKey: ["feeds"] });
    // Smart views (Starred / Read Later / Unread) are each their own
    // ["articles", …] query. The optimistic `patch` above fixes articles
    // already in a list, but it can't add or remove rows — so a freshly
    // starred article never appears in the Starred list. Mark every
    // article/search list stale so it re-fetches with the correct
    // membership when next opened. `refetchType: "none"` avoids yanking
    // rows out of the list the user is currently looking at.
    qc.invalidateQueries({ queryKey: ["articles"], refetchType: "none" });
    qc.invalidateQueries({ queryKey: ["search"], refetchType: "none" });
    qc.invalidateQueries({ queryKey: ["cp-search"], refetchType: "none" });
  };

  // The query keys an article-state change can affect. A bare
  // `invalidateQueries()` would also refetch unrelated caches (AI summaries,
  // settings, FreshRSS status, rules, the feed-discovery search), so callers
  // invalidate only these — plus any `extra` keys.
  const refreshArticleKeys = (extra: string[][] = []) => refreshArticleQueries(qc, extra);

  // After a bulk operation (mark-all-read) potentially every article's state
  // changed, so optimistic patching can't cover it.
  const refreshAfterBulk = () => refreshArticleKeys();

  // After a manual feed refresh new articles may have arrived; `storage-stats`
  // is added because the new articles grow the database.
  const refreshAfterFetch = () => refreshArticleKeys([["storage-stats"]]);

  const applyFlag = async (
    id: number,
    field: keyof ArticleFlags,
    value: boolean,
    write: () => Promise<unknown>,
  ) => {
    try {
      await writeArticleFlag(qc, id, field, value, write);
      refreshLists();
    } catch (e) {
      onError?.(errorText(e));
    }
  };

  return {
    patch,
    refreshAfterBulk,
    refreshAfterFetch,
    setRead: (id: number, read: boolean) =>
      applyFlag(id, "isRead", read, () => api.markRead(id, read)),
    setStarred: (id: number, starred: boolean) =>
      applyFlag(id, "isStarred", starred, () => api.markStarred(id, starred)),
    setReadLater: (id: number, value: boolean) =>
      applyFlag(id, "readLater", value, () => api.markReadLater(id, value)),
  };
}
