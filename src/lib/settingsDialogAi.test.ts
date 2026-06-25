import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const settingsDialogSource = readFileSync(
  fileURLToPath(new URL("../components/SettingsDialog.tsx", import.meta.url)),
  "utf8",
);

function functionSource(name: string, nextName: string): string {
  const start = settingsDialogSource.indexOf(`const ${name} =`);
  const end = settingsDialogSource.indexOf(`const ${nextName} =`, start + 1);

  if (start === -1 || end === -1) {
    throw new Error(`Unable to locate ${name} in SettingsDialog.tsx`);
  }

  return settingsDialogSource.slice(start, end);
}

describe("AI settings save boundary", () => {
  it("does not expose profile or provider preset controls", () => {
    expect(settingsDialogSource).not.toContain('className="ai-profile-actions"');
    expect(settingsDialogSource).not.toContain('className="ai-preset-strip"');
    expect(settingsDialogSource).not.toContain('t("settings.advanced.aiCreateProfile")');
    expect(settingsDialogSource).not.toContain('t("settings.advanced.aiDeleteProfile")');
  });

  it("does not persist the draft when testing the connection", () => {
    const testCurrentProfileSource = functionSource(
      "testCurrentProfile",
      "save",
    );

    expect(testCurrentProfileSource).not.toContain("persistProfileSettings(");
  });

  it("persists AI settings only from the explicit save handler", () => {
    expect(settingsDialogSource.match(/persistProfileSettings\(/g)).toHaveLength(1);
  });
});
