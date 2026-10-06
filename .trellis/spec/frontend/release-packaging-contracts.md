# Manual Release Packaging Contracts

## 1. Scope / Trigger

Use for CI, installer/APK collection, GitHub Release publication and Android
signing. Daily CI performs checks only; application packages require an explicit
`workflow_dispatch` in `package-release.yml`.

## 2. Signatures

```text
package_release.py prepare | collect windows/android | publish
configure-android-signing.ps1 [-KeystorePath path] [-KeyAlias alias] [-Repository owner/repo]
workflow inputs: platforms, release_tag, prerelease, android_build_number
Gradle: -PpaprAndroidAbis=arm64-v8a,armeabi-v7a -PpaprRequireReleaseSigning=true
```

## 3. Contracts

- Every job validates/builds the same full SHA. Reuse `ci.yml` with `workflow_call`
  and its `ref` input; do not use a branch's moving HEAD in downstream jobs.
- Android `settings.gradle.kts` must declare plugin repositories in
  `pluginManagement`: Google Maven for AGP, Maven Central and the Gradle Plugin
  Portal. Project dependency repositories do not configure plugin resolution.
  A cached local Gradle configuration check cannot establish fresh-runner
  availability; real cloud packaging must verify that boundary.
- Pin pnpm 11.5.0 in package.json; setup actions read that single source, matching local validation.
  The build-script allowlist in pnpm-workspace.yaml is incompatible with the old
  pnpm 9 setup, which failed `pnpm store path` with a missing packages-field error.
- Validate `papr-build-*` tags with both a restricted alphabet and Git ref rules;
  Android versionCode is explicit, > pubspec code and <= 2100000000.
- GitHub API 404 may mean missing; authentication, quota and server failures
  abort. Peel annotated tags and refuse a conflicting tag or published Release.
- All selected platform jobs must succeed before publication. Skipped unselected
  jobs do not block it. Exact staging inventory, hashes, sizes and build metadata
  must match before any write. Draft recovery accepts only identical asset hashes;
  never upload with `--clobber` or move a tag.
- Normal jobs use contents:read; only the publisher gets contents:write. PR checks
  receive no signing Secrets. Signing paths never enter cache/artifact inventories.
- Android CI consumes all four PAPR_ANDROID_* signing values from step environment,
  restored from ANDROID_KEYSTORE_* / ANDROID_KEY_* Secrets. Local key.properties
  is an alternative. Incomplete configuration fails; keyless local builds may use
  Debug signing, but required CI signing must never fall back.
- Verify APK package, version, expected split ABI, bridge library and certificate
  against public `ANDROID_SIGNING_CERT_SHA256` before uploading artifacts.
- Key setup preserves an existing keystore, verifies store/key passwords, sends
  Secrets via stdin without a trailing newline, and backs up no credentials into
  source. Quote PowerShell's `-J-Duser.language=en` / password modifier arguments.
  If keytool explicitly ignores a separate PKCS12 key password, save the store
  password that actually signed the CSR, not the ignored input. Root keystore
  remains ignored and is not disposable build cache.
- Release assets are the long-term download source; staging artifacts expire in
  three days. Upload installers only; platform metadata remains in staging and
  SHA-256 values go in release notes. Titles use Papr plus app version; filenames
  use version/platform/ABI without tag dates or commit suffixes. Immutable tags
  and metadata still identify the exact SHA and Android versionCode.
  No automatic app updater or automatic local cache deletion.
- Runtime deep-link registration first checks `is_registered`; skip writes when
  already registered or when the check fails. Preserve `papr://` subscriptions
  for unbundled builds instead of disabling the feature to suppress AV prompts.

## 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Invalid tag, SHA or Android build number | Fail before building/publishing |
| Existing tag points elsewhere / Release is published | Refuse; require new tag |
| Selected platform fails | No completed Release publication |
| Missing / altered / unexpected staged asset | Fail before GitHub writes |
| Existing draft asset digest missing/different | Refuse replacement |
| CI signing Secret missing / certificate mismatch | Fail before upload |
| Installed app has a different signature | No direct update claim; preserve data and plan migration |

## 5. Good / Base / Bad Cases

Good: manual both-platform build validates one SHA and publishes one complete
Release. Base: Windows-only build skips APK/signing but still validates code.
Bad: every push creates packages, a partial matrix publishes success, or fresh
runner Debug keys are called a persistent signing identity.

## 6. Tests Required

Run `python -B -m unittest discover -s scripts/tests -p 'test_*.py'`, actionlint,
PowerShell syntax checks, Gradle configuration with/without required signing, and
existing frontend/Flutter checks. Tests cover injection/ref rejection, versionCode
bounds, tag conflicts, API errors, annotated tags, partial/tampered assets and
draft conflicts. Real signing setup, cloud builds, download/install and in-place
updates require separate owner/device evidence.

Release lookup tests must model the real API: `releases/tags/{tag}` only returns
published releases. Find drafts through the paginated releases list and require
their `target_commitish` to equal the immutable build SHA. A draft's Git tag may
not exist until publication; recheck existing tag conflicts before publishing,
then verify the resulting tag after publication.

## 7. Wrong vs Correct

Wrong: release workflow writes key.properties but Gradle always picks Debug.
Correct: complete signing values select the persistent key; CI requires it and
checks the built APK certificate before publication.

Wrong: new tag indicates success before packaging finished, or existing assets
are silently clobbered. Correct: one guarded publisher stages a draft after all
selected builds pass, verifies all uploaded digests, then publishes it.
