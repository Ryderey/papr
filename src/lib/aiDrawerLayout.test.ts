import { describe, expect, it } from "vitest";
import { resolveAiLayout } from "./aiDrawerLayout";

describe("resolveAiLayout", () => {
  it("keeps all panes when the preferred drawer and reader fit", () => {
    expect(
      resolveAiLayout({ viewportWidth: 1920, preferredDrawerWidth: 480, open: true }),
    ).toEqual({ mode: "all", drawerWidth: 480, maxDrawerWidth: 640 });
  });

  it("hides the article list before the sidebar", () => {
    expect(
      resolveAiLayout({ viewportWidth: 1280, preferredDrawerWidth: 480, open: true }),
    ).toEqual({ mode: "without-list", drawerWidth: 480, maxDrawerWidth: 512 });
  });

  it("hides both browsing panes near the minimum window width", () => {
    expect(
      resolveAiLayout({ viewportWidth: 1024, preferredDrawerWidth: 480, open: true }),
    ).toEqual({ mode: "reader-only", drawerWidth: 480, maxDrawerWidth: 504 });
  });

  it("temporarily clamps the drawer so the reader keeps 520px", () => {
    expect(
      resolveAiLayout({ viewportWidth: 920, preferredDrawerWidth: 480, open: true }),
    ).toEqual({ mode: "reader-only", drawerWidth: 400, maxDrawerWidth: 400 });
  });

  it("does not collapse browsing panes while the drawer is closed", () => {
    expect(
      resolveAiLayout({ viewportWidth: 920, preferredDrawerWidth: 480, open: false }),
    ).toEqual({ mode: "all", drawerWidth: 320, maxDrawerWidth: 320 });
  });
});
