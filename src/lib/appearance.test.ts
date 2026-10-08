import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { APPEARANCE_PRESETS, matchingAppearancePreset } from "./appearance";
import * as api from "../api";

vi.mock("../api", () => ({ setSetting: vi.fn().mockResolvedValue(undefined) }));
vi.mock("../i18n", () => ({ default: { t: (key: string) => key } }));

beforeEach(() => {
  vi.resetModules();
  vi.clearAllMocks();
  const values = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, String(value)),
    removeItem: (key: string) => values.delete(key),
  });
});

afterEach(() => vi.unstubAllGlobals());

describe("appearance preset preferences", () => {
  it("keeps an existing custom appearance on startup", async () => {
    localStorage.setItem("theme", "dark");
    localStorage.setItem("accent", "ink");
    localStorage.setItem("readerSize", "21");
    const { useUi } = await import("../store");

    expect(useUi.getState()).toMatchObject({ theme: "dark", accent: "ink", readerSize: 21 });
    expect(matchingAppearancePreset(useUi.getState())).toBeNull();
  });

  it("applies each preset in one state update and persists every controlled value", async () => {
    const { useUi } = await import("../store");
    const listener = vi.fn();
    const unsubscribe = useUi.subscribe(listener);
    useUi.getState().setPref({ reduceMotion: true, markReadOnOpen: false });
    useUi.getState().openArticle(42);
    listener.mockClear();

    for (const preset of APPEARANCE_PRESETS) {
      useUi.getState().applyAppearancePreset(preset.id);
      expect(useUi.getState()).toMatchObject(preset.settings);
      expect(matchingAppearancePreset(useUi.getState())).toBe(preset.id);
      for (const [key, value] of Object.entries(preset.settings)) {
        expect(localStorage.getItem(key)).toBe(String(value));
      }
      expect(api.setSetting).toHaveBeenCalledWith("theme", preset.settings.theme);
      expect(api.setSetting).toHaveBeenCalledWith("dark_shade", preset.settings.darkShade);
    }

    expect(listener).toHaveBeenCalledTimes(APPEARANCE_PRESETS.length);
    expect(useUi.getState().prefs).toMatchObject({ reduceMotion: true, markReadOnOpen: false });
    expect(useUi.getState().selectedArticleId).toBe(42);
    unsubscribe();
  });

  it("restores the preset after reloading the store", async () => {
    const { useUi } = await import("../store");
    useUi.getState().applyAppearancePreset("midnight");
    vi.resetModules();
    const { useUi: restored } = await import("../store");

    expect(restored.getState()).toMatchObject(APPEARANCE_PRESETS.find((p) => p.id === "midnight")!.settings);
    expect(matchingAppearancePreset(restored.getState())).toBe("midnight");
  });

  it("reflects manual typography or theme changes instead of retaining a stale selected name", async () => {
    const { useUi } = await import("../store");
    useUi.getState().applyAppearancePreset("paper");
    useUi.getState().setReader({ readerSize: 19 });
    expect(matchingAppearancePreset(useUi.getState())).toBeNull();
    useUi.getState().setReader({ readerSize: 17 });
    useUi.getState().setDarkShade("black");
    expect(matchingAppearancePreset(useUi.getState())).toBe("paper");
    useUi.getState().setTheme("dark");
    expect(matchingAppearancePreset(useUi.getState())).toBeNull();
  });
});
