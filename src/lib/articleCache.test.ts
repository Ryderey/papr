import { describe, expect, it } from "vitest";
import { QueryClient } from "@tanstack/react-query";
import { patchArticleFlags, readArticleFlags } from "./articleCache";
import type { ArticleSummary } from "../types";

const article = (id: number, flags: Partial<ArticleSummary> = {}) =>
  ({ id, isRead: false, isStarred: false, readLater: false, ...flags }) as ArticleSummary;

function seeded() {
  const qc = new QueryClient();
  qc.setQueryData(["articles", "unread"], {
    pages: [[article(1), article(2, { isRead: true })], [article(3)]],
  });
  qc.setQueryData(["search", "term"], [article(4, { isStarred: true })]);
  qc.setQueryData(["cp-search", "term"], [article(5)]);
  qc.setQueryData(["article", 6], article(6, { readLater: true }));
  return qc;
}

describe("optimistic article flags", () => {
  it("patches every cache that holds the article and nothing else", () => {
    const qc = seeded();
    patchArticleFlags(qc, 2, { isRead: false, isStarred: true });

    const pages = qc.getQueryData<any>(["articles", "unread"]).pages;
    expect(pages[0][1]).toMatchObject({ id: 2, isRead: false, isStarred: true });
    expect(pages[0][0]).toMatchObject({ id: 1, isRead: false, isStarred: false });
    expect(pages[1][0]).toMatchObject({ id: 3, isRead: false, isStarred: false });

    patchArticleFlags(qc, 4, { isStarred: false });
    expect(qc.getQueryData<any>(["search", "term"])[0]).toMatchObject({ id: 4, isStarred: false });

    patchArticleFlags(qc, 5, { readLater: true });
    expect(qc.getQueryData<any>(["cp-search", "term"])[0]).toMatchObject({ id: 5, readLater: true });

    patchArticleFlags(qc, 6, { readLater: false });
    expect(qc.getQueryData<any>(["article", 6])).toMatchObject({ id: 6, readLater: false });
    qc.clear();
  });

  it("reads the current flags back for a rollback", () => {
    const qc = seeded();
    expect(readArticleFlags(qc, 2)).toEqual({ isRead: true, isStarred: false, readLater: false });
    expect(readArticleFlags(qc, 6)).toEqual({ isRead: false, isStarred: false, readLater: true });
    expect(readArticleFlags(qc, 99)).toBeUndefined();
    qc.clear();
  });
});
