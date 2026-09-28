# P7 移动范围验收与内部 RC

## Goal

按精简后的移动端范围，交付供个人使用的内部 Release Candidate，并明确记录尚未完成的扩展验收。2026-09-28 用户决定将 P7 剩余验收延后至独立任务 `09-28-mobile-deferred-acceptance`。

## Requirements

1. 建立“已实现 / Android 替代 / 明确排除”范围矩阵，禁止用桌面功能数量作为完成度。
2. 保持三种语言键集合一致，完成已有的日文窄屏/字体 Widget 回归，并记录 API 37 设置页语言与浅/深主题烟测。
3. 以自动化测试验证 Alpha v16 schema 连续迁移不清库；在 API 37 测试模拟器验证内部测试包安装/升级、冷启动、临时 AI 凭据删除和清除数据语义。
4. 构建内部测试 APK，记录版本、校验值和签名证书指纹；内部构建可使用 Android Debug 签名。
5. 提供隐私说明，覆盖本地数据、Feed/AI/翻译/同步请求、凭据和删除方式。
6. 将完整业务链路、多版本/平板、全面无障碍、压力、安全深度验证明确转入后续任务，不宣称这些项目已通过。

## Acceptance Criteria

- [x] P0–P6 全部精简范围任务已归档，范围矩阵和延期项目均有记录。
- [x] Rust、FRB、Flutter、桌面回归和内部 Release 构建通过；结果见 `implement.md`。
- [x] 内部测试 APK 可安装、从同证书测试包升级并启动；Core 的 Alpha schema 迁移测试通过。
- [x] API 37 模拟器完成临时 AI 凭据删除和清除数据烟测，隐私说明记录验证边界。
- [x] 未覆盖的设备、业务主链路和深度安全/无障碍验收已转入 `09-28-mobile-deferred-acceptance`，不作为当前自用 RC 的完成声明。

## Dependencies / Out of Scope

依赖 P3–P6 归档。多版本/平板、全面无障碍、压力、完整业务链路和深度安全验收延后至 `09-28-mobile-deferred-acceptance`。实体机验收和正式 Alpha APK 原位升级按自用范围决定跳过；正式签名、AAB、Google Play 上架和商店运营材料也不属于本任务，未来有分发需求时另行规划。内部签名材料不提交仓库。
