# Implementation and verification

1. Update reusable daily CI and root Rust cache paths.
2. Implement manual packaging workflow, guarded Release helper and focused regression checks.
3. Wire Android release signing and provide concealed local credential setup.
4. Guard legacy/duplicate workflows; update README, RC procedure and relevant mobile contract.
5. Run actionlint, Python helper tests, PowerShell syntax checks, frontend build/tests and Flutter analyze/tests. Use Gradle configuration checks for missing required signing and debug fallback; do not build repeated full APKs without reason.
6. Review the complete diff, report pending credential/cloud prerequisites, then prepare the task-scoped commit plan. Do not include unknown files without review or publish an actual Release before successful verification.

Baseline before implementation: frontend build and 89 tests pass; Flutter analyze and 40 tests pass. Existing workflows pass actionlint 1.7.12. Flutter is 3.44.5 (Dart 3.12.2). Existing keystore is preserved; no GitHub signing Secrets or Releases existed at that point.

## Delivered implementation

- `d2b3231`: manual packaging, guarded publisher, signing setup, docs and release contracts.
- `c65d642`: align CI with pnpm 11.5.0 required by the existing workspace configuration.
- `361a06f`: find draft releases through pagination, validate draft target SHA, check the published tag, and isolate test version fixtures.
- Default branch and tracking are master; inherited upstream main remains preserved.

## Verification

- Local frontend production build / 89 tests and Flutter analyze / 40 tests passed.
- Release regression suite: 10 tests passed after modeling the actual draft API semantics.
- actionlint 1.7.12 passed for all five affected workflows (external shellcheck/pyflakes disabled).
- PowerShell parser and isolated concealed-input setup simulation passed. No real signing Secrets were written by tests.
- Gradle configuration compiled; missing mandatory signing fails; a temporary fixture key binds the release signing configuration. The user's keystore was not used or changed.
- Real cloud ordinary CI [37345156802](https://github.com/Ryderey/papr/actions/runs/37345156802) passed all checks and created no installers/APKs/Releases.
- First cloud Windows builder succeeded, but publisher stopped at an empty draft because the by-tag REST endpoint cannot return drafts. After the fix, [37345208121](https://github.com/Ryderey/papr/actions/runs/37345208121) passed the entire Windows build/publication flow, with Android skipped as selected. [Prerelease papr-build-20261006-02](https://github.com/Ryderey/papr/releases/tag/papr-build-20261006-02) has the x64 installer and three checksum/metadata files. Its tag resolves to `361a06f0d5a7f4681817fd836a7c797936a37f34`.

All four Windows Release assets were downloaded and matched GitHub digests and sizes; checksum entries and build SHA/platform matched. The empty draft from the failed validation run was removed after checking its tag, target SHA, draft status and empty asset inventory. Temporary validation tools, fixture key and downloads are cleaned at session close; the user's keystore and existing build/SDK caches are retained.

## Remaining acceptance

Owner must run `pwsh -NoProfile -File .\scripts\configure-android-signing.ps1` in a local terminal using the existing keystore password. Android cloud build, APK download/install and fixed-signature in-place update are not yet verified. No Google Play or store submission is part of this task. Windows installation/startup also requires separate device evidence; downloading and hashing an installer does not establish runtime behavior.

Task remains in_progress; recording this session does not archive incomplete acceptance.
