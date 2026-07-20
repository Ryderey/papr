export const AI_DRAWER_BOUNDS = {
  min: 320,
  max: 640,
  default: 480,
} as const;

const READER_MIN_WIDTH = 520;
const SIDEBAR_WIDTH = 248;
const ARTICLE_LIST_WIDTH = 388;

export type AiLayoutMode = "all" | "without-list" | "reader-only";

interface AiLayoutInput {
  viewportWidth: number;
  preferredDrawerWidth: number;
  open: boolean;
}

export interface AiLayout {
  mode: AiLayoutMode;
  drawerWidth: number;
  maxDrawerWidth: number;
}

const clamp = (value: number, min: number, max: number) =>
  Math.min(max, Math.max(min, value));

/** Keep the preferred drawer width while progressively yielding browse chrome. */
export function resolveAiLayout({
  viewportWidth,
  preferredDrawerWidth,
  open,
}: AiLayoutInput): AiLayout {
  const preferred = clamp(
    preferredDrawerWidth,
    AI_DRAWER_BOUNDS.min,
    AI_DRAWER_BOUNDS.max,
  );

  let mode: AiLayoutMode = "all";
  let chromeWidth = SIDEBAR_WIDTH + ARTICLE_LIST_WIDTH;

  if (open && viewportWidth - chromeWidth < READER_MIN_WIDTH + preferred) {
    mode = "without-list";
    chromeWidth = SIDEBAR_WIDTH;
  }
  if (open && viewportWidth - chromeWidth < READER_MIN_WIDTH + preferred) {
    mode = "reader-only";
    chromeWidth = 0;
  }

  const maxDrawerWidth = clamp(
    viewportWidth - chromeWidth - READER_MIN_WIDTH,
    AI_DRAWER_BOUNDS.min,
    AI_DRAWER_BOUNDS.max,
  );

  return {
    mode,
    drawerWidth: Math.min(preferred, maxDrawerWidth),
    maxDrawerWidth,
  };
}
