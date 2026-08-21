<div align="center">

<img src="docs/logo.svg" alt="Papr" width="96" height="96" />

# Papr

A fast, native RSS reader for the desktop.

<img src="docs/screenshot.webp" alt="Papr" width="820" />

</div>

## Features

- **Feeds & folders** — subscribe, organize, and import/export OPML.
- **Smart views** — All, Unread, Starred, and Read Later, with live counts.
- **Tags & rules** — color-coded tags and rules that tag new articles automatically.
- **Full-text** — fetch and clean the complete article when a feed ships only a summary.
- **AI** — summaries with inline follow-up Q&A, ask-the-article Q&A, and digests. Bring your own API key.
- **Audio** — a built-in player that follows you from article to article.
- **FreshRSS sync** — keep read state in step with a FreshRSS server.
- **Local-first** — everything in a local SQLite database. No account, no cloud.
- **Localized** — English, Japanese, and Simplified Chinese.

## Installation

### macOS

Install with [Homebrew](https://brew.sh):

```sh
brew install --cask l0ng-ai/papr/papr
```

### All platforms

Download the installer for your platform from the [latest release](https://github.com/l0ng-ai/papr/releases/latest).

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v20+)
- [pnpm](https://pnpm.io/) (v9+)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable via rustup)
- **Windows only**: WebView2 runtime (usually pre-installed) and MSVC build tools (installed via rustup).

### Install dependencies

```sh
pnpm install
```

If pnpm reports `Ignored build scripts: esbuild`, allow the build script:

```sh
pnpm approve-builds
```

### Run in development mode

```sh
pnpm tauri dev
```

This starts the Vite dev server and the Tauri desktop window with hot reload.

### Build the desktop app

```sh
pnpm tauri build
```

After the build completes, the installable bundles are located at:

```text
src-tauri/target/release/bundle/
```

On Windows you will typically find:

- `src-tauri/target/release/bundle/msi/Papr_*.msi`
- `src-tauri/target/release/bundle/nsis/Papr_*-setup.exe`

### Android (Flutter)

The Android client lives in `mobile/` and shares `papr-core` via Flutter Rust Bridge.

#### Prerequisites (one-time)

1. Install the Rust Android targets:

   ```sh
   rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
   ```

2. Install the Android NDK (`30.0.14904198` is known to work) and point each Rust
   target at its clang linker in `~/.cargo/config.toml`:

   ```toml
   [target.aarch64-linux-android]
   linker = "<ndk>/toolchains/llvm/prebuilt/<host>/bin/aarch64-linux-android21-clang.cmd"
   [target.armv7-linux-androideabi]
   linker = "<ndk>/toolchains/llvm/prebuilt/<host>/bin/armv7a-linux-androideabi21-clang.cmd"
   [target.i686-linux-android]
   linker = "<ndk>/toolchains/llvm/prebuilt/<host>/bin/i686-linux-android21-clang.cmd"
   [target.x86_64-linux-android]
   linker = "<ndk>/toolchains/llvm/prebuilt/<host>/bin/x86_64-linux-android21-clang.cmd"
   ```

3. If Gradle plugin/dependency downloads are blocked (TLS handshake
   interruptions to `plugins.gradle.org` etc.), add a Gradle init script that
   injects a mirror (e.g. Aliyun) into `pluginManagement` and
   `dependencyResolutionManagement` for every build.

#### Regenerate the Rust↔Dart bindings (only when the bridge API changes)

```sh
flutter_rust_bridge_codegen generate --config-file rust_frb_codegen.yaml
```

#### Build the APK

```sh
cd mobile
flutter build apk --debug --no-pub
```

`preBuild` automatically cross-compiles `papr-flutter-bridge` (and `papr-core`)
for each Android ABI and bundles `libpapr_flutter_bridge.so` into the APK, so a
plain `flutter build apk` picks up Rust changes with no extra step.

- `--debug` produces a debug-signed APK for real devices. Use `--release` for a
  shippable build.
- `--no-pub` skips `flutter pub get` (deps are already locked).

The APK is written to:

```text
mobile/build/app/outputs/flutter-apk/app-debug.apk
```

#### Install on a device

Enable USB debugging on the phone, plug it in, then:

```sh
adb install -r mobile/build/app/outputs/flutter-apk/app-debug.apk
```

### Other useful scripts

```sh
pnpm dev        # Start the frontend dev server only (no Tauri window)
pnpm build      # Build the frontend for production
pnpm preview    # Preview the production frontend
pnpm test       # Run unit tests
```
