<div align="center">

<img src="docs/logo.svg" alt="Papr" width="96" height="96" />

# Papr

A local-first RSS reader with desktop and Android clients.

<img src="docs/screenshot.webp" alt="Papr" width="820" />

</div>

## Features

- **Feeds & folders** — subscribe, organize, and import/export OPML.
- **Smart views** — All, Unread, Starred, and Read Later, with live counts.
- **Tags & rules** — color-coded tags and rules that tag new articles automatically.
- **Full-text** — fetch and clean the complete article when a feed ships only a summary.
- **AI** — summaries, summary follow-up Q&A, and translation. Desktop also provides Ask/RAG and digests. Bring your own API key.
- **Audio** — a built-in player that follows you from article to article.
- **FreshRSS sync** — keep read state in step with a FreshRSS server.
- **GitHub sync** — synchronize Windows and Android through your own private repository; see [setup instructions](#github-私有仓库同步).
- **Local-first** — reading data lives in local SQLite; cloud synchronization is optional.
- **Localized** — English, Japanese, and Simplified Chinese.

### 平台范围

当前分支已整合 Tauri 桌面端与 Flutter Android 端，自用验证以 Windows 和 Android 为主。两端共用 `papr-core`，界面与部分功能按平台实现。

| 范围 | 当前能力 |
|---|---|
| 两端共有 | 订阅与文件夹、OPML、文章阅读与全文提取、已读／星标／稍后读、标签／规则／高亮、AI 摘要与摘要追问、LLM 翻译、音频播放、FreshRSS／Miniflux 同步 |
| Windows 与 Android | GitHub 私有仓库同步；当前桌面 GitHub 凭据存储仅实现 Windows，不适用于 macOS／Linux |
| 桌面功能 | 全局 Ask/RAG、Digest、Newsletter/IMAP、Send to Kindle，以及托盘、快捷键、拖动等桌面交互 |
| Android | 触控界面、系统分享、WorkManager 后台任务、Media3 后台音频；上述桌面专属功能未移植 |

桌面仍保留 macOS／Linux 构建配置。Android 完整范围及后续验收见 [移动端范围矩阵](docs/mobile-rc-scope-matrix.md)；GitHub 新增能力以本文同步章节为准。Android 后台同步、休眠及重启场景的实际验收记录在 [Android 后续验收任务](.trellis/tasks/09-28-mobile-deferred-acceptance/prd.md)。

## 基于原仓库的修改

本仓库基于原始项目 [l0ng-ai/papr](https://github.com/l0ng-ai/papr) 的 `0.9.0` 版本继续改造。当前分支相对原仓库的主要变化如下。

### 已实现的功能改造

- **多模型 AI 配置**：将原先固定的 Anthropic / OpenAI 配置改造成协议化配置档，支持 Anthropic Messages 与 OpenAI Chat Completions 兼容接口；可配置 API Key、模型、Base URL、认证方式和自定义请求头，并保留旧版 `ai_provider`、`ai_api_key`、`ai_model`、`ai_base_url` 设置的兼容回退。
- **AI 连接测试**：在设置页新增 AI 配置保存与连接测试能力，测试失败时会隐藏 API Key，并对 TLS / 证书类错误给出更明确的提示。
- **摘要模板**：新增多种文章摘要模板，包括 Classic TL;DR、5W1H News、Decision helper、Three-layer funnel、Argument deconstruction、Ultra-minimal；默认模板可在设置中保存，也可以在阅读器的 AI 摘要面板里临时切换。
- **摘要追问**：在 AI 摘要抽屉中加入追问输入框，用户可以基于已经生成的摘要继续提问；追问回答保持流式输出，并使用当前摘要和临时问答历史作为上下文。

<p align="center">
  <img src="docs/AI摘要及追问.webp" alt="Papr 的 AI 摘要与追问界面" width="960" />
</p>

- **AI 抽屉布局优化**：默认宽度为 480px，可拖动调整并保存宽度偏好，配置范围为 320–640px；实际宽度受可用空间限制。窗口变窄时会依次收起文章列表和侧栏，为正文与 AI 面板保留空间。
- **LLM 翻译配置对齐**：LLM 翻译会复用新的 AI 配置解析逻辑，同时保留 Google、DeepL、Bing 等独立翻译引擎选项。
- **国际化补充**：为新增的 AI 配置、摘要模板、追问和连接测试文案补齐 English、Japanese、Simplified Chinese 三套语言资源。

### 工程与构建调整

- **TLS / 打包修复**：调整 `reqwest` TLS 特性，启用 `rustls-tls-webpki-roots`、`system-proxy`、`http2` 等能力；同时在 Tauri 打包配置中启用 `useLocalToolsDir`，降低 Windows 环境下载/证书问题对打包流程的影响。
- **Windows 开发体验**：`pnpm dev` 启动前会先尝试释放 Vite 默认端口 `1430` 上遗留的 Node / Vite 进程，避免热重载服务因端口占用启动失败。
- **pnpm 构建脚本允许列表**：新增 `pnpm-workspace.yaml`，允许 `esbuild` 的构建脚本，减少安装依赖后的手工确认。
- **忽略本地敏感与工具状态**：扩展 `.gitignore`，忽略 `.env*`、本地 SQLite 数据库、CodeGraph、Playwright MCP、历史记录、缓存和覆盖率目录，避免把本地凭据或工具产物提交进仓库。

### 文档与规划补充

- **多 LLM 配置规格**：新增 `docs/multi-llm-provider-adapter-spec.md`，记录多供应商 LLM 配置层的目标架构、兼容策略和后续演进方向。
- **协作规则文档**：新增 `AGENTS.md` 以及 `docs/agents/*`，记录本仓库的 Agent 协作规则、Issue 追踪方式、标签约定和领域说明。
- **TLS 排障记录**：新增 `tls.md`，记录 Windows / 代理 / 证书链相关的 Tauri 打包排障过程和建议操作。

## GitHub 私有仓库同步

Windows 与 Android Papr 可以连接同一个 GitHub 私有仓库，双向同步订阅、文件夹、文章标题和链接，以及已读、星标和稍后读状态。普通文章目录保留 90 天，星标或稍后读条目长期保留。正文由各端自行获取；主题、字体、AI 配置等保留在本地。个人使用无需部署服务器或 GitHub Actions。

### 1. 准备私有仓库

在自己的 GitHub 账号下创建一个专用 **Private** 仓库，例如 `papr-sync`，并勾选添加 README，使默认分支已有首次提交。已有初始化的私有仓库可直接使用。

### 2. 获取 GitHub 精细权限 Token

1. 登录拥有同步仓库的 GitHub 账号，打开 [创建精细权限 Token 页面](https://github.com/settings/personal-access-tokens/new)。也可以从头像菜单依次进入 **Settings → Developer settings → Personal access tokens → Fine-grained tokens → Generate new token**。
2. **Token name**：填写便于识别的名称，例如电脑端 `Papr-Windows`、手机端 `Papr-Android`。
3. **Expiration**：选择有效期，建议先用 90 天；到期后需生成新 Token 并在 Papr 中更新。这与普通文章目录的 90 天保留期是两个独立设置。
4. **Resource owner**：选择拥有同步仓库的账号。
5. **Repository access**：选择 **Only select repositories**，在 **Selected repositories** 中只勾选同步仓库，例如 `papr-sync`。
6. **Permissions → Repository permissions**：将 **Contents** 设置为 **Read and write**。保留自动包含的 **Metadata: Read-only**，无需额外开启 Actions、Workflows 或 Administration 权限。
7. 点击 **Generate token**，立即复制完整 Token，并粘贴到 Papr 的 Token 输入框。生成结果只显示一次；遗失时需重新生成。

建议每台设备使用独立 Token，便于单独更换或撤销；两端共用一个 Token 也可以。不要把 Token 写入仓库、文档、截图或聊天。Windows 使用系统凭据管理器保存 Token，Android 使用 Keystore 保护的凭据存储。

参考：[GitHub 官方 Token 创建说明](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-fine-grained-personal-access-token)、[GitHub Git 引用写入权限](https://docs.github.com/en/rest/git/refs#update-a-reference)。

### 3. 在 Papr 中填写并连接

进入 **设置 → 同步 → GitHub**。如果已连接其他同步服务，先断开该服务，再连接 GitHub。

以仓库 `https://github.com/YOUR_USERNAME/papr-sync.git` 为例：

| 字段 | 填写内容 |
|---|---|
| 仓库所有者 | `YOUR_USERNAME`，替换为自己的 GitHub 用户名 |
| 私有仓库名称 | `papr-sync`，只填名称，不带 URL 或 `.git` |
| 分支 | 留空使用默认分支；也可以填写实际分支名，例如 `main` |
| GitHub 精细权限 Token | 刚生成的完整 Token，不是 GitHub 登录密码 |

例如仓库为 `Ryderey/papr-sync` 时，所有者填写 `Ryderey`，仓库名称填写 `papr-sync`，分支填写 `main` 或留空。

先点击 **预览连接**，核对本地与云端订阅、文章数量及排除提示，再点击 **确认连接**，然后点击 **立即同步**，等待成功。预览只读取仓库；首次同步成功才能确认 Token 的写入权限正常。

### 4. 连接另一台设备与首次合并

先让已有资料的电脑端完成一次同步，再在手机端填写相同的所有者、仓库名称和分支，使用手机端 Token 预览、确认并立即同步。

- 全新安装且本地为空的手机会恢复云端资料。本地缺少条目不会被当作删除云端条目的操作。
- 如果另一端已有资料，首次连接会合并双方数据。同一订阅保留已有云端设置；同一文章的已读、星标和稍后读状态，首次合并时任一端为真就保留为真。
- 后续明确取消星标、稍后读、已读或订阅会正常传播。两端都可以修改；每次同步先读取云端，再合并本地待上传操作。
- 首次加入可能产生设备登记与进度提交，即使手机本地没有资料，也可能看到新的 GitHub 提交。

### 5. Token 到期或更换

按上述最小权限设置重新生成 Token，在已连接的 Papr 同步设置中填写新 Token，点击 **更新 Token**，然后立即同步。无需断开并重新连接；到期或网络失败期间，本地待上传操作会保留。

遇到仓库不可访问时，检查账号、仓库名称和 Token 的仓库选择；遇到写入失败时，检查 **Contents: Read and write** 及分支保护规则。连接和同步都需要设备能够访问 GitHub。

## Installation

### 本仓库版本

查看 [Ryderey/papr Releases](https://github.com/Ryderey/papr/releases)，选择与所需分支和功能对应的构建，并核对发行说明。源码合并或推送不等于已生成安装包；如果没有包含本分支改造的发行版，请按下面的开发步骤从源码构建。

Android 当前面向自用／内部测试，安装 APK 前请核对包名、版本和签名；操作步骤见 [Android RC 安装与升级说明](docs/mobile-rc-release.md)。

### 上游原版（macOS／桌面）

以下入口安装的是原始项目 `l0ng-ai/papr` 的发行版，不能作为本仓库改造功能的安装保证。macOS 原版可通过 [Homebrew](https://brew.sh) 安装：

```sh
brew install --cask l0ng-ai/papr/papr
```

其他桌面平台的上游安装包见 [上游最新发行版](https://github.com/l0ng-ai/papr/releases/latest)。

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) — Node 20.x requires 20.19.0 or newer; Node 22 and later require 22.12.0 or newer. Node 22 matches the desktop CI configuration. These bounds follow the locked Vite dependency.
- [pnpm](https://pnpm.io/) (v9+)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable via rustup)
- **Windows only**: WebView2 runtime (usually pre-installed), Visual Studio C++ Build Tools with the Windows SDK, and the Rust MSVC toolchain installed through rustup.
- **Linux only**: WebKitGTK 4.1 development libraries, AppIndicator, librsvg, and patchelf; see the package list in [the desktop release workflow](.github/workflows/release.yml).

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
target/release/bundle/
```

On Windows you will typically find:

- `target/release/bundle/msi/Papr_*.msi`
- `target/release/bundle/nsis/Papr_*-setup.exe`

The root Cargo workspace owns the default `target/` directory. A custom Cargo target directory or an explicit cross-compilation target changes these paths; use the path printed by the build in that case.

### Android (Flutter)

The Android client lives in `mobile/` and shares `papr-core` via Flutter Rust Bridge.

#### Prerequisites (one-time)

Prepare these tools before installing the Rust Android targets:

- Flutter SDK on `PATH`, including its bundled Dart SDK. The current lockfile requires Flutter >=3.38.4 and Dart >=3.11.0 <4.0.0; the selected Flutter installation must satisfy both requirements.
- JDK 17 and Android SDK tooling, including Platform Tools (`adb`) and the SDK platform/build tools required by the selected Flutter SDK. Android Studio can manage the SDK, NDK, and device/emulator setup.
- Rust stable and the Android NDK pinned by the Gradle configuration (`30.0.14904198`).

Run `flutter doctor -v`, resolve Android toolchain issues, and accept the SDK licenses with `flutter doctor --android-licenses`. Configure the SDK paths for the local machine; do not commit `mobile/android/local.properties`.

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

   The linker examples above are for Windows, where `<host>` is `windows-x86_64`. On Linux or macOS, use the corresponding NDK host directory and omit the `.cmd` suffix.

3. If Gradle plugin/dependency downloads are blocked (TLS handshake
   interruptions to `plugins.gradle.org` etc.), add a Gradle init script that
   injects a mirror (e.g. Aliyun) into `pluginManagement` and
   `dependencyResolutionManagement` for every build.

#### Regenerate the Rust↔Dart bindings (only when the bridge API changes)

Existing generated bindings are committed. When regeneration is needed, install `flutter_rust_bridge_codegen` matching the resolved FRB bridge version (currently 2.12.0) and run the following from the repository root, after fetching the Flutter dependencies:

```sh
flutter_rust_bridge_codegen generate --config-file rust_frb_codegen.yaml
```

#### Build the APK

```sh
cd mobile
flutter pub get
flutter build apk --debug
```

`preBuild` automatically cross-compiles `papr-flutter-bridge` (and `papr-core`)
for each Android ABI and bundles `libpapr_flutter_bridge.so` into the APK, so a
plain `flutter build apk` picks up Rust changes with no extra step.

- `--debug` produces a Debug-signed APK for development and device testing.
- `--release` produces an optimized APK, but the currently committed Release configuration also uses Debug signing. It is an internal/self-use build; production signing, AAB packaging, and store distribution require separate setup. See [the Android RC procedure](docs/mobile-rc-release.md).
- `--no-pub` is suitable for repeat builds only after `flutter pub get` has completed and the dependency configuration is unchanged. A committed lockfile does not download dependencies or create `.dart_tool/package_config.json`.

The APK is written to:

```text
mobile/build/app/outputs/flutter-apk/app-debug.apk
```

#### Install on a device

Enable USB debugging on the phone, plug it in, then:

Run this command from the repository root (use `build/app/outputs/flutter-apk/app-debug.apk` if the terminal is still in `mobile/`):

```sh
adb install -r mobile/build/app/outputs/flutter-apk/app-debug.apk
```

An in-place update requires a compatible signing certificate and version code. If installation reports a signature/version mismatch, preserve the existing app data and resolve the build configuration; do not uninstall the app merely to make the command succeed.

### Other useful scripts

```sh
pnpm dev        # Start the frontend dev server only (no Tauri window)
pnpm build      # Build the frontend for production
pnpm preview    # Preview the production frontend
pnpm test       # Run unit tests
```
