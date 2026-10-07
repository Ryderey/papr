<div align="center">

<img src="docs/logo.svg" alt="Papr" width="96" height="96" />

# Papr

A local-first RSS reader for Windows and Android.

**English** · [简体中文](README.zh-CN.md)

<img src="docs/screenshot.webp" alt="Papr reader" width="820" />

</div>

This fork builds on [l0ng-ai/papr](https://github.com/l0ng-ai/papr) 0.9.0, with an Android client, GitHub repository sync, multiple AI profiles, summary templates, and follow-up questions. The maintained application targets in this repository are **Windows x64 and Android**.

## Features

- Feeds and folders, OPML import/export, and All, Unread, Starred, and Read Later views.
- Full-text extraction, tags, automatic tagging rules, highlights, and audio playback.
- AI summaries, summary follow-up questions, and LLM translation using your own API credentials. Profiles support Anthropic Messages and OpenAI Chat Completions-compatible endpoints.
- Multiple summary templates, connection testing, and a resizable AI panel on desktop.
- Optional GitHub private-repository sync, plus FreshRSS and Miniflux integrations.
- Local SQLite storage and English, Japanese, and Simplified Chinese interfaces.

Windows additionally provides global Ask/RAG, digests, Newsletter/IMAP, Send to Kindle, and desktop tray and shortcut interactions. These features are not available in the Android client. Android provides system sharing, scheduled background work, and background audio playback.

<p align="center">
  <img src="docs/AI摘要及追问.webp" alt="AI summary and follow-up questions" width="960" />
</p>

## Installation

Download packages from [the latest release](https://github.com/Ryderey/papr/releases/latest), or browse [all releases](https://github.com/Ryderey/papr/releases) for prereleases.

| Platform | Package |
| --- | --- |
| Windows x64 | `*-windows-x64-setup.exe` |
| Android ARM64 | `*-android-arm64-v8a.apk` — most modern ARM phones |
| Android ARM32 | `*-android-armeabi-v7a.apk` — 32-bit ARM devices |

Release notes contain version information, SHA-256 checksums, and build provenance. Updates are installed manually. Android in-place updates require a compatible signing certificate and version code.

## Synchronization

Cloud sync is optional. GitHub sync connects Windows and Android clients to the same private repository without a separate server or GitHub Actions. FreshRSS and Miniflux use their respective server integrations; the scope below describes **GitHub sync**. Only one sync provider can be connected at a time.

### What GitHub sync includes

| Data | Synced? | Scope |
| --- | --- | --- |
| Subscriptions | Yes | Feed URL, source type, display title, folder assignment, and subscription/unsubscription state |
| Folders | Yes | Names and folder order |
| Article catalog | Yes | Article identity, title, link, publication time, and first-seen time |
| Reading state | Yes | Read/unread, starred, and Read Later |
| Article bodies and cached content | No | Each device fetches content independently |
| Tags, tagging rules, and highlights | No | Remain on the device where they were created |
| AI profiles and credentials | No | API keys, endpoints, models, custom headers, and summary preferences remain local |
| AI-generated content | No | Summaries, translations, and Q&A history are not part of the sync payload |
| Appearance and other app settings | No | Theme, fonts, language, shortcuts, notifications, and refresh preferences remain local |
| Sync credentials and timing | No | Configure the token and automatic-sync schedule separately on each device |

Eligible source types are RSS, YouTube, podcast, Mastodon, Bluesky, and Reddit, with valid HTTP(S) feed URLs. Newsletter/IMAP and entries outside the supported source or metadata format are excluded; connection preview reports excluded counts.

Ordinary article catalog entries are retained for **90 days**. Entries that are starred or in Read Later are retained long-term while either flag remains set.

### Connect a repository

1. Create a dedicated **private repository** under your GitHub account, for example `papr-sync`. Initialize it with a README so its default branch exists.
2. Open [GitHub's fine-grained token creation page](https://github.com/settings/personal-access-tokens/new). Choose a name and expiration, set **Resource owner** to the repository owner, and select only the sync repository under **Repository access → Only select repositories**.
3. Under **Repository permissions**, grant **Contents: Read and write**. Keep the automatically included **Metadata: Read-only** permission. No Actions, Workflows, or Administration permission is needed. Generate and copy the token.
4. In **Settings → Sync → GitHub**, fill in the following fields. Disconnect any other sync provider first.

| Field | Example / meaning |
| --- | --- |
| Repository owner | `YOUR_USERNAME` |
| Private repository name | `papr-sync`, without a URL or `.git` suffix |
| Branch | Leave blank for the default branch, or enter an existing branch such as `main` |
| Fine-grained token | The generated token, not your GitHub password |

Select **Preview connection**, inspect the counts and exclusions, then confirm and select **Sync now**. Preview only reads the repository; a successful sync also confirms write access.

Tokens are credentials: keep them out of source control and logs. Windows stores them in Credential Manager; Android uses Keystore-protected credential storage. Separate tokens per device allow independent replacement or revocation. See [GitHub's token documentation](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-fine-grained-personal-access-token).

### Add another device

Let the device that already has reading data finish syncing, then connect the other device to the same owner, repository, and branch.

- A fresh installation restores the cloud catalog. Missing local entries are not interpreted as requests to delete cloud data.
- If both devices already have data, initial connection merges them. Existing cloud settings win for matching subscriptions; matching articles retain a read, starred, or Read Later flag if either side has it set during the initial merge.
- Later explicit changes, including clearing these flags or unsubscribing, propagate through sync. Each sync reads cloud changes before merging pending local operations.
- Device registration can create a commit even when the joining device has no local articles.

### Scheduling and recovery

Changes to the automatic-sync switch and intervals take effect only after selecting **Save**. Closing the settings page discards unsaved edits; a failed save preserves the draft for retry. These preferences apply only to the current device.

| Setting | Default | Choices |
| --- | --- | --- |
| Upload delay after local edits | 30 seconds | 10, 30, 60, 120 seconds |
| Foreground cloud check | 10 minutes | 5, 10, 15, 30, 60 minutes |
| Android background check | 60 minutes | 15, 30, 60, 120, 360 minutes |

Continuous edits become eligible for sync after the greater of 60 seconds or twice the upload delay. Publications are at least 60 seconds apart. Network failures and GitHub rate limits can extend the wait, including for manual sync. Turning off automatic sync keeps pending changes and allows **Sync now**.

Windows must remain running. Android foreground sync runs while the app is visible; background work is scheduled by the system and can be delayed by power-saving and network conditions. Feed refresh has its own settings. Unchanged data uses a lightweight cloud check; a full maintenance pass is due approximately every 24 hours to apply retention rules, or on the next manual sync in manual-only mode.

To replace an expired token, enter a new token with the same repository permissions and select **Update token**, then sync. Pending changes survive connection failures. For access errors, check the owner, repository selection, token permissions, and branch protection. Both devices need network access to GitHub.

## Development

The following commands assume a Windows development machine. Application support is scoped to the targets listed above.

### Repository layout

| Path | Responsibility |
| --- | --- |
| `src/` | React / TypeScript desktop interface |
| `src-tauri/` | Tauri desktop host and platform integration |
| `crates/papr-core/` | Shared Rust database, feed processing, and sync logic |
| `crates/papr-flutter-bridge/` | Rust API exposed to Flutter |
| `mobile/` | Flutter Android application and native integrations |
| `rust_frb_codegen.yaml` | Rust-to-Dart binding generation configuration |

### Windows desktop

Prerequisites:

- Node.js 22.12 or newer in the 22.x series, matching the Node 22 CI configuration.
- pnpm **11.5.0**, matching `package.json`.
- Rust stable via rustup, with the Windows MSVC toolchain.
- Visual Studio C++ Build Tools, Windows SDK, and WebView2 runtime.

From the repository root:

```powershell
pnpm install --frozen-lockfile
pnpm tauri dev
```

If pnpm reports a blocked `esbuild` build script, run `pnpm approve-builds` and approve that dependency. `pnpm tauri dev` starts both Vite and the desktop window; `pnpm dev` starts only the frontend server.

Build an installer:

```powershell
pnpm tauri build
```

The default Cargo workspace output is `target/release/bundle/`, with Windows installers under `nsis/` and `msi/`.

### Android

Install Flutter and its bundled Dart SDK, JDK 17, Android SDK tools, and NDK **30.0.14904198**. CI pins Flutter **3.44.5**. Run `flutter doctor -v` and `flutter doctor --android-licenses` to check the Android toolchain.

Install the Rust targets used by the Android build:

```powershell
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

Configure their linkers in your user Cargo configuration (`~/.cargo/config.toml`), replacing `<ndk>` with the installed NDK directory:

```toml
[target.aarch64-linux-android]
linker = "<ndk>/toolchains/llvm/prebuilt/windows-x86_64/bin/aarch64-linux-android21-clang.cmd"
[target.armv7-linux-androideabi]
linker = "<ndk>/toolchains/llvm/prebuilt/windows-x86_64/bin/armv7a-linux-androideabi21-clang.cmd"
[target.i686-linux-android]
linker = "<ndk>/toolchains/llvm/prebuilt/windows-x86_64/bin/i686-linux-android21-clang.cmd"
[target.x86_64-linux-android]
linker = "<ndk>/toolchains/llvm/prebuilt/windows-x86_64/bin/x86_64-linux-android21-clang.cmd"
```

From the repository root:

```powershell
cd mobile
flutter pub get --enforce-lockfile
flutter build apk --debug
adb install -r build/app/outputs/flutter-apk/app-debug.apk
```

Enable USB debugging on the test device before installing.

Release signing inputs are defined in [the Android Gradle configuration](mobile/android/app/build.gradle.kts).

### Regenerate bridge bindings

Generated Rust/Dart bindings are committed. Regenerate them when the bridge API changes, after fetching Flutter dependencies. Install `flutter_rust_bridge_codegen` matching the resolved bridge version, currently **2.12.0**, then run from the repository root:

```powershell
flutter_rust_bridge_codegen generate --config-file rust_frb_codegen.yaml
```

### Checks

Run frontend and Rust checks from the repository root:

```powershell
pnpm build
pnpm test
cargo check --workspace --all-targets --locked
cargo test --workspace --locked -- --test-threads=1
```

Run Flutter checks from `mobile/`:

```powershell
flutter analyze
flutter test
```

The CI workflow also checks version consistency and release tooling with Python 3.12. Run these commands from the repository root:

```powershell
python scripts/app_version.py check
python -m unittest discover -s scripts/tests -p 'test_*.py'
```

## License and attribution

[MIT](LICENSE). Based on [l0ng-ai/papr](https://github.com/l0ng-ai/papr); the original copyright notice is retained.
