# Design: P7 移动范围验收与内部 RC

## Evidence Model

每项验收必须链接到自动化测试、构建产物或设备记录之一。范围矩阵以父任务明确排除项为基线，发现范围回流先修计划而非临时实现。

## Release Matrix

当前自用 RC 的设备证据限于 API 37 手机、浅/深主题与日文设置、窄屏/平板 Widget 布局、内部测试包升级、临时 AI 凭据删除和应用内清除数据。Core Alpha schema 迁移由自动化测试覆盖。多 Android 版本、平板模拟器、完整无障碍/压力/业务链路及更深的数据安全检查转入 `09-28-mobile-deferred-acceptance`；实体机与正式 Alpha APK 原位升级不在本轮范围。

## Artifact and Secrets

Debug APK 用于迭代验收；内部 RC 使用 Debug 签名的 Release APK。仓库只记录构建步骤、版本、校验值和证书指纹，不记录密钥。正式签名与商店分发留待后续任务规划。

## Rollback

发布候选失败则回到具体 P 阶段修复并重跑受影响矩阵，不通过清库、禁用校验或扩大权限绕过。
