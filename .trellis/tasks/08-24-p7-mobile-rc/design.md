# Design: P7 移动范围验收与内部 RC

## Evidence Model

每项验收必须链接到自动化测试、构建产物或设备记录之一。范围矩阵以父任务明确排除项为基线，发现范围回流先修计划而非临时实现。

## Release Matrix

按 Android 版本、手机/平板尺寸、浅色/深色、系统字体倍率和首次安装/测试包连续升级组织模拟器及 Widget 验证。Core Alpha schema 迁移、数据安全和凭据删除为阻断项；实体机与正式 Alpha APK 原位升级不在本轮范围。

## Artifact and Secrets

Debug APK 用于迭代验收；内部 RC 使用 Debug 签名的 Release APK。仓库只记录构建步骤、版本、校验值和证书指纹，不记录密钥。正式签名与商店分发留待后续任务规划。

## Rollback

发布候选失败则回到具体 P 阶段修复并重跑受影响矩阵，不通过清库、禁用校验或扩大权限绕过。
