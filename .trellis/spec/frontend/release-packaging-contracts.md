# Manual Release Packaging Contracts

## 1. Scope / Trigger

Use for CI, installer/APK collection, GitHub Release publication and Android
signing. Daily CI performs checks only; application packages require an explicit
`workflow_dispatch` in `package-release.yml`.

The maintained workflow inventory is `ci.yml` and `package-release.yml` only.
Inherited Claude, desktop release and Homebrew workflows, plus the superseded
Android release draft, are removed. Keep the reusable CI dependency intact;
tag pushes alone must not build or publish application packages.

## 2. Signatures

```text
package_release.py prepare | collect windows/android | publish
app_version.py check | set <MAJOR.MINOR.PATCH> --android-code <positive integer>
configure-android-signing.ps1 [-KeystorePath path] [-KeyAlias alias] [-Repository owner/repo]
workflow inputs: platforms, release_tag, prerelease, android_build_number
prepare outputs: sha, android_build_number (resolved source/default or override)
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
- Root `Cargo.toml` workspace version is canonical. All local Rust packages
  inherit it; package.json, Tauri JSON, pubspec display version and all three local
  Cargo.lock package entries must agree. `app_version.py` requires Python >=3.11
  for stdlib TOML support; CI pins 3.12. Updates validate the entire batch, preserve
  formatting/line endings/third-party versions and restore files on write errors.
  Rollback skips untouched paths, tries all changed paths and reports any path
  it cannot restore; callers must not treat that failure as a completed update.
- New tags are exactly `papr-v<product-version>` or `papr-v<product-version>-rc.N`
  (positive N), validated with Git ref rules. RC requires prerelease=true; stable
  requires false. Existing `papr-build-*` releases remain historical.
- Android code defaults to pubspec. An override is >= source, positive and
  <=2100000000; the resolved value flows from prepare to Android and publisher.
  Prepare and publication scan all pages of published releases with APK assets;
  parse exactly one script-owned `Android: ... (versionCode N)` line. Missing or
  ambiguous metadata fails closed, drafts are ignored, and reused/lower published
  codes fail. Publisher jobs serialize across tags so the final recheck prevents
  two releases racing the same build number. Credentials/API failures still abort.
- Decode CLI/API subprocess output explicitly as UTF-8. Windows GBK defaults
  cannot reliably decode international Release notes; do not rely on shell locale.
- Direct-distribution Release splits must retain that exact versionCode. Flutter
  3.44.5 otherwise adds ABI offsets (1000/2000), and does not implement the newer
  force-version-code-ignoring-abi switch. Register the Release `configureEach`
  callback after Flutter's callback, before AGP finalizes properties; a later
  afterEvaluate mutation fails and an earlier onVariants value is overwritten.
  `scripts/tests/verify_android_version.gradle` checks final versionCode providers
  with split-per-abi and both target platforms. Store publishing is outside scope.
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
  Final stable publication uses latest=true; RC uses latest=false. Draft creation
  always stays non-Latest. Default manual channel is stable, not prerelease.
  No automatic app updater or automatic local cache deletion.
- Runtime deep-link registration first checks `is_registered`; skip writes when
  already registered or when the check fails. Preserve `papr://` subscriptions
  for unbundled builds instead of disabling the feature to suppress AV prompts.

## 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Invalid tag, SHA or Android build number | Fail before building/publishing |
| Product manifests/lock drift or tag/channel mismatch | Offline CI/preflight reject |
| Code equals source but exceeds all published Android codes | Accept source/default code |
| Code reused, below source, or historical APK code unreadable | Reject before publishing |
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

Version tests cover normalization/inheritance, third-party lock preservation,
CRLF preservation, malformed input/manifest no-write, drift and write rollback.
Include partial writes and persistent permission denial, plus UTF-8 subprocess
output under a simulated GBK locale.
Release tests cover source defaults/overrides, channel matching, paginated APK
history including draft-ID recovery, fail-closed notes parsing, prepare outputs,
and stable/RC Latest behavior. Keep existing immutable-asset/draft recovery tests.

Release lookup tests must model the real API: `releases/tags/{tag}` only returns
published releases. Find drafts through the paginated releases list and require
their `target_commitish` to equal the immutable build SHA. A draft's Git tag may
not exist until publication; recheck existing tag conflicts before publishing,
then verify the resulting tag after publication.
Create via the REST response and retain its release ID for subsequent checks;
fresh draft creation may not immediately appear in the paginated list. Tests
must keep that list empty after creation while the ID endpoint remains valid.

## 7. Wrong vs Correct

Wrong: release workflow writes key.properties but Gradle always picks Debug.
Correct: complete signing values select the persistent key; CI requires it and
checks the built APK certificate before publication.

Wrong: new tag indicates success before packaging finished, or existing assets
are silently clobbered. Correct: one guarded publisher stages a draft after all
selected builds pass, verifies all uploaded digests, then publishes it.
