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
- **AI** — summaries, ask-the-article Q&A, and digests. Bring your own API key.
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

### Other useful scripts

```sh
pnpm dev        # Start the frontend dev server only (no Tauri window)
pnpm build      # Build the frontend for production
pnpm preview    # Preview the production frontend
pnpm test       # Run unit tests
```
