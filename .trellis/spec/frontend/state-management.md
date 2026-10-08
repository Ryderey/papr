# State Management

> How state is managed in this project.

---

## Overview

<!--
Document your project's state management conventions here.

Questions to answer:
- What state management solution do you use?
- How is local vs global state decided?
- How do you handle server state?
- What are the patterns for derived state?
-->

(To be filled by the team)

---

## State Categories

<!-- Local state, global state, server state, URL state -->

(To be filled by the team)

---

## When to Use Global State

<!-- Criteria for promoting state to global -->

(To be filled by the team)

---

## Server State

<!-- How server data is cached and synchronized -->

(To be filled by the team)

---

## Common Mistakes

<!-- State management mistakes your team has made -->

(To be filled by the team)

## Desktop appearance presets

### Scope

Coordinated appearance presets use the existing Zustand preferences in `src/store.ts` and the definitions in `src/lib/appearance.ts`.

### Signatures

- `applyAppearancePreset(id: AppearancePresetId): void` persists and applies a preset.
- `matchingAppearancePreset(settings: AppearanceSettings): AppearancePresetId | null` derives selection from the current controls.

### Contracts

- Presets own `theme`, `darkShade`, `accent`, `density`, `viewMode`, `readerFont`, `readerSize`, `readerLeading`, and `readerWidth`.
- Persist those existing localStorage keys before one Zustand update. Mirror `theme` and `dark_shade` through the existing backend setting helpers.
- Do not persist a separate selected-preset identifier or apply a preset on startup: users may already have custom preferences.
- Manual adjustments update the same fields. A light theme ignores dormant `darkShade` when matching.
- UI and previews share `ACCENTS`. Native dark backing values must continue matching the reader surfaces in CSS and the native host.

### Validation and error behavior

Keep preset values within the existing enum options and reader slider bounds. An unknown preset ID is ignored. Existing preference readers continue validating persisted values. Backend theme mirroring remains best-effort, like manual theme changes.

### Examples

- Base: the default settings match `paper` without rewriting storage.
- Good: applying `midnight` persists the complete appearance and rehydrates after restart.
- Bad: changing reader size while still displaying the old preset name.

### Verification

`src/lib/appearance.test.ts` checks custom-startup preservation, a single state update, persistence, backend theme mirrors, rehydration, and manual override matching.

### Wrong vs correct

Wrong: save a preset label and use it as the selected state after independent controls change.

Correct: derive the label from all current preset-controlled values; show a custom appearance when they do not match.
