// Temporary follow-up Q&A about a single AI-generated article summary.
// History lives only as long as the summary drawer is mounted; it is not
// persisted to the database.

import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "../api";
import { toast } from "../toast";
import type { AiEvent } from "../types";

export interface FollowUpQA {
  id: string;
  question: string;
  answer: string;
  status: "loading" | "done" | "error";
}

function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

export function useSummaryFollowUp(summaryText: string) {
  const [items, setItems] = useState<FollowUpQA[]>([]);
  const [asking, setAsking] = useState(false);
  const runRef = useRef(0);

  // Drop any temporary follow-ups when the underlying summary changes
  // (e.g. switching article or switching template).
  useEffect(() => {
    runRef.current += 1;
    setItems([]);
    setAsking(false);
  }, [summaryText]);

  const ask = useCallback(
    async (question: string) => {
      if (!question.trim() || asking) return;

      const history: [string, string][] = items
        .filter((i) => i.status === "done")
        .map((i) => [i.question, i.answer]);

      const id = generateId();
      setItems((prev) => [
        ...prev,
        { id, question: question.trim(), answer: "", status: "loading" },
      ]);
      setAsking(true);

      const run = ++runRef.current;
      let sawError = false;

      try {
        await api.aiSummarizeFollowUp(
          summaryText,
          history,
          question.trim(),
          (ev: AiEvent) => {
            if (runRef.current !== run) return;
            if (ev.type === "delta") {
              setItems((prev) =>
                prev.map((item) =>
                  item.id === id
                    ? { ...item, answer: item.answer + ev.data }
                    : item,
                ),
              );
            } else if (ev.type === "error") {
              sawError = true;
              setItems((prev) =>
                prev.map((item) =>
                  item.id === id ? { ...item, status: "error" } : item,
                ),
              );
              toast.error(ev.data);
            }
          },
        );

        if (runRef.current === run) {
          setItems((prev) =>
            prev.map((item) =>
              item.id === id && item.status === "loading"
                ? { ...item, status: "done" }
                : item,
            ),
          );
        }
      } catch (e) {
        if (!sawError) {
          setItems((prev) =>
            prev.map((item) =>
              item.id === id ? { ...item, status: "error" } : item,
            ),
          );
          reportError(e);
        }
      } finally {
        if (runRef.current === run) {
          setAsking(false);
        }
      }
    },
    [summaryText, items, asking],
  );

  const clear = useCallback(() => {
    runRef.current += 1;
    setItems([]);
    setAsking(false);
  }, []);

  return { items, asking, ask, clear };
}

function reportError(e: unknown) {
  const msg = e instanceof Error ? e.message : String(e);
  toast.error(msg || "AI follow-up failed");
}
