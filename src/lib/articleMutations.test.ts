import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClient } from "@tanstack/react-query";
import { useArticleActions } from "../hooks/articleActions";
import type { ArticleSummary } from "../types";

const mocks = vi.hoisted(() => ({
  client: undefined as QueryClient | undefined,
  markRead: vi.fn(),
  markStarred: vi.fn(),
  markReadLater: vi.fn(),
}));
vi.mock("@tanstack/react-query", async (importOriginal) => ({
  ...await importOriginal<typeof import("@tanstack/react-query")>(),
  useQueryClient: () => mocks.client,
}));
vi.mock("../api", () => ({
  markRead: mocks.markRead,
  markStarred: mocks.markStarred,
  markReadLater: mocks.markReadLater,
}));
vi.mock("./errors", () => ({ errorText: () => "Write failed" }));

function deferred<T = void>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

const article = (id = 1) =>
  ({ id, isRead: false, isStarred: false, readLater: false }) as ArticleSummary;

beforeEach(() => {
  mocks.client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  mocks.client.setQueryData(["article", 1], article());
  vi.clearAllMocks();
});
afterEach(() => mocks.client?.clear());

describe("article state writes", () => {
  it.each([
    { view: "unread", field: "isRead", value: true, action: "setRead", api: "markRead" },
    { view: "starred", field: "isStarred", value: false, action: "setStarred", api: "markStarred" },
    { view: "readLater", field: "readLater", value: false, action: "setReadLater", api: "markReadLater" },
  ] as const)("updates $view safely with read-only cached rows", async ({ view, field, value, action, api }) => {
    const client = mocks.client!;
    const original = Object.freeze({ ...article(), isStarred: true, readLater: true });
    const pages = Object.freeze([Object.freeze([original])]);
    const key = ["articles", { kind: view }, false, false];
    client.setQueryData(key, Object.freeze({ pages, pageParams: [0] }));
    client.setQueryData(["article", 1], original);
    const onError = vi.fn();
    mocks[api].mockResolvedValueOnce(undefined);

    await useArticleActions(onError)[action](1, value);

    expect(mocks[api]).toHaveBeenCalledWith(1, value);
    expect(onError).not.toHaveBeenCalled();
    const updated = client.getQueryData<{ pages: ArticleSummary[][] }>(key)!;
    expect(updated.pages[0][0][field]).toBe(value);
    expect(client.getQueryData<ArticleSummary>(["article", 1])?.[field]).toBe(value);
    expect(original).toMatchObject({ isRead: false, isStarred: true, readLater: true });
    expect(pages[0]).toEqual([original]);
  });

  it("keeps a successful read change after an older detail query returns", async () => {
    const client = mocks.client!;
    const query = deferred<ArticleSummary>();
    const write = deferred();
    mocks.markRead.mockReturnValueOnce(write.promise);
    const fetch = client.fetchQuery({ queryKey: ["article", 1], queryFn: () => query.promise })
      .catch(() => undefined);
    const change = useArticleActions().setRead(1, true);
    expect(client.getQueryData<ArticleSummary>(["article", 1])?.isRead).toBe(true);
    query.resolve(article());
    await fetch;
    expect(client.getQueryState(["article", 1])).toMatchObject({
      status: "success", error: null, fetchStatus: "idle",
    });
    write.resolve();
    await change;
    expect(client.getQueryData<ArticleSummary>(["article", 1])?.isRead).toBe(true);
  });

  it("reconciles a stale list fetch started while the write is pending", async () => {
    const client = mocks.client!;
    const key = ["articles", "all"];
    client.setQueryData(key, { pages: [[article()]], pageParams: [0] });
    const query = deferred<{ pages: ArticleSummary[][]; pageParams: number[] }>();
    const write = deferred();
    mocks.markRead.mockReturnValueOnce(write.promise);
    const change = useArticleActions().setRead(1, true);
    const fetch = client.fetchQuery({ queryKey: key, queryFn: () => query.promise })
      .catch(() => undefined);
    query.resolve({ pages: [[article()]], pageParams: [0] });
    await fetch;
    write.resolve();
    await change;
    const data = client.getQueryData<{ pages: ArticleSummary[][] }>(key);
    expect(data?.pages[0][0].isRead).toBe(true);
  });

  it("restores the confirmed value when two overlapping writes both fail", async () => {
    const first = deferred();
    const second = deferred();
    mocks.markStarred.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    const onError = vi.fn();
    const one = useArticleActions(onError).setStarred(1, true);
    const two = useArticleActions(onError).setStarred(1, false);
    await vi.waitFor(() => expect(mocks.markStarred).toHaveBeenCalled());
    first.reject(new Error("first rejected"));
    await one;
    await vi.waitFor(() => expect(mocks.markStarred.mock.calls.length).toBeGreaterThanOrEqual(2));
    second.reject(new Error("second rejected"));
    await two;
    expect(mocks.client!.getQueryData<ArticleSummary>(["article", 1])?.isStarred).toBe(false);
    expect(onError).toHaveBeenCalledTimes(2);
  });

  it("keeps the latest true intent when an earlier true write fails", async () => {
    const jobs = [deferred(), deferred(), deferred()];
    for (const job of jobs) mocks.markStarred.mockReturnValueOnce(job.promise);
    const one = useArticleActions().setStarred(1, true);
    const two = useArticleActions().setStarred(1, false);
    const three = useArticleActions().setStarred(1, true);
    await vi.waitFor(() => expect(mocks.markStarred).toHaveBeenCalled());
    jobs[0].reject(new Error("first rejected"));
    await one;
    expect(mocks.client!.getQueryData<ArticleSummary>(["article", 1])?.isStarred).toBe(true);
    await vi.waitFor(() => expect(mocks.markStarred.mock.calls.length).toBeGreaterThanOrEqual(2));
    jobs[1].resolve();
    await two;
    await vi.waitFor(() => expect(mocks.markStarred.mock.calls.length).toBeGreaterThanOrEqual(3));
    jobs[2].resolve();
    await three;
    expect(mocks.client!.getQueryData<ArticleSummary>(["article", 1])?.isStarred).toBe(true);
  });

  it("keeps a later pending intent visible and rolls it back to the last success", async () => {
    const first = deferred();
    const second = deferred();
    mocks.markReadLater.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    const one = useArticleActions().setReadLater(1, true);
    const two = useArticleActions().setReadLater(1, false);
    await vi.waitFor(() => expect(mocks.markReadLater).toHaveBeenCalled());
    first.resolve();
    await one;
    expect(mocks.client!.getQueryData<ArticleSummary>(["article", 1])?.readLater).toBe(false);
    await vi.waitFor(() => expect(mocks.markReadLater.mock.calls.length).toBeGreaterThanOrEqual(2));
    second.reject(new Error("second rejected"));
    await two;
    expect(mocks.client!.getQueryData<ArticleSummary>(["article", 1])?.readLater).toBe(true);
  });

  it("does not let a query hide pending flags on different articles", async () => {
    const client = mocks.client!;
    const key = ["articles", "all"];
    client.setQueryData(key, { pages: [[article(), article(2)]], pageParams: [0] });
    const first = deferred();
    const second = deferred();
    mocks.markRead.mockReturnValueOnce(first.promise);
    mocks.markStarred.mockReturnValueOnce(second.promise);
    const one = useArticleActions().setRead(1, true);
    const two = useArticleActions().setStarred(2, true);
    await client.fetchQuery({ queryKey: key, queryFn: async () => ({ pages: [[article(), article(2)]], pageParams: [0] }) });
    const data = client.getQueryData<{ pages: ArticleSummary[][] }>(key)!;
    expect(data.pages[0][0].isRead).toBe(true);
    expect(data.pages[0][1].isStarred).toBe(true);
    first.resolve();
    second.resolve();
    await Promise.all([one, two]);
  });

  it("lets different fields finish independently without undoing a success", async () => {
    const read = deferred();
    const star = deferred();
    mocks.markRead.mockReturnValueOnce(read.promise);
    mocks.markStarred.mockReturnValueOnce(star.promise);
    const one = useArticleActions().setRead(1, true);
    const two = useArticleActions().setStarred(1, true);
    await vi.waitFor(() => expect(mocks.markRead).toHaveBeenCalled());
    await vi.waitFor(() => expect(mocks.markStarred).toHaveBeenCalled());
    read.resolve();
    await one;
    star.reject(new Error("star rejected"));
    await two;
    expect(mocks.client!.getQueryData<ArticleSummary>(["article", 1])).toMatchObject({ isRead: true, isStarred: false });
  });
});
