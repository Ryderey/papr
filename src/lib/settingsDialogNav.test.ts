import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const settingsDialogSource = readFileSync(
  fileURLToPath(new URL("../components/SettingsDialog.tsx", import.meta.url)),
  "utf8",
);

describe("settings section nav", () => {
  // The section rows used to be clickable <div>s, which Tab could not reach —
  // keyboard users could not switch sections at all. Keep them real buttons.
  it("renders section rows as buttons, not divs", () => {
    expect(settingsDialogSource).toMatch(/<button[\s\S]{0,200}settings-nav-item/);
    expect(settingsDialogSource).not.toMatch(/<div[^>]*className=\{`settings-nav-item/);
  });
});
