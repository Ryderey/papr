// Optimistic article-flag cache helpers. Plain functions over a QueryClient so
// the patch/rollback path stays testable without a DOM harness.
import type { QueryClient } from "@tanstack/react-query";
import type { ArticleSummary } from "../types";

export type ArticleFlags = Pick<
  ArticleSummary,
  "isRead" | "isStarred" | "readLater"
>;

/** Stop stale reads before patching; QueryClient retains the latest manual data. */
export function cancelArticleFlagReads(qc: QueryClient, id: number) {
  return qc.cancelQueries({
    predicate: ({ queryKey }) =>
      queryKey[0] === "articles" || queryKey[0] === "search" ||
      queryKey[0] === "cp-search" ||
      (queryKey[0] === "article" && queryKey[1] === id),
  });
}

/** Optimistically patch an article's flags across every cache that may hold it. */
export function patchArticleFlags(
  qc: QueryClient,
  id: number,
  p: Partial<ArticleFlags>,
) {
  // Paginated browse lists.
  qc.setQueriesData({ queryKey: ["articles"] }, (old: any) => {
    if (!old?.pages) return old;
    return {
      ...old,
      pages: old.pages.map((page: ArticleSummary[]) =>
        page.map((x) => (x.id === id ? { ...x, ...p } : x)),
      ),
    };
  });
  // Flat result arrays: hybrid search and the command-palette search.
  const patchFlat = (old: any) =>
    Array.isArray(old)
      ? old.map((x: ArticleSummary) => (x.id === id ? { ...x, ...p } : x))
      : old;
  qc.setQueriesData({ queryKey: ["search"] }, patchFlat);
  qc.setQueriesData({ queryKey: ["cp-search"] }, patchFlat);
  // The open article detail.
  qc.setQueryData(["article", id], (old: any) =>
    old ? { ...old, ...p } : old,
  );
}

/** The cached flags of one article, used to roll an optimistic patch back. */
export function readArticleFlags(
  qc: QueryClient,
  id: number,
): ArticleFlags | undefined {
  const flags = (x: ArticleSummary): ArticleFlags => ({
    isRead: x.isRead,
    isStarred: x.isStarred,
    readLater: x.readLater,
  });
  for (const [, data] of qc.getQueriesData<any>({ queryKey: ["articles"] })) {
    const pages = data?.pages as ArticleSummary[][] | undefined;
    if (!pages) continue;
    for (const page of pages) {
      const hit = page.find((x) => x.id === id);
      if (hit) return flags(hit);
    }
  }
  for (const key of [["search"], ["cp-search"]]) {
    for (const [, data] of qc.getQueriesData<any>({ queryKey: key })) {
      if (!Array.isArray(data)) continue;
      const rows = data as ArticleSummary[];
      const hit = rows.find((x) => x.id === id);
      if (hit) return flags(hit);
    }
  }
  const detail = qc.getQueryData<ArticleSummary>(["article", id]);
  return detail ? flags(detail) : undefined;
}

type Flag = keyof ArticleFlags;
type PendingFlag = {
  confirmed: boolean | undefined;
  desired: boolean;
  pending: number;
  tail: Promise<void>;
};
type PendingClient = {
  articles: Map<number, Map<Flag, PendingFlag>>;
  unsubscribe: () => void;
};

// Hook instances and re-renders share a queue through their QueryClient. Only
// writes to the same article field are serialized; other fields remain free.
const pendingWrites = new WeakMap<QueryClient, PendingClient>();

export function writeArticleFlag(
  qc: QueryClient,
  id: number,
  field: Flag,
  value: boolean,
  write: () => Promise<unknown>,
): Promise<void> {
  let client = pendingWrites.get(qc);
  if (!client) {
    const articles: PendingClient["articles"] = new Map();
    const unsubscribe = qc.getQueryCache().subscribe((event) => {
      if (event.type !== "updated" || event.action.type !== "success" || event.action.manual) return;
      if (!["articles", "article", "search", "cp-search"].includes(String(event.query.queryKey[0]))) return;
      // Background refreshes can begin after the initial cancellation. Overlay
      // only pending fields, preserving the fresh body and unrelated state.
      for (const [articleId, fields] of articles) {
        const patch: Partial<ArticleFlags> = {};
        for (const [key, queued] of fields) patch[key] = queued.desired;
        patchArticleFlags(qc, articleId, patch);
      }
    });
    client = { articles, unsubscribe };
    pendingWrites.set(qc, client);
  }
  let fields = client.articles.get(id);
  if (!fields) {
    fields = new Map();
    client.articles.set(id, fields);
  }
  let queued = fields.get(field);
  if (!queued) {
    queued = {
      confirmed: readArticleFlags(qc, id)?.[field],
      desired: value,
      pending: 0,
      tail: Promise.resolve(),
    };
    fields.set(field, queued);
  }
  const state = queued;
  const owner = client;
  const articleFields = fields;
  state.desired = value;
  state.pending += 1;
  const cancelledReads = cancelArticleFlagReads(qc, id);
  patchArticleFlags(qc, id, { [field]: value });

  const result = state.tail.then(async () => {
    try {
      await cancelledReads;
      await write();
      state.confirmed = value;
    } finally {
      await cancelArticleFlagReads(qc, id);
      state.pending -= 1;
      const visible = state.pending > 0 ? state.desired : state.confirmed;
      if (state.pending === 0) {
        articleFields.delete(field);
        if (articleFields.size === 0) owner.articles.delete(id);
        if (owner.articles.size === 0) {
          owner.unsubscribe();
          pendingWrites.delete(qc);
        }
      }
      if (visible !== undefined) patchArticleFlags(qc, id, { [field]: visible });
    }
  });
  state.tail = result.catch(() => {});
  return result;
}
