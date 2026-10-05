import { describe, expect, it } from "vitest";
import { QueryClient } from "@tanstack/react-query";
import { refreshArticleQueries } from "./readingQueries";

describe("reading cache refresh", () => {
  it("marks folders, open details and reading lists stale without invalidating settings", () => {
    const client = new QueryClient();
    const keys = [["folders"], ["article", 7], ["articles", "unread"], ["counts"], ["feeds"], ["search", "term"], ["cp-search", "term"]];
    for (const key of [...keys, ["settings"]]) client.setQueryData(key, { old: true });
    refreshArticleQueries(client);
    for (const key of keys) expect(client.getQueryState(key)?.isInvalidated).toBe(true);
    expect(client.getQueryState(["settings"])?.isInvalidated).toBe(false);
    client.clear();
  });
});
