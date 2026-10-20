import { describe, expect, it } from "vitest";
import { isFeedMuted, muteTimerDelayMs, nextMuteExpiry } from "./feedMute";

describe("isFeedMuted", () => {
  it("treats a future timestamp as muted", () => {
    expect(isFeedMuted(1_001, 1_000)).toBe(true);
  });

  it("treats a past or equal timestamp as expired", () => {
    expect(isFeedMuted(999, 1_000)).toBe(false);
    expect(isFeedMuted(1_000, 1_000)).toBe(false);
  });

  it("treats no timestamp as unmuted", () => {
    expect(isFeedMuted(null, 1_000)).toBe(false);
    expect(isFeedMuted(undefined, 1_000)).toBe(false);
  });
});

describe("nextMuteExpiry", () => {
  it("returns the earliest future expiry", () => {
    expect(
      nextMuteExpiry(
        [{ mutedUntil: 1_300 }, { mutedUntil: 1_100 }, { mutedUntil: null }],
        1_000,
      ),
    ).toBe(1_100);
  });

  it("ignores expired mutes", () => {
    expect(
      nextMuteExpiry([{ mutedUntil: 900 }, { mutedUntil: 1_000 }], 1_000),
    ).toBeNull();
  });

  it("returns null when no feed is muted", () => {
    expect(nextMuteExpiry([{ mutedUntil: null }], 1_000)).toBeNull();
  });
});

describe("muteTimerDelayMs", () => {
  it("returns 0 once the expiry has passed", () => {
    expect(muteTimerDelayMs(1_000, 1_000_500)).toBe(0);
  });

  it("returns the remaining delay for a near expiry", () => {
    expect(muteTimerDelayMs(1_000, 0)).toBe(1_000_500);
  });

  it("clamps the 30-day preset below the setTimeout ceiling", () => {
    expect(muteTimerDelayMs(43_200 * 60, 0)).toBe(2 ** 31 - 1);
  });
});
