# On-demand Windows / Android Release packaging

## Requirements

- The user approved the design in `docs/ci-packaging-design-2026-10-05.md`.
- Daily master push / PR checks cover frontend, Rust workspace and Flutter without creating installers, APKs or Releases.
- A manual workflow selects Windows, Android or both and publishes complete assets to GitHub Release for long-term download.
- Every selected platform uses the same immutable commit SHA. A failed selected platform prevents publication.
- Android APKs use a persistent private signing identity, require explicit increasing versionCode, and are distributed directly without Google Play.
- Existing local Android ABI-selection work and `papr-release.keystore` must be preserved. Never print or commit credentials.
- Do not silently replace published assets or move existing tags. Scope inherited upstream-only workflows to the upstream repository.
- Document signing setup, manual runs, installation, migration and local-space cleanup boundaries.

## Acceptance

- Workflow static validation and release-helper regression tests pass.
- Existing frontend / Flutter checks pass; Gradle release signing is checked without private secret output.
- Actual cloud build, signed installation and coverage update evidence are reported separately from local/static checks.
- Source files contain no keystore/password/Token material; no local data or caches are deleted automatically.

## External prerequisites

The owner confirmed reuse of the existing ignored `papr-release.keystore`. GitHub signing Secrets remain unconfigured until the owner runs `pwsh -NoProfile -File .\scripts\configure-android-signing.ps1` with concealed password input. Do not request passwords in chat. Actual Android cloud packaging and device upgrade acceptance remain pending this prerequisite.
