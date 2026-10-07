<div align="center">

<img src="docs/logo.svg" alt="Papr" width="96" height="96" />

# Papr

面向 Windows 和 Android 的本地优先 RSS 阅读器。

[English](README.md) · **简体中文**

<img src="docs/screenshot.webp" alt="Papr 阅读界面" width="820" />

</div>

本仓库基于 [l0ng-ai/papr](https://github.com/l0ng-ai/papr) 的 0.9.0 版本继续开发，增加 Android 客户端、GitHub 仓库同步、多 AI 配置档、摘要模板和摘要追问。本仓库维护的应用平台为 **Windows x64 和 Android**。

## 功能

- 订阅源与文件夹管理、OPML 导入导出，以及全部、未读、星标和稍后读视图。
- 文章全文提取、标签、自动打标签规则、高亮和音频播放。
- AI 摘要、摘要追问和 LLM 翻译，使用自行配置的 API 凭据；支持 Anthropic Messages 和 OpenAI Chat Completions 兼容接口。
- 多种摘要模板、连接测试，以及桌面端可调整宽度的 AI 面板。
- 可选的 GitHub 私有仓库同步，以及 FreshRSS、Miniflux 服务集成。
- 本地 SQLite 存储，支持英文、日文和简体中文界面。

Windows 端还提供全局 Ask/RAG、Digest、Newsletter/IMAP、Send to Kindle，以及托盘和快捷键等桌面交互。这些功能尚未移植到 Android；Android 提供系统分享、定时后台任务和后台音频播放。

<p align="center">
  <img src="docs/AI摘要及追问.webp" alt="AI 摘要与追问" width="960" />
</p>

## 安装

从 [最新发行版](https://github.com/Ryderey/papr/releases/latest) 下载安装包；测试版本见 [全部发行版](https://github.com/Ryderey/papr/releases)。

| 平台 | 安装包 |
| --- | --- |
| Windows x64 | `*-windows-x64-setup.exe` |
| Android ARM64 | `*-android-arm64-v8a.apk`，适用于多数现代 ARM 手机 |
| Android ARM32 | `*-android-armeabi-v7a.apk`，适用于 32 位 ARM 设备 |

发行说明包含版本、SHA-256 校验值及构建来源。更新需手动下载安装；Android 覆盖升级要求签名证书和版本号兼容。

## 同步

云端同步为可选功能。GitHub 同步通过同一个私有仓库连接 Windows 和 Android，无需另行部署服务器或 GitHub Actions。FreshRSS 和 Miniflux 使用各自的服务端集成；以下范围专门针对 **GitHub 同步**。同一时间只能连接一种同步服务。

### GitHub 具体同步哪些内容

| 数据 | 是否同步 | 范围 |
| --- | --- | --- |
| 订阅源 | 是 | 订阅地址、来源类型、显示名称、所属文件夹，以及订阅／取消订阅状态 |
| 文件夹 | 是 | 名称和文件夹顺序 |
| 文章目录 | 是 | 文章标识、标题、链接、发布时间和首次发现时间 |
| 阅读状态 | 是 | 已读／未读、星标和稍后读 |
| 文章正文与缓存内容 | 否 | 各端自行获取 |
| 标签、自动打标签规则和高亮 | 否 | 保留在创建它们的设备上 |
| AI 配置档与凭据 | 否 | API Key、接口地址、模型、自定义请求头及摘要偏好均保留在本地 |
| AI 生成内容 | 否 | 摘要、翻译结果和问答历史不在同步数据中 |
| 外观与其他应用设置 | 否 | 主题、字体、语言、快捷键、通知和刷新偏好均保留在本地 |
| 同步凭据与时间设置 | 否 | 每台设备分别配置 Token 和自动同步周期 |

支持同步的来源类型为 RSS、YouTube、播客、Mastodon、Bluesky 和 Reddit，订阅地址需为有效的 HTTP(S) 地址。Newsletter/IMAP，以及不符合支持的来源或元数据格式的条目会被排除；连接预览会显示排除数量。

普通文章目录保留 **90 天**；只要仍有星标或稍后读标记，对应条目就会长期保留。

### 连接仓库

1. 在自己的 GitHub 账号下创建一个专用的 **私有仓库**，例如 `papr-sync`。添加 README 完成初始化，使默认分支存在。
2. 打开 [GitHub 精细权限 Token 创建页面](https://github.com/settings/personal-access-tokens/new)，填写名称和有效期；将 **Resource owner** 设为仓库所属账号，在 **Repository access → Only select repositories** 中只选择同步仓库。
3. 在 **Repository permissions** 中授予 **Contents: Read and write**，保留自动包含的 **Metadata: Read-only**。无需开启 Actions、Workflows 或 Administration 权限。生成并复制 Token。
4. 进入 **设置 → 同步 → GitHub**，填写下表字段。如果已连接其他同步服务，先断开该服务。

| 字段 | 示例／含义 |
| --- | --- |
| 仓库所有者 | `YOUR_USERNAME`，替换为自己的 GitHub 用户名 |
| 私有仓库名称 | `papr-sync`，不带 URL 或 `.git` 后缀 |
| 分支 | 留空使用默认分支，也可填写已存在的分支，例如 `main` |
| 精细权限 Token | 刚生成的 Token，不是 GitHub 登录密码 |

点击 **预览连接**，核对数量与排除提示，再确认连接并点击 **立即同步**。预览仅读取仓库；同步成功才能同时确认写入权限正常。

Token 属于凭据，不应写入代码仓库或日志。Windows 使用系统凭据管理器保存 Token，Android 使用 Keystore 保护的凭据存储。每台设备使用独立 Token，便于分别更换或撤销。参见 [GitHub 官方 Token 文档](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens#creating-a-fine-grained-personal-access-token)。

### 添加另一台设备

先让已有阅读数据的设备完成同步，再将另一台设备连接到相同的仓库所有者、仓库名称和分支。

- 全新安装会恢复云端文章目录；本地没有条目，不会被当成删除云端数据的请求。
- 两端已有数据时，首次连接会合并双方数据。同一订阅采用已有云端设置；同一文章在首次合并时，只要任一端为已读、星标或稍后读，就保留对应标记。
- 后续明确修改，包括取消这些标记或取消订阅，会通过同步传播。每次同步先读取云端变化，再合并本地待上传操作。
- 即使新加入的设备没有本地文章，设备登记也可能产生 GitHub 提交。

### 同步周期与故障恢复

修改自动同步开关或时间选项后，点击 **保存** 才会生效。未保存就关闭设置页会丢弃修改；保存失败会保留编辑内容，便于重试。这些偏好仅作用于当前设备。

| 设置 | 默认值 | 可选值 |
| --- | --- | --- |
| 本地修改后的上传等待 | 30 秒 | 10、30、60、120 秒 |
| 前台检查云端更新 | 10 分钟 | 5、10、15、30、60 分钟 |
| Android 后台检查 | 60 分钟 | 15、30、60、120、360 分钟 |

持续操作时，待上传修改达到“60 秒或上传等待的两倍，取较大值”后，也会满足同步触发条件。两次发布至少间隔 60 秒；网络故障或 GitHub 限流可能延长等待，手动同步也遵守这些限制。关闭自动同步后，待上传修改仍会保留，可通过 **立即同步** 发起同步。

Windows 端需保持运行。Android 前台同步在应用可见时执行；后台由系统调度，可能受省电和网络条件影响而延迟。订阅源刷新使用独立设置。数据无变化时进行轻量云端检查；约每 24 小时需要一次完整维护，以执行保留规则。仅手动模式下，维护会在下一次手动同步时执行。

Token 到期后，生成具有相同仓库权限的新 Token，在设置中填写并点击 **更新 Token**，再进行同步。连接失败期间待上传操作会保留。访问失败时检查账号、仓库选择、Token 权限和分支保护规则；两台设备都需要能够访问 GitHub。

## 开发

以下命令以 Windows 开发环境为例。应用维护范围以上述平台为准。

### 目录结构

| 路径 | 职责 |
| --- | --- |
| `src/` | React / TypeScript 桌面界面 |
| `src-tauri/` | Tauri 桌面宿主与平台集成 |
| `crates/papr-core/` | Rust 共享数据库、订阅抓取与同步逻辑 |
| `crates/papr-flutter-bridge/` | 暴露给 Flutter 的 Rust API |
| `mobile/` | Flutter Android 应用与原生集成 |
| `rust_frb_codegen.yaml` | Rust 到 Dart 的绑定生成配置 |

### Windows 桌面端

环境要求：

- Node.js 22.x，至少 22.12，与 CI 使用的 Node 22 系列一致。
- pnpm **11.5.0**，与 `package.json` 一致。
- 通过 rustup 安装的 Rust stable 和 Windows MSVC 工具链。
- Visual Studio C++ Build Tools、Windows SDK 和 WebView2 运行时。

在仓库根目录执行：

```powershell
pnpm install --frozen-lockfile
pnpm tauri dev
```

如果 pnpm 提示阻止了 `esbuild` 构建脚本，运行 `pnpm approve-builds` 并批准该依赖。`pnpm tauri dev` 会同时启动 Vite 和桌面窗口；`pnpm dev` 仅启动前端开发服务器。

构建安装包：

```powershell
pnpm tauri build
```

默认 Cargo 工作区输出目录为 `target/release/bundle/`，Windows 安装包位于其中的 `nsis/` 和 `msi/`。

### Android

安装 Flutter 及其自带的 Dart SDK、JDK 17、Android SDK 工具和 NDK **30.0.14904198**。CI 固定使用 Flutter **3.44.5**。通过 `flutter doctor -v` 和 `flutter doctor --android-licenses` 检查 Android 工具链。

安装 Android 构建使用的 Rust 目标：

```powershell
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

在用户级 Cargo 配置（`~/.cargo/config.toml`）中指定链接器，将 `<ndk>` 替换为已安装的 NDK 目录：

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

从仓库根目录执行：

```powershell
cd mobile
flutter pub get --enforce-lockfile
flutter build apk --debug
adb install -r build/app/outputs/flutter-apk/app-debug.apk
```

安装前需在测试设备启用 USB 调试。

发布构建的签名输入见 [Android Gradle 配置](mobile/android/app/build.gradle.kts)。

### 重新生成桥接绑定

生成的 Rust/Dart 绑定已纳入版本控制。桥接 API 变更后，在获取 Flutter 依赖的基础上重新生成。安装与解析后的桥接版本一致的 `flutter_rust_bridge_codegen`，当前为 **2.12.0**，然后在仓库根目录执行：

```powershell
flutter_rust_bridge_codegen generate --config-file rust_frb_codegen.yaml
```

### 检查与测试

在仓库根目录执行前端和 Rust 检查：

```powershell
pnpm build
pnpm test
cargo check --workspace --all-targets --locked
cargo test --workspace --locked -- --test-threads=1
```

在 `mobile/` 中执行 Flutter 检查：

```powershell
flutter analyze
flutter test
```

CI 还使用 Python 3.12 检查版本一致性和发行工具，以下命令在仓库根目录运行：

```powershell
python scripts/app_version.py check
python -m unittest discover -s scripts/tests -p 'test_*.py'
```

## 许可与来源

采用 [MIT 许可](LICENSE)。基于 [l0ng-ai/papr](https://github.com/l0ng-ai/papr)，保留原始版权声明。
