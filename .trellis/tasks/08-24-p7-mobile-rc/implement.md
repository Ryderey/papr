# Implement: P7 移动范围验收与可发布 RC

1. [x] 冻结移动范围矩阵并审计明确排除项无回流。
2. [ ] 完成 l10n、动态字体、窄屏/平板、TalkBack、对比度与深色模式修复。
3. [x] 运行 Rust/Bridge/Flutter/桌面全量自动化门禁。
4. [ ] 执行冷启动、大库、长文、长音频、后台、同步与进程恢复压力矩阵。
5. [ ] 验证 Alpha→RC 连续迁移、清除数据和凭据删除。
6. [ ] 生成签名 Release APK/AAB，记录版本、校验值和安装/升级结果。
7. [ ] 完成隐私说明和最终设备端到端验收。
8. [ ] 归档 P7 与父任务并记录未进入 RC 的后续候选。

## Progress — 2026-09-27

- P0–P6 已归档。`docs/mobile-rc-scope-matrix.md` 已按实际代码路径列出实现、Android 替代和排除项；最终设备验收仍待记录。
- 三种 ARB 语言的键集合一致。与英文相同的六项均为应用名、URL/OPML、Anthropic 或认证术语；文案校对和无障碍设备检查尚未完成。
- `docs/mobile-privacy.md` 记录当前数据流、Android 备份边界和删除方式；发布前仍需核对最终 APK 中的 SDK 和设备备份行为。
- Release Gradle 配置改为读取未入库的 `key.properties`；正式上传密钥、Alpha 签名基线与升级测试尚不可用。此前 debug-signed AAB 不计作 RC 产物。
- 当前只有 API 37 手机模拟器；Android 10、中间版本、平板和实体机矩阵仍未完成。
- 设置页新增 320×640、2 倍日文字体的首屏与滚动末端回归；Flutter 33 项测试、`flutter analyze --no-pub` 和 Debug APK 构建通过。无 `key.properties` 时，Release 构建现以明确签名错误停止；正式签名的正向构建尚未验证。
- Core 增加 v16 Alpha→当前 v17 的保留文章/摘要/设置迁移测试。全量门禁：Core 102、Bridge 3、桌面 Rust 252、Flutter 33、桌面前端 73 项测试通过；FRB check、Flutter analyze、Debug APK、`pnpm build` 通过。桌面 Rust 与 `pnpm` 在沙箱权限不足后以授权环境复测通过。真实安装升级仍待同签名 Alpha 包。
- Debug APK 的 `minSdkVersion=24`、`targetSdkVersion=36`；16 KiB ZIP 对齐与全部 64 位 `.so` ELF `LOAD` 对齐检查通过。既有安装版在 API 37、16 KiB 页模拟器上冷启动成功（`am start -W`，`Status: ok`，4170 ms）；正式签名 AAB 仍待同项复查。

## Rollback

任何阻断项都回到所属阶段修复；不得用关闭迁移、清库、跳过安全检查或扩大权限作为发布方案。
