# Implement: P7 移动范围验收与内部 RC

1. [x] 冻结移动范围矩阵并审计明确排除项无回流。
2. [x] 核对三语键集合，完成已有日文窄屏/字体 Widget 回归与 API 37 浅/深主题烟测。
3. [x] 运行 Rust/Bridge/Flutter/桌面全量自动化门禁。
4. [x] 验证 API 37 冷启动、进程重启与设置保留；压力矩阵转入后续任务。
5. [x] 验证 Core Alpha schema 连续迁移、临时 AI 凭据删除和应用内清除数据；跳过正式 Alpha APK 原位升级。
6. [x] 生成内部测试 Release APK，记录版本、校验值、证书指纹和安装/升级结果。
7. [x] 完成隐私说明和 API 37 自用 RC 烟测；完整业务主链路转入后续任务。
8. [x] 将未完成的兼容、无障碍、压力、深度安全与业务链路验收记录在 `09-28-mobile-deferred-acceptance`。

## Progress — 2026-09-27

- P0–P6 已归档。`docs/mobile-rc-scope-matrix.md` 已按实际代码路径列出实现、Android 替代和排除项；最终设备验收仍待记录。
- 三种 ARB 语言的键集合一致。与英文相同的六项均为应用名、URL/OPML、Anthropic 或认证术语；文案校对和无障碍设备检查尚未完成。
- `docs/mobile-privacy.md` 记录当前数据流、Android 备份边界和删除方式；内部交付前仍需核对最终 APK 中的 SDK 和设备备份行为。
- 2026-09-27 范围调整：正式上传签名、AAB 与 Google Play 准备移出 P7；内部 Release APK 使用 Debug 签名。正式 Alpha 包的证书及数据升级仍待核对。
- 当前只有 API 37 手机模拟器；Android 10、中间版本、平板和实体机矩阵仍未完成。
- 设置页新增 320×640、2 倍日文字体的首屏与滚动末端回归；Flutter 33 项测试、`flutter analyze --no-pub` 和 Debug APK 构建通过。此前 Release 构建因缺少 `key.properties` 停止，该门禁已随范围调整移除。
- Core 增加 v16 Alpha→当前 v17 的保留文章/摘要/设置迁移测试。全量门禁：Core 102、Bridge 3、桌面 Rust 252、Flutter 33、桌面前端 73 项测试通过；FRB check、Flutter analyze、Debug APK、`pnpm build` 通过。桌面 Rust 与 `pnpm` 在沙箱权限不足后以授权环境复测通过。正式 Alpha 数据升级仍待同签名 Alpha 包。
- Debug APK 的 `minSdkVersion=24`、`targetSdkVersion=36`；16 KiB ZIP 对齐与全部 64 位 `.so` ELF `LOAD` 对齐检查通过。既有安装版在 API 37、16 KiB 页模拟器上冷启动成功（`am start -W`，`Status: ok`，4170 ms）。
- 内部 RC `flutter build apk --release --no-pub --build-number=2` 通过；APK SHA-256、Debug 证书指纹、16 KiB ZIP/ELF 对齐详见 `docs/mobile-rc-release.md`。证书与模拟器已装版本 1 匹配，`adb install -r` 升级到版本 2 成功，冷启动 `Status: ok`（2339 ms）。截至当日，正式 Alpha 数据保留、凭据和完整设备主链路仍未验证。

## Progress — 2026-09-28

- 用户确认当前仅自用，跳过实体机验收和正式 Alpha APK 原位升级；保留 Core v16→当前的自动化数据迁移门禁。此前记录的正式 Alpha 包/实体机待验不再阻断 P7。
- 内部 APK 构建、版本 1→2 同证书模拟器安装和启动证据已满足步骤 6。AI 配置删除按钮补充本地化辅助功能标签；320×640、2 倍日文字体 Widget 测试通过。
- Flutter 34 项测试、`flutter analyze --no-pub`、Dart 格式检查和 Core v16 Alpha 数据保留迁移测试通过。`docs/mobile-privacy.md` 补记内部 APK 权限与凭据备份排除规则的静态核对；设备恢复行为仍未验证。
- 本机 SDK 仅安装 Android 37.1 系统镜像；Android 10/中间版本模拟器矩阵暂缺系统镜像。现有 Widget 测试覆盖 320×640 窄屏与 800×1000 平板导航布局，不能代替 Android 版本和完整端到端验收。
- 复用 `Medium_Phone`（API 37、1080×2400、420 dpi）验证：已装版本 1 与内部 RC 的 Debug 证书 SHA-256 相同，`adb install -r` 升级到版本 2 成功；冷启动 `Status: ok`（1619 ms）。设置页浅/深主题与日文切换可见且无溢出；强制停止后冷启动 `Status: ok`（1028 ms），日文和深色设置仍保留。模拟器无订阅数据，完整业务主链路仍待验。
- 在同一测试模拟器上用占位值新建 AI 配置，Android 私有 `papr_ai_credentials.xml` 出现 1 个凭据引用；经应用内确认删除后，配置列表为空且该文件凭据条目数为 0。再用新的占位配置和日文设置验证“清除所有数据”：确认后进程结束，应用私有偏好目录消失；重新冷启动 `Status: ok`（874 ms），语言恢复英语，AI 配置为空，凭据偏好文件未重新出现。未直接核查数据库文件、Keystore 别名和同步凭据删除。
- 用户将 P7 余项延后；完整列表与恢复条件见独立 planning 任务 `09-28-mobile-deferred-acceptance`。当前只将内部 APK 与已记录的自动化/API 37 烟测视为自用 RC 证据，不将延期项写为通过。

## Rollback

任何阻断项都回到所属阶段修复；不得用关闭迁移、清库、跳过安全检查或扩大权限作为发布方案。
