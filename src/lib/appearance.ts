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

// Shared by the document theme and the preset previews. Hues come from the
// reference palettes (Catppuccin, Nord, Everforest, Gruvbox, Rose Pine); the
// lightness is papr's own, targeted at >=4.4:1 on --paper since accents paint
// link text and not just dots and rules. (clay, the original, sits just under.)
export const ACCENTS: Record<
  Accent,
  { accent: string; soft: string; ink: string; dAccent: string; dSoft: string; dInk: string }
> = {
  clay: { accent: "oklch(0.60 0.13 38)", soft: "oklch(0.94 0.04 50)", ink: "oklch(0.42 0.10 38)", dAccent: "oklch(0.74 0.13 45)", dSoft: "oklch(0.32 0.06 40)", dInk: "oklch(0.80 0.10 45)" },
  pine: { accent: "oklch(0.50 0.10 165)", soft: "oklch(0.94 0.04 160)", ink: "oklch(0.38 0.08 165)", dAccent: "oklch(0.72 0.11 170)", dSoft: "oklch(0.30 0.05 165)", dInk: "oklch(0.80 0.08 170)" },
  indigo: { accent: "oklch(0.52 0.14 268)", soft: "oklch(0.94 0.04 270)", ink: "oklch(0.40 0.12 268)", dAccent: "oklch(0.74 0.13 270)", dSoft: "oklch(0.30 0.06 268)", dInk: "oklch(0.82 0.10 270)" },
  ink: { accent: "oklch(0.30 0.02 50)", soft: "oklch(0.92 0.005 50)", ink: "oklch(0.20 0.01 50)", dAccent: "oklch(0.86 0.005 50)", dSoft: "oklch(0.30 0.005 50)", dInk: "oklch(0.92 0.005 50)" },
  mauve: { accent: "oklch(0.575 0.135 310)", soft: "oklch(0.94 0.04 310)", ink: "oklch(0.42 0.10 310)", dAccent: "oklch(0.775 0.125 310)", dSoft: "oklch(0.32 0.05 310)", dInk: "oklch(0.80 0.10 310)" },
  frost: { accent: "oklch(0.555 0.115 235)", soft: "oklch(0.94 0.04 235)", ink: "oklch(0.42 0.09 235)", dAccent: "oklch(0.755 0.11 235)", dSoft: "oklch(0.32 0.05 235)", dInk: "oklch(0.80 0.09 235)" },
  leaf: { accent: "oklch(0.55 0.115 148)", soft: "oklch(0.94 0.04 148)", ink: "oklch(0.42 0.09 148)", dAccent: "oklch(0.75 0.11 148)", dSoft: "oklch(0.32 0.05 148)", dInk: "oklch(0.80 0.09 148)" },
  amber: { accent: "oklch(0.565 0.115 75)", soft: "oklch(0.94 0.04 75)", ink: "oklch(0.42 0.09 75)", dAccent: "oklch(0.765 0.11 75)", dSoft: "oklch(0.32 0.05 75)", dInk: "oklch(0.80 0.09 75)" },
  rose: { accent: "oklch(0.575 0.145 5)", soft: "oklch(0.94 0.04 5)", ink: "oklch(0.42 0.10 5)", dAccent: "oklch(0.775 0.135 5)", dSoft: "oklch(0.32 0.05 5)", dInk: "oklch(0.80 0.10 5)" },
  slate: { accent: "oklch(0.56 0.03 260)", soft: "oklch(0.92 0.02 260)", ink: "oklch(0.42 0.02 260)", dAccent: "oklch(0.76 0.03 260)", dSoft: "oklch(0.32 0.01 260)", dInk: "oklch(0.80 0.02 260)" },
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
    id: "orchid",
    settings: { theme: "light", darkShade: "default", accent: "mauve", density: "spacious", viewMode: "card", readerFont: "serif", readerSize: 18, readerLeading: 170, readerWidth: 660 },
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
  {
    id: "glacier",
    settings: { theme: "dark", darkShade: "dimmer", accent: "frost", density: "cozy", viewMode: "list", readerFont: "sans", readerSize: 17, readerLeading: 165, readerWidth: 700 },
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
