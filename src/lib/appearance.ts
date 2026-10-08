import type { Accent, DarkShade, Density, ReaderFont, Theme, ViewMode } from "../store";

export interface AppearanceSettings {
  theme: Theme;
  darkShade: DarkShade;
  accent: Accent;
  density: Density;
  viewMode: ViewMode;
  readerFont: ReaderFont;
  readerSize: number;
  readerLeading: number;
  readerWidth: number;
}

// Shared by the document theme and the preset previews.
export const ACCENTS: Record<
  Accent,
  { accent: string; soft: string; ink: string; dAccent: string; dSoft: string; dInk: string }
> = {
  clay: { accent: "oklch(0.60 0.13 38)", soft: "oklch(0.94 0.04 50)", ink: "oklch(0.42 0.10 38)", dAccent: "oklch(0.74 0.13 45)", dSoft: "oklch(0.32 0.06 40)", dInk: "oklch(0.80 0.10 45)" },
  pine: { accent: "oklch(0.50 0.10 165)", soft: "oklch(0.94 0.04 160)", ink: "oklch(0.38 0.08 165)", dAccent: "oklch(0.72 0.11 170)", dSoft: "oklch(0.30 0.05 165)", dInk: "oklch(0.80 0.08 170)" },
  indigo: { accent: "oklch(0.52 0.14 268)", soft: "oklch(0.94 0.04 270)", ink: "oklch(0.40 0.12 268)", dAccent: "oklch(0.74 0.13 270)", dSoft: "oklch(0.30 0.06 268)", dInk: "oklch(0.82 0.10 270)" },
  ink: { accent: "oklch(0.30 0.02 50)", soft: "oklch(0.92 0.005 50)", ink: "oklch(0.20 0.01 50)", dAccent: "oklch(0.86 0.005 50)", dSoft: "oklch(0.30 0.005 50)", dInk: "oklch(0.92 0.005 50)" },
};

// Match the dark reader surfaces in styles.css and the native launch backing.
export const DARK_BACKING: Record<DarkShade, string> = {
  default: "#25201F",
  dimmer: "#1C1715",
  black: "#15100F",
};

export const APPEARANCE_PRESETS = [
  {
    id: "paper",
    settings: { theme: "light", darkShade: "default", accent: "clay", density: "cozy", viewMode: "list", readerFont: "serif", readerSize: 17, readerLeading: 165, readerWidth: 680 },
  },
  {
    id: "pine",
    settings: { theme: "light", darkShade: "default", accent: "pine", density: "spacious", viewMode: "card", readerFont: "serif", readerSize: 18, readerLeading: 175, readerWidth: 680 },
  },
  {
    id: "ink",
    settings: { theme: "light", darkShade: "default", accent: "ink", density: "compact", viewMode: "list", readerFont: "sans", readerSize: 16, readerLeading: 155, readerWidth: 720 },
  },
  {
    id: "dusk",
    settings: { theme: "dark", darkShade: "default", accent: "clay", density: "cozy", viewMode: "list", readerFont: "serif", readerSize: 18, readerLeading: 175, readerWidth: 680 },
  },
  {
    id: "midnight",
    settings: { theme: "dark", darkShade: "dimmer", accent: "indigo", density: "cozy", viewMode: "card", readerFont: "sans", readerSize: 17, readerLeading: 165, readerWidth: 720 },
  },
  {
    id: "focus",
    settings: { theme: "dark", darkShade: "black", accent: "pine", density: "compact", viewMode: "list", readerFont: "sans", readerSize: 17, readerLeading: 170, readerWidth: 640 },
  },
] as const satisfies readonly { id: string; settings: AppearanceSettings }[];

export type AppearancePresetId = (typeof APPEARANCE_PRESETS)[number]["id"];

/** Match the actual controls, rather than persisting a stale preset label. */
export function matchingAppearancePreset(settings: AppearanceSettings): AppearancePresetId | null {
  return APPEARANCE_PRESETS.find((preset) =>
    (Object.keys(preset.settings) as (keyof AppearanceSettings)[]).every((key) =>
      key === "darkShade" && settings.theme === "light"
        ? true
        : settings[key] === preset.settings[key],
    ),
  )?.id ?? null;
}
