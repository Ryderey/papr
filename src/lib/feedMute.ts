import type { Feed } from "../types";

/** Durations offered by both the article and sidebar context menus. */
export const MUTE_DURATIONS = [
  { minutes: 60, labelKey: "feedMute.hour1" },
  { minutes: 1_440, labelKey: "feedMute.hour24" },
  { minutes: 10_080, labelKey: "feedMute.day7" },
  { minutes: 43_200, labelKey: "feedMute.day30" },
] as const;

export function isFeedMuted(
  mutedUntil: number | null | undefined,
  nowSec: number,
): boolean {
  return mutedUntil != null && mutedUntil > nowSec;
}

/** Nearest future mute expiry, or null when no feed is currently muted. */
export function nextMuteExpiry(
  feeds: readonly Pick<Feed, "mutedUntil">[],
  nowSec: number,
): number | null {
  let next: number | null = null;
  for (const feed of feeds) {
    const until = feed.mutedUntil;
    if (until != null && until > nowSec && (next == null || until < next)) {
      next = until;
    }
  }
  return next;
}

export function formatMuteUntil(mutedUntil: number, locale: string): string {
  return new Date(mutedUntil * 1000).toLocaleString(locale, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** Delay before the next expiry refresh. `0` means the mute has expired.
 *  Clamped to the 32-bit setTimeout ceiling; callers re-arm afterward. */
export function muteTimerDelayMs(expirySec: number, nowMs: number): number {
  const remaining = expirySec * 1000 - nowMs + 500;
  if (remaining <= 0) return 0;
  return Math.min(remaining, 2 ** 31 - 1);
}
