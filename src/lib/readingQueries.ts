import type { QueryClient } from "@tanstack/react-query";

export function refreshArticleQueries(qc: QueryClient, extra: string[][] = []) {
  const keys = [
    ["counts"],
    ["feeds"],
    ["folders"],
    ["tags"],
    ["articles"],
    ["article"],
    ["search"],
    ["cp-search"],
    ...extra,
  ];
  for (const key of keys) {
    qc.invalidateQueries({ queryKey: key });
  }
}
