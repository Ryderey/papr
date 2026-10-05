# Papr 个人 GitHub 同步方案设计

日期：2026-10-05。状态：待评审的设计，未实施。面向 Windows 桌面 Papr 与 Android Papr，个人使用。本文给出 V1 可执行合同与实施顺序；不代表现有版本已支持。所有建议的表、文件与接口均标注为拟新增，源码链接指向当前实现。

## 1. 已确定需求与交付边界

| 项目 | V1 合同 |
|---|---|
| 同步存储 | 一个 GitHub 私有仓库；无需腾讯云服务器、GitHub Actions 或本机 Git |
| 设备 | Windows 桌面与 Android，均继续使用 Papr |
| 订阅 | 源定义、显示名称、类型、文件夹归属、新增、改名、移动、取消订阅 |
| 文件夹 | 新增、改名、删除、排序；采用当前平面文件夹模型 |
| 文章目录 | 标题、链接、来源、发布时间、GUID/稳定标识 |
| 文章状态 | 已读、星标、稍后读三个独立字段；已读作为阅读同步的配套范围 |
| 普通文章目录 | 最近 90 天，用户已确认 |
| 保存文章 | 星标或稍后读任一为真，目录与状态长期保留 |
| 正文 | 各端自行获取；不上传摘要、正文、图片、音视频附件 |
| 设备配置 | 不同步主题、字体、AI 配置/Key、代理、通知、抓取频率 |
| 其他数据 | 标签、高亮、阅读位置、规则配置、IMAP 账号与邮件内容不在 V1 |
| 使用方式 | 本地读写优先；断网可阅读、编辑，恢复网络后同步 |

90 天规则只管理 GitHub 当前版本中的目录与状态，不自动清除本地正文缓存，不等于删除 Git 历史。未同步范围内的本地历史、规则或过滤行为可以导致界面不同；一致性验收针对共同目录窗口及相同展示条件。

支持 Papr 已有、可通过公开 HTTP(S) 源地址重建的 RSS/Atom/JSON Feed，以及使用这类地址的 podcast/youtube 等源类型。Newsletter/IMAP 连接和凭据排除；含 URL 用户名/密码的源禁止自动导出，需给出可识别的跳过原因。含查询参数的源地址仍可能包含访问凭据，应按私密订阅信息处理，不能承诺源 URL 完全没有秘密。

## 2. 当前实现依据

| 当前事实 | 对设计的影响 |
|---|---|
| Core 提供规范数据库迁移，桌面也调用该迁移 | 新 schema 只在 Core 维护 |
| 桌面使用 AppState 的 Connection，Android 使用 Core Db | 共用 SQL/协议，保留两种本地 Adapter；不为同步另开一套桌面 Db |
| 现有 GReader SyncPort 是 push/pull/ack，远端变更只有少量字段 | GitHub 的“读完整版本、合并、整体发布”使用并列模块，不硬塞进该接口 |
| 现有文章状态 pull 只匹配已有 URL | 必须新增缺失文章的目录导入 |
| 已有 read_later，但同步筛选不包含它 | 新日志必须捕获稍后读设置和取消 |
| 两端抓取遇到已有 GUID 时跳过插入 | 必须为目录占位文章增加正文补全路径 |
| 删除 feed 会级联删除 articles | GitHub 模式取消订阅要保留保存文章，不能直接复用级联删除 |
| Android 已有凭据插件及后台引擎支持 | 复用插件；Windows 新增系统安全存储 |

源码依据：[共享迁移](D:/Work/sync_remote_projtcts/papr/src-tauri/src/db.rs:39)、[桌面连接](D:/Work/sync_remote_projtcts/papr/src-tauri/src/state.rs:10)、[Core Db](D:/Work/sync_remote_projtcts/papr/crates/papr-core/src/db.rs:673)、[现有协议](D:/Work/sync_remote_projtcts/papr/crates/papr-core/src/sync.rs:82)、[文章应用](D:/Work/sync_remote_projtcts/papr/crates/papr-core/src/db.rs:2248)、[出站筛选](D:/Work/sync_remote_projtcts/papr/crates/papr-core/src/db.rs:2095)、[Core 插入](D:/Work/sync_remote_projtcts/papr/crates/papr-core/src/db.rs:2350)、[桌面插入](D:/Work/sync_remote_projtcts/papr/src-tauri/src/db.rs:663)、[源删除](D:/Work/sync_remote_projtcts/papr/crates/papr-core/src/db.rs:1057)、[Android 凭据插件](D:/Work/sync_remote_projtcts/papr/mobile/plugins/papr_credentials/android/src/main/kotlin/com/papr/papr_mobile/SyncCredentialPlugin.kt:7)。

## 3. 架构与模块 Interface

~~~mermaid
flowchart LR
  W["Windows Papr"] --> WA["Tauri Adapter"]
  A["Android Papr"] --> AA["Flutter Rust Bridge Adapter"]
  WA --> M["Core GitHubSyncModule"]
  AA --> M
  M --> S["本地 SQLite / 持久化待发操作"]
  M --> G["GitHub REST / 私有仓库"]
  W --> R["订阅源与原站"]
  A --> R
~~~

拟新增 GitHubSyncModule，承担身份、合并、发布、恢复与状态报告。两端只跨同一个 Interface：test_connection、preview_initial_sync、run_once、status、disconnect。界面接收类型化报告，不解析仓库 JSON 或决定合并规则。

内部有两个确实存在变化的 seam：

- LocalSyncStore Interface：准备有序本地批次/快照、持久化发布尝试、原子完成远端导入与确认。Core Db 和桌面 Connection 分别提供 Adapter，共用基于 Connection/Transaction 的 SQL helper。
- GitHubSnapshotTransport Interface：按固定 commit 读取快照、创建候选版本、非强制发布、查询提交是否已生效。真实 HTTP Adapter 与故障注入 Fake Adapter 用于同一合同测试。

合并函数是纯函数：输入远端有效快照与有序本地操作，输出候选快照、终结操作序号、冲突结果和变化路径。它不访问网络、凭据、系统时间或 UI。

保留现有 Reader SyncProfile/SyncPort；新增独立 GitHubProfile，并由外层同步入口选择 Reader 或 GitHub 流程。每个本地数据库同时只有一个主动同步后端，避免 GReader 与 GitHub 的写入互相传播；切换前停用原后端，不删除其业务功能或远端数据。

## 4. 仓库与认证

### 4.1 配置

建议专用私有仓库名 papr-sync，先在 GitHub 创建并勾选 README，使默认分支已有初始 commit。Papr 读取默认分支名称并固定保存，不假定一定叫 main。V1 不负责创建账号、仓库或空仓库首个 commit。

GitHubProfile（仅本地）包含：repository_id、owner、repo、branch、固定 path_prefix=papr-sync/v1、credential_ref、connection_generation。以不可变 repository_id 校验仓库身份，改名可更新显示地址，删库后同名重建必须重新接入。

只连接 api.github.com，V1 不支持 GitHub Enterprise 或自定义 API 域名。权限错误、仓库不存在/不可访问、公开仓库、未知数据格式、branch 保护阻止写入均给出独立原因。只读连接测试不能证明写权限；实际首次发布是写入验证，不为探测权限生成空 commit。

### 4.2 凭据

每台设备建议使用一个 fine-grained PAT：只选择同步仓库，Contents 读写，其余权限不额外申请；默认 Metadata 读取按 GitHub 要求。两台令牌便于分别撤销。PAT 到期后保留本地待发操作，暂停自动认证重试，要求更新令牌。

Windows 使用系统 Credential Manager；Android 复用现有 Android Keystore + AES-GCM 凭据插件。数据库只保存 credential_ref；令牌不进入 SQLite、仓库、URL、日志、崩溃报告或普通导出。Windows 系统存储失败时不能降级为明文。

首期不用 OAuth App/GitHub App，也不把客户端 secret 打入安装包。私有仓库提供访问控制，V1 不提供端到端加密；GitHub 及具有仓库权限者能读取同步数据。

依据：[GitHub PAT](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)、[GitHub Tree 权限](https://docs.github.com/en/rest/git/trees)、[Windows CredWrite](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew)、[Android 当前存储](D:/Work/sync_remote_projtcts/papr/mobile/plugins/papr_credentials/android/src/main/kotlin/com/papr/papr_mobile/AiCredentialStore.kt:17)。

## 5. 仓库布局与数据合同

~~~text
README.md
papr-sync/v1/
  manifest.json
  subscriptions.json
  articles/
    00.json ... 3f.json
  states/
    00.json ... 3f.json
~~~

V1 固定 64 个逻辑分片；按 article_key 的前一字节取低 6 位分配，路径用两位小写十六进制。空分片不创建。每篇文章固定一个分片，不因日期改动或星标改变而移动。最多 130 个协议文件，避免每篇一个文件，也避免只改星标就重写整篇目录。

subscriptions.json 存文件夹、订阅、排序、文件夹别名和订阅/文件夹墓碑；体积小，结构相关操作一起提交。articles 分片只存目录；states 分片存三个状态及已发生修改的版本号。只有变化文件被写入新 Git tree。

序列化约定：UTF-8 JSON、固定缩进/换行、对象键按字节序排序、数组按合同排序，禁止重复键、NaN、浮点时间戳。时间使用 UTC RFC3339；seq/generation 为安全范围内的整数，版本上限不得超过 2^53-1。字段缺失与显式 false 不同。

### 5.1 manifest

以下示例 ID 为便于阅读的缩写；实际 dataset/device ID 为 32 位十六进制，文章/源 key 为 64 位 SHA-256 十六进制。

~~~json
{
  "protocol_version": 1,
  "dataset_id": "dataset_01",
  "epoch": 1,
  "shard_count": 64,
  "retention": {
    "ordinary_days": 90,
    "protected_forever": true,
    "cutoff_at": "2026-07-07T00:00:00Z"
  },
  "devices": {
    "desktop_01": {
      "processed_seq": 42,
      "last_batch_id": "desktop_01:42",
      "last_rejections": []
    },
    "android_01": {
      "processed_seq": 17,
      "last_batch_id": "android_01:17",
      "last_rejections": []
    }
  }
}
~~~

dataset_id 标识这一套资料，epoch 标识协议级重建代次。devices 的 processed_seq 表示连续前缀已经终结处理，包括已应用、幂等无效操作或带明确理由拒绝的操作，不等于每项都执行成功。拒绝列表与本批次一起发布，原设备在超时恢复时也能取得原因；批次最多 500 项，所以列表可完整保存最近一批而不丢失该批恢复信息。

head commit SHA 就是整个远端快照版本，不另造全局递增 revision。manifest 不引用自身 commit SHA 或文件 blob SHA，避免构造循环；路径到 blob SHA 由 Git tree 给出。

### 5.2 subscriptions

文件夹：folder_id、name、deleted、name_version。folder_order 是存活 folder_id 的完整顺序列表；order_version 是排序操作版本。V1 没有嵌套层级，也不新增当前不存在的订阅自定义排序字段。

订阅：feed_key、feed_url、source_type、display_title、custom_title、folder_id、active、generation，以及用户可编辑字段版本。自动抓取不得覆盖 custom_title。删除墓碑保留 feed_key、generation、active=false；保存文章所需的源名称和地址也保留。

文件夹别名仅用于合并离线首次创建的同名文件夹，指向存活规范 ID；禁止循环。改名引起的撞名不自动合并，按第 8 节拒绝并报告。

### 5.3 articles 与 states

~~~json
{
  "article_a": {
    "feed_key": "feed_a",
    "identity_kind": "guid",
    "identity_value": "urn:example:post:123",
    "guid": "urn:example:post:123",
    "title": "文章标题",
    "url": "https://example.com/posts/123",
    "published_at": "2026-10-05T01:00:00Z",
    "first_seen_at": "2026-10-05T02:00:00Z"
  }
}
~~~

~~~json
{
  "article_a": {
    "read": { "value": true, "version": "desktop_01:40" },
    "starred": { "value": true, "version": "desktop_01:41" },
    "read_later": { "value": false, "version": "android_01:17" }
  }
}
~~~

从未修改的状态字段可缺省为 false/version=null；已修改成 false 仍保留显式值和版本，直到该文章因保留规则移出目录。星标与稍后读不互斥，read=true 也不自动清除它们。不得对两个客户端的正文、摘要、抓取错误、缓存时间或 AI 结果做投影。

元数据合并只补齐缺失信息；同一身份已确认的非空标题/链接优先保留，V1 不争夺“上游标题修订最后更新时间”。发现明显矛盾记诊断，不因此重置用户状态。

## 6. 跨设备身份与去重

### 6.1 源

feed_key = SHA256(length-prefixed UTF-8("feed:v1", canonical_feed_url))。

canonical_feed_url 使用共用 Rust URL helper：去掉首尾空白、标准化 scheme/host 和默认端口、移除 fragment；保留路径大小写、尾斜线语义、查询参数和值及顺序。不把 HTTP 与 HTTPS、不同跳转地址或带不同参数的源武断合并。两个客户端必须用同一 helper；源地址改变视为另一个源，改显示名不改身份。

generation 处理同一 URL 的取消/重订：
- 新源 generation=1；两端首次新增同一地址可合并。
- 取消订阅针对已知 generation，置 active=false。
- 明确重新订阅需针对已观察到的墓碑 generation 创建下一代。
- 旧代的离线改名、移动、抓取结果或删除不能改变新代，也不能复活旧订阅。
- 未见过云端墓碑的新设备不能通过初始化合并自动重订；显示冲突，让用户同步后明确重订。

### 6.2 文件夹

folder_id 在创建事务中生成 128 位随机 ID，改名保留 ID。现有同名文件夹在首次合并按当前 trim + ASCII 大小写不敏感规则匹配；两个新建 ID 如代表同名文件夹，复用先被远端接受的 ID，保存别名并改写归属引用。不能用名称作为永久身份，否则改名会变成删除和新建。

### 6.3 文章

article_key = SHA256(length-prefixed UTF-8("article:v1", feed_key, identity_kind, identity_value))。

优先使用解析/数据库中的稳定非空 GUID；没有 GUID 时回退到同一源内规范化 URL。GUID 和 URL 都缺失的条目不纳入 V1，同步状态需报告数量。数据库已有的 opaque GUID 不擅自改写。

不同源中的同一链接保留两条身份，避免将一个源的星标强加给另一个源。URL 回退到真实 GUID 的升级只在同源、唯一候选且能确认原项为 URL 回退项时建立身份别名；不同的真实 GUID 不能仅凭相同链接强合并。无法判断的旧数据保留并记重复候选，优先不丢记录。

每端自增 id 只是本地外键，绝不上云。GitHub 身份与 GReader remote_id 分开保存；导入目录沿用 GUID，并用映射定位本地行。

## 7. 本地持久化与写入合同

拟新增 Core 迁移；迁移只建表/列与索引，不联网，不导出数据：

| 拟新增存储 | 内容与约束 |
|---|---|
| github_sync_connection | 本地配置、dataset/epoch、设备 ID、next_seq、last_confirmed_head、连接 generation、租约 |
| github_sync_outbox | connection、device、seq、类型、实体稳定 key、完整必要载荷、base_version；无业务外键级联删除 |
| github_sync_shadow | 路径、已确认 blob SHA 和已验证 JSON；全部属于同一已确认 head |
| github_sync_attempt | 本批次到 N 的操作、base head、候选 commit、候选结果；用于超时/退出恢复 |
| github_sync_entity_map | dataset 范围内的稳定 key 到本地 id、文章身份种类、订阅 generation、归档/本地抑制标志 |
| github_sync_conflicts | 本端有限保留的拒绝/同字段覆盖结果，已读状态与时间；不存令牌 |
| articles 新字段 | metadata_only；便于正文补全，值不上传 |
| feeds 新字段 | subscription_active（默认 1）；用于 GitHub 模式取消订阅与归档源查询 |

复用现有 folders/feeds 的 sync_id 字段作为本地稳定身份线索，正式远端映射仍按 dataset 隔离；GReader 既有 remote_id_map 保持独立，不拿通用列同时承载两个提供方含义。最终 SQL 命名按实施时的 schema 版本确定，不能复用旧版本号。

所有同步域的本地修改与 github_sync_outbox 插入必须在同一个 SQLite 事务完成：
- 创建/改名/删除文件夹、排序。
- 新增/改名/移动/取消订阅。
- 抓取新增文章的目录，以及规则造成的 read/star 初始状态。
- 单篇或批量已读、星标、稍后读设置/取消。
- OPML 导入产生的同步域变化。

共用 capture helper 定义操作载荷；桌面和 Core 的业务写入口都调用它。不能只改 Tauri command，因为定时抓取、OPML 与批量规则可绕过 command。业务行删除前先截取稳定 ID 和必要载荷，不能事后 JOIN 已删除行拼操作。

已连接时：事务分配无空洞的每-device/per-connection seq，operation_id=device_id:seq。批次有序处理至 N，网络期间的新编辑只进入 N 后缀。相同字段多次操作可以在纯函数中折叠，但不提前删除本地记录，也不跳过未终结前缀。

base_version 取该字段当前已知版本：有本地待发前驱时引用前驱 operation_id，否则引用 shadow 版本。真正的用户 setter 即使最终值回到起点，也保留这次意图；抓取到相同目录、远端导入或无变化的后台投影不能伪造编辑。seq 在同一设备与 dataset 的命名空间内不因断开/重连归零；缺少本地序列状态时要结合远端水位恢复或创建新 device_id。

V1 操作种类固定如下；都包含 operation_id、seq、dataset_id。操作记录只保存在本地 outbox，远端发布的是合并后快照和处理水位，不额外保存无限操作日志。

| kind | 必要载荷 |
|---|---|
| CreateFolder / RenameFolder / DeleteFolder | folder_id、name（创建/改名）、base_version（已有实体） |
| ReorderFolders | 有序存活 folder_id 列表、base_version |
| Subscribe | feed_key、公开源定义、初始归属；只在无旧代或已存活同源时创建/复用 |
| SetSubscriptionField | feed_key、generation、字段名、精确值、base_version |
| Unsubscribe / Resubscribe | feed_key、明确观察到的 generation；重订还携带必要源定义 |
| EnsureArticle | 稳定 key、来源、GUID/身份种类、标题/链接/日期；不重置状态 |
| SetArticleState | article_key、read/starred/read_later 之一、true/false、base_version、必要目录载荷 |
| SeedInitial | 预览确认后的初始数据暂存引用及扫描边界，只在初始化流程解释 |

用户可编辑的订阅字段白名单为 display_title/custom_title、folder_id；源 URL 更换采用新源身份。EnsureArticle 与 SeedInitial 不能模拟普通用户的状态取消；批量状态操作逐项展开后仍受单批预算约束。

自动本地缓存清理或本地 skip 规则不生成云端“删除文章”；排除内容可用本地抑制标记保持既有本端行为，导入不得造成清理/再导入的反复循环。本地过滤不改变云端共同目录。用户明确取消订阅、删除文件夹才产生对应远端生命周期操作。

远端导入使用专用路径，不调用会再次捕获操作的普通业务 setter，也不重跑各端规则。导入、FTS、映射、shadow/head、终结日志和冲突记录原子提交；发生错误整体回滚。

## 8. 合并规则：以操作为单位，不覆盖整份本地快照

云端状态是上次 GitHub 接受的版本；本地 outbox 表示本端尚未终结的意图。每次合并都从最新有效云端快照开始，按序重放本地操作，不能把“本地不存在”普遍解释为删除。

| 情况 | V1 规则 |
|---|---|
| 不同文章/字段并发修改 | 分别保留 |
| 同字段并发相反操作 | 后被有效 commit 接受的操作生效；记录 base_version 已变化的冲突 |
| 网络重试 | processed_seq 已覆盖的操作不再重放 |
| 首次合并已有数据 | 已读/星标/稍后读为真时做并集，只用于初始化；不会用本地 false 清除云端 true |
| 普通状态同步 | 精确写 true/false，不能取 OR，否则无法取消 |
| 订阅旧代编辑遇到墓碑/新代 | 拒绝旧代操作，不复活源，报告原因 |
| 文件夹删除与源移动冲突 | 删除文件夹生效，仍引用它的源置未分类；移动到其他存活文件夹仍可生效 |
| 文件夹删除 | 不删除订阅，不删除文章 |
| 同名新建文件夹 | 合并 ID 和引用 |
| 改名撞到其他存活文件夹 | 保留远端已接受结构，拒绝该改名，提示换名 |
| 文件夹排序并发 | 后接受的完整顺序优先；过滤已删 ID，将未出现的新 ID 按稳定 key 追加 |
| 元数据缺失与状态操作 | 状态操作携带必要目录；可以在同一 commit 恢复过期旧文章 |
| 批量“全部已读” | 记录点击时确实匹配的文章，不包含随后才出现的文章 |

这里的“后”是 GitHub 接受提交的顺序，不是手机/电脑上的编辑时间。离线数天的旧操作晚提交，可能覆盖此前另一端对同字段的操作；这是 V1 明确取舍。不同字段不受影响，用户无需逐条确认阅读状态冲突。结构拒绝项要可查看并重做，不静默丢掉。

示例：
1. 电脑 starred=true，手机 read_later=true：最终两者都为真。
2. 两端操作同一 starred，手机最后成功发布 false：最终取消星标。
3. 同一已确认批次响应丢失，另一端后来改变 starred：重试不得把旧批次再覆盖回来。
4. 取消订阅后，离线设备仅抓取到旧源新文章：不能把源重新订阅。
5. 90 天以外文章本端加稍后读：上传标题/链接与 read_later=true，恢复长期保留；已过期且没有待发修改的其他状态不凭旧本地副本复活。

## 9. 一次同步的完整流程

~~~mermaid
sequenceDiagram
  participant UI as Papr
  participant DB as 本地数据库
  participant GH as GitHub
  UI->>DB: 单飞/租约，冻结批次至 N
  DB-->>UI: 待发操作与已确认版本
  UI->>GH: 读取 head H，固定 H 的 tree/blob
  GH-->>UI: 完整有效远端快照
  UI->>UI: 跳过已处理操作，按字段合并
  UI->>DB: 持久化本次尝试
  UI->>GH: 创建 tree T，创建 parent=H 的 commit C
  UI->>DB: 持久化候选 C
  UI->>GH: 更新分支到 C，force=false
  alt 成功
    UI->>DB: 原子导入 C + 重放 N 后缀 + 确认前缀
  else 并发冲突
    UI->>GH: 重读新 head，重新合并与发布
  else 响应未知
    UI->>GH: 查当前 head 的 processed_seq
    UI->>DB: 已生效则确认，否则保留待发
  end
~~~

### 9.1 读取

1. 取得本地同步单飞锁；Android UI/后台可能有独立 Core 实例，再用 SQLite 租约避免重复工作。租约只优化调度，不能作为远端一致性的唯一保障。
2. 短事务读取连接 generation、device_id、shadow 和到 N 的批次；释放数据库锁再联网。
3. GET git/ref 获取 head H。H 相同且无本地变化/无到期目录维护时快速结束，不产生空 commit。
4. H 变化时，核对与 last_confirmed_head 的祖先关系，读取 H 对应 commit/tree，按 blob SHA 仅下载变化路径；所有文件固定在 H，不能按活动分支逐个读。
5. 校验 manifest、dataset、epoch、版本、字段、路径、文件数量、身份、引用与上限。协议路径只允许固定前缀内约定的普通 blob，不接收符号链接/子模块或仓库指定的下载域名；GitHub Authorization 不发送给原站正文请求。新安装完整读一次；后续用本地 shadow 拼回有效完整快照。
6. 若 tree truncated，按子树分层补全后再验证，不能将未取得的文件当成删除。任何片段缺失/异常均不推进 head。

变更路径只是传输优化，不代表用户删除；文章消失可表示目录过期，真正的结构删除由墓碑判定。

### 9.2 合并与发布

7. 先将远端已处理的本端前缀及 last_rejections 持久化为本地终结结果，再组织尚未处理的批次。若远端 processed_seq 已覆盖本地尝试的 N，恢复确认，避免重放旧操作。不得尚未收回上一批拒绝结果就发布下一批并覆盖它的远端 receipt。
8. 将尚未处理的连续操作按序应用到远端快照，处理身份别名/结构引用，记录拒绝和覆盖冲突。
9. 对完整候选执行保留规则与最终不变量校验，形成确定序列化文件及该设备处理水位。
10. 在本地持久化尝试，不删除 outbox。无 outbox 且只有有效远端变化时，直接原子导入 H，无需写 GitHub。
11. 用 Git Database API 创建 tree：base_tree=H 的 tree，仅带变化路径；小文本可直接用 tree entry 的 content，避免每文件再调用一次 blob 写入。大批次按预算先创建 blob，再引用 SHA。
12. 创建 commit：tree=T，parents=[H]。持久化候选 SHA 后 PATCH refs/heads/{branch}，明确 force=false。只有推进分支才表示发布完成，创建 blob/tree/commit 本身不算成功。
13. 如果对方已从 H 发布了另一个 commit，C 是其兄弟节点，非快进更新被拒绝。重新读取新 head，重新合并相同待发意图并创建新候选；不强推、不复用旧 tree 直接改 parent。最多 3 次竞争重试，之后延后本轮并保留队列。

### 9.3 完成本地应用

14. 成功后原子导入已确认快照、映射和 shadow，更新 last_confirmed_head，终结处理到 N 的日志并保存冲突结果。
15. 事务内重新读取 N 之后的新操作并覆盖到展示状态，保护请求在途的新编辑；不能用出发时的整份本地快照覆盖数据库。
16. 发布若成功而本地事务失败，下次依赖 manifest 水位恢复，不重复覆盖。
17. 连接/账号已变化则拒绝旧结果落入新连接；不销毁旧连接待发记录。取消请求不保证已发出的 GitHub PATCH 没有生效，旧仓库结果按旧连接的恢复流程处理。

GitHub 支持 tree 基于旧树更新、commit 指定 parent、ref 非强制快进更新。上述多文件发布及并发协议是基于这些能力的本方案设计，需集成验证。[Trees](https://docs.github.com/en/rest/git/trees)、[Commits](https://docs.github.com/en/rest/git/commits)、[Refs](https://docs.github.com/en/rest/git/refs)、[Commit 比较](https://docs.github.com/en/rest/commits/commits#compare-two-commits)。

## 10. 崩溃、超时与幂等恢复

| 故障位置 | 恢复规则 |
|---|---|
| 业务写入过程中退出 | 业务与日志一起回滚或一起提交 |
| tree/commit 创建失败 | outbox 不变；已有不可达对象不代表资料已发布 |
| commit 已创建、ref 未更新 | 可重试推进；若竞争发生则重建候选 |
| ref 更新请求超时 | 状态标为结果待确认，读取当前有效快照的设备水位 |
| remote waterline >= N | 该前缀已终结；导入最新快照并确认，不重放 |
| remote waterline < N | 仍待发，重新合并；旧在途 PATCH 与新候选竞争时由非快进规则和水位防重 |
| GitHub 成功，本地 apply 失败 | 下次通过远端水位重建本地应用 |
| 本地 apply 后进程退出 | 同一事务中的 shadow/head/log 确认保持一致 |
| token 过期、权限不足 | 保留队列与本地操作，暂停自动认证重试 |
| 连接切换 | 增加 connection_generation，丢弃旧结果对新连接的应用 |
| 仓库重置/历史分叉/水位倒退 | 暂停并报告 remoteHistoryChanged，禁止自动强推或清空队列 |

device_id 必须在每台安装独立生成。复制数据库、恢复备份或重装时，先核对本机安装标识；不能让两个设备共用同一个 device_id/seq 空间。安装标识放在不会随数据库导出传播的系统存储，本地数据库中的 device_id 只是绑定副本。

恢复发现设备身份不匹配时：先读取远端，按旧水位判断已有操作是否处理，再把剩余意图迁移为新设备序列；已确认的旧状态不当作新编辑再次上传。此路径必须有故障测试。

发布租约使用本地 SQLite 原子获取与 fence；短期过期可恢复。正确性仍依赖远端快进检查、唯一序列和水位。不能持有数据库写锁等待网络。

## 11. 首次同步、迁移与切换

### 11.1 首台

1. 输入 owner/repo 和 PAT，读取验证私有仓库、初始分支、格式。
2. 本地预览：可导出源、文件夹、90 天普通目录、全部保存条目、被排除条目及预计体积；不显示或导出正文。
3. 云端未初始化时，推荐“将本机资料建立为同步资料”；用户触发后才初始化 dataset。并发初始化只允许第一个有效 commit 建立 dataset，后来者重读并改走合并，不能另建一套覆盖。
4. 先在短事务进入 initializing 并开启持久化 capture，再建立一致的本地读取快照、分页扫描/暂存及初始导入批次。将扫描基线与扫描期间已经捕获的操作按序合并；身份回填与删除载荷也由共享 helper 负责。不能扫描完才开启 capture，也不能长时间阻塞写入。初始化被取消仍保留本机资料和暂存队列，确认重新初始化前不自动上传。
5. 第一次成功后开启自动同步。

### 11.2 第二台

读取现有 dataset，推荐“合并本机与云端”。同源按地址、文件夹按规则匹配；普通目录取满足保留策略的并集，保存条目取并集。已有云端非空源名称/归属优先；状态 true 并集仅用于这一初始化步骤。初始化后的取消状态按普通操作精确同步。

另提供“使用云端作为同步起点”：当前同步域先做本地备份，再按预览应用；本地主题/字体/AI 不受影响。V1 不提供自动“本机覆盖整个云端”或远端清空功能，避免旧副本误覆盖历史。

界面等待确认期间 head 若改变，重新预览并更新冲突数，不能用旧预览直接覆盖新版本。与已有 tombstone 冲突的本机源不自动恢复。

### 11.3 旧 GReader 与新仓库

首次接入 GitHub 不复用 GReader 的文章 remote_id 或待发队列；从现有本地资料建立新的初始基线。旧 GReader 功能仍保留，但自动同步要停用，未发送的旧后端操作仍可提示处理。

仓库/dataset 切换需要新的本地命名空间及首次预览。移除连接只停止同步并删除该设备令牌，不清空本地文章，不删除 GitHub 仓库；待发日志在本地保留，可恢复同一连接或导出备份。

协议升级：未知 major version 直接暂停，不尝试部分写入。V1 的显式字段白名单和“拒绝未知格式”防止旧客户端丢新字段。未来升级需要设计独立迁移，不作为 V1 背景动作。

## 12. 正文补全与取消订阅

### 12.1 目录占位

远端目录落为本地 article 行：保留真实 GUID/URL/title/date，content_html/extracted_html 不填，body_text 为空，metadata_only=1。FTS 至少索引标题，列表可正常展示和操作。

本端后续 RSS 抓取遇到同源同 GUID 的 metadata_only 行时，补齐正文、摘要、附件及 FTS；不另插一篇，不重置 read/starred/read_later，不重跑会覆盖状态的初始规则，也不把补全计入新文章通知。桌面的 URL dedup 提前返回也需要避开占位行。

RSS 已不含条目时，打开文章可按链接获取原站正文或跳转原文；失败保留目录和状态，允许重试，不阻断其他同步。没有链接的真实 GUID 条目可显示标题但无法承诺补正文。

### 12.2 取消订阅

拟采用 GitHub 模式的安全语义：取消抓取、取消普通订阅展示，保存条目仍留在星标/稍后读列表，保留一个归档源用于满足 articles.feed_id 外键。普通目录随后受 90 天策略约束，本地缓存是否清理仍由原有本地策略决定。

因此 GitHub 模式不直接调用当前 delete_feed 的级联删除；使用 subscription_active=0，并让正常源列表与刷新查询过滤归档源，保存文章查询仍可 JOIN 它。明确“清除本地资料”与“取消订阅”是不同用户动作，原有未连接模式的删除行为不在这次改造中擅自改变。

源重新订阅开启下一代，article_key 仍以 feed_key/GUID 定义，可复用保存文章与已有正文；旧代队列不得操作新代订阅结构。

## 13. 保留策略与容量预算

### 13.1 90 天目录

- 使用可信有效 published_at；缺失时使用第一次确认的 first_seen_at。
- 未来异常日期按首次确认时间保守处理；first_seen_at 不随另一端重抓刷新，不延长所有旧文章寿命。
- 清理参考 GitHub HTTP 响应时间并进行合理性校验，不用设备错误时钟直接大量清理；缺少可信时间就暂不清理。cutoff_at 在同一策略下只向前推进。
- 先合并本地操作，再清理；starred=true 或 read_later=true 的条目绝不按时间清理。
- 超期且两状态均 false 的目录及状态从当前 tree 移除；不向各端发送“删除本地正文”指令。
- 本地反复重新抓到旧 GUID 不重新上传过期普通项；旧项重新加星/稍后读时必须带完整必要目录恢复。
- 所有订阅/文件夹墓碑和设备处理水位在 V1 保留。设备退役不是自动删水位，避免长期离线设备重放。
- 未同步的高亮不构成云端保护条件，但本地既有高亮/缓存保护仍有效。

### 13.2 拟定工程预算（不是服务承诺）

| 项目 | 初始预算 |
|---|---|
| 单分片 JSON | 目标不超过 1 MiB |
| 单次普通批次 | 最多 500 个操作；大批量分成多次完整提交 |
| 单次 tree 写请求 | 目标不超过 2 MiB；超出改用 blobs 或减少批次 |
| 当前完整目录与状态 | 先按 50 MiB 软预算做连接预览与性能验证 |
| GitHub 报告仓库体积 | 500 MiB 提示增长，1 GiB 强提示维护评估 |
| 瞬时 HTTP 下载 | 最多 4 个并发；写入串行 |
| 首次导入 | 可取消、显示进度，SQLite 分页暂存后原子应用 |

达到预算时给出明确容量错误和待发数，不静默截断文章、不丢保存条目、不假报成功。64 分片确有不足时，需要后续协议升级/分片扩容设计；V1 不悄悄改变 shard_count。

估算示例：20,000 条文章，每条目录及状态合计假设 600–1,000 字节，当前快照约 12–20 MB；实际须用真实序列化样本测量。首次下载完整快照，之后只传变化分片。此数字不包含 Git 历史，不能据此保证永久免费或历史增长速率。

GitHub 建议小文件与健康仓库结构，程序生成文件通常更适合对象存储；个人采用 GitHub 是基于少量元数据和低频写入的取舍，不能扩张为正文网盘。仓库大小读取是诊断指标；删当前文件不释放全部历史，V1 不自动 force push、改写历史或删除旧仓库。[GitHub 仓库限制](https://docs.github.com/en/repositories/creating-and-managing-repositories/repository-limits)

## 14. 调度、错误与产品状态

### 14.1 调度默认值（拟定）

| 触发 | 行为 |
|---|---|
| 启动/回到前台/网络恢复 | 至少间隔 60 秒触发一次检查，合并重复触发 |
| 手动同步 | 可立即请求，但遵守服务端 Retry-After，不绕过限流 |
| 本地编辑 | 延迟 10 秒合并；连续编辑最多等待 60 秒，写入最小间隔 60 秒 |
| RSS 批量刷新 | 抓取批次完成后触发，不每条提交 |
| 前台没有编辑 | 每 5 分钟检查远端 head，仅应用有变化的版本 |
| Android 后台 | 复用 WorkManager 刷新任务，初期约每 6 小时补同步；系统调度不保证精确时间 |
| 目录清理 | 每日有可信时间时评估，尽量搭车正常提交，不生成频繁维护 commit |
| 网络失败 | 带抖动退避，30 秒、1 分钟、2 分钟、5 分钟，之后最长约 30 分钟 |

上述是客户端设计参数，不承诺另一端在固定秒数内看到变化。App 关闭时没有 GitHub 抓取器；无实时推送。

### 14.2 请求预算

GitHub PAT 认证请求通常每账号每小时 5,000 次；内容生成请求另有通常每分钟 80、每小时 500 的二级限制，部分端点更低，可变。[官方限流](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)

一次普通发布采用 tree(content)+commit+ref，约 3 个写请求，不是 1 次；使用额外 blobs 或竞争重试会增加。读请求包括 ref、commit、tree、变化 blobs，以及必要的祖先检查。两端共用账号额度。每分钟最多一次正常发布的设计有助于控制开销，但不是一定不会限流的证明。

401 暂停并提示重新配置令牌；403 区分权限与限流；429/限流 403 按 Retry-After、X-RateLimit-Reset 处理。422 不一概视为并发：读取最新 ref 判断是否 head 变化；未变化则报告格式/权限/保护规则错误，不盲目重试。404 不当作“空数据”覆盖，避免私人仓库权限丢失造成清空。

### 14.3 设置界面

连接表单：GitHub 私有仓库 owner/repo、令牌、可展开分支选择。显示固定同步范围与“普通目录 90 天，星标/稍后读长期保留”。提供连接测试、首次预览、立即同步、更新令牌、断开。

连接后状态：上次成功、待上传数量、结果待确认、网络/限流等待、身份/格式错误、需要处理的结构冲突。同步期间本地操作仍可用。不能只显示“已连接”而掩盖长时间未成功。

SyncReport 至少包括 pull_catalog_added、state_updates、processed_operations、rejected_operations、concurrent_overwrites、pending_count、last_success_at、stable_error_code。UI 不展示底层 SQL、SHA 或完整 HTTP 请求作为常规操作步骤；诊断导出需脱敏。

## 15. 首期实施文件范围与依赖判断

以下为拟修改范围，本轮没有改动这些代码。

| 范围 | 主要工作 |
|---|---|
| crates/papr-core/src/sync/github/ 下拟新增模块 | model/merge/store/transport；统一协议与纯合并 |
| crates/papr-core/src/db.rs | 共享迁移；稳定身份、目录导入、占位补全、捕获事务与查询过滤 |
| crates/papr-core/src/services/sync.rs、services/settings.rs | GitHub 外层路由、状态、配置；保留 GReader 合同 |
| crates/papr-core/src/ingestion/ 与相关 services | 抓取、OPML、批量状态的完整捕获 |
| src-tauri/src/db.rs、commands.rs、state.rs、sync.rs | 桌面 LocalSyncStore Adapter、触发、业务事务捕获与后端切换 |
| src-tauri/src/credentials.rs 拟新增 | Windows 系统凭据 Adapter |
| src/components/SettingsDialog.tsx、src/lib/api.ts 与相关状态/语言资源 | GitHub 配置、首次预览、报告、错误呈现 |
| crates/papr-flutter-bridge/src/api.rs、dto.rs | 显式桥接接口和类型转换 |
| mobile/lib/repositories/sync_repository.dart、ui/screens/sync_settings_screen.dart | GitHub 表单/流程，复用凭据插件 |
| mobile/lib/services/background_refresh_service.dart | GitHub 后台分支与调度 |
| Rust/Dart 生成桥文件 | 通过现有 rust_frb_codegen.yaml 再生成，不手工改生成内容 |
| 相应 Rust、React、Flutter 测试 | 下节合同与平台接入验证 |

已有 reqwest、serde/serde_json、rusqlite、tokio、chrono、url 足够承担 HTTP、JSON、事务与调度。不引入 Node 服务、Git 客户端、libgit2、Redis、远端数据库。

拟增加的少量直接依赖：
- Core sha2：为跨设备身份使用标准 SHA-256；现有业务依赖没有可直接声明使用的同等 helper。影响 workspace/core Cargo 配置与 Cargo.lock。不能用不稳定的 DefaultHasher 代替，也不自行实现密码学哈希。
- Windows 专用 windows-sys 最小 Win32 credentials feature：调用 CredWrite/CredRead/CredDelete；影响桌面目标依赖和凭据模块。现有 SQLite token 保存不满足安全存储要求，跨平台 keyring 为本轮 Windows/Android 引入的范围更大。
- 128 位随机非秘密 ID 可利用 SQLite randomblob 在创建事务生成，避免仅为 ID 添加 uuid。Android 复用现有插件，不新增安全存储大依赖。

依赖版本与锁文件以实施时仓库和平台构建验证为准；本轮没有安装依赖。如现有直接依赖已经能提供同等功能，优先复用并调整实施计划。

## 16. 分阶段实施与退出条件

| 阶段 | 交付 | 退出条件 |
|---|---|---|
| P0 合同冻结 | 数据格式、身份/生命周期、合并、真实样本预算 | 本设计及样本评审完成，接受离线同字段覆盖语义 |
| P1 Core 本地能力 | schema、capture、纯 reducer、占位导入/正文补全 | 事务/身份/保留/在途修改测试通过 |
| P2 GitHub Transport | 固定 head 读取、tree/commit/ref 发布、限流和超时恢复 | Fake HTTP 故障测试通过；专用私有测试仓库验证真实竞争 |
| P3 Windows 端 | 凭据、首次同步、手动流程与状态 | 本地真实资料备份后完成单设备验证 |
| P4 Android 端 | 桥接、凭据复用、首次合并、前后台流程 | Windows/Android 双设备与离线场景通过 |
| P5 自动化与发布 | debounce、Worker、容量提示、升级/回退说明 | 所有必须验收项通过，协议兼容测试与平台构建通过 |

不单独发布“能保存到 GitHub、但不能合并”的半成品为同步功能。P1/P2 可内部开发测试，端到端能力未齐前不开默认自动上传。不在这份设计阶段创建远端资源或开始实施。

回退：停用 GitHub 自动同步、导出本地同步域和确认版本备份，保留只读归档数据；不退回不能识别新 schema 的旧二进制直接打开新库。若需回旧版，使用升级前数据库备份，明确升级期间新操作的迁移方式。远端错误恢复使用新 epoch/独立仓库的显式迁移流程，不自动 reset/force push。

## 17. 必须通过的验收矩阵

| 编号 | 场景 | 通过标准 |
|---|---|---|
| A01 | Windows 新增源/文件夹，Android 同步 | 归属正确，本地 id 可不同 |
| A02 | Android 独有文章 | Windows 出现相同标题/链接与来源，正文占位可操作 |
| A03 | 两端分别加星与稍后读 | 两字段都为真 |
| A04 | 取消星标/稍后读/已读 | 精确 false 到达另一端，不被 OR 恢复 |
| A05 | 同字段并发与时钟相差一天 | 按成功发布顺序收敛，设备时钟不影响赢家 |
| A06 | 两端从同一个 H 发布 | 一次非快进拒绝，重读合并后保留双方不同字段 |
| A07 | ref 成功后响应丢失，再有另一端修改 | 重试不重放旧批次，不覆盖新状态 |
| A08 | 本地业务写或 apply 中断 | 数据与日志/游标一起提交或回滚 |
| A09 | 网络中用户继续编辑 | N 后缀保留，不能误确认或覆盖 |
| A10 | 占位文章再抓取/全文提取 | 补全同一行，保留状态和标题索引，不重复通知 |
| A11 | 删除文件夹/移动到已删文件夹 | 源变未分类，不被删除 |
| A12 | 取消源并由离线旧设备更新 | 不复活；星标/稍后读条目仍可访问 |
| A13 | 同 URL 重订，新代遇到旧代删除 | 新代不被旧操作取消 |
| A14 | 同名创建与改名撞名 | 前者合并归属；后者有可恢复拒绝提示 |
| A15 | 普通项 90 天过期 | 当前云目录移除，本地正文不因同步被清除 |
| A16 | 数年前保存项/过期项离线重新保存 | 保存项不失去；重新保存携带目录恢复 |
| A17 | 首次合并含已有 false/true | 保存条目并集；以后取消仍传播 |
| A18 | token 到期/限流/网络断开 | 本地读写可用，待发持久，正确暂停/退避 |
| A19 | 分片缺失/损坏/未知版本/tree 截断 | 不部分导入、不推进 head、不覆盖远端 |
| A20 | 仓库重建/分叉/epoch变化/连接切换 | 暂停隔离，旧响应不应用到新连接 |
| A21 | 数据库克隆/Android 恢复备份 | 不复用写设备身份，不重放已确认旧状态 |
| A22 | OPML、批量已读、规则、缓存清理 | 必须捕获的操作完整；本地清理不云删除/反复导入 |
| A23 | 同步字段排除 | 仓库与日志无正文、AI Key、PAT、主题/字体等 |
| A24 | 同 head、无编辑、无维护 | 不生成空 commit |
| A25 | 两端真实网络与 Worker | 手动/前台同步可用，后台实际凭据可读，延迟符合平台事实 |

未来验证命令：cargo test -p papr-core；cargo test -p papr-flutter-bridge；cargo check/test --manifest-path src-tauri/Cargo.toml；pnpm build；pnpm test；mobile 下 flutter analyze、flutter test、Android debug 构建与设备测试。bridge 生成器使用仓库配置和锁定工具版本；不在设计阶段执行代码生成。

真实私有仓库测试需单独测试仓库与最小权限凭据；验证前备份本地实际资料。这里只列未来验证，不声称已运行。

## 18. 本轮设计验证与实施前决策

已完成静态源代码对照、官方 API 能力核对和设计场景审阅。本轮只新增本文件，未修改业务代码、安装依赖、访问令牌、创建仓库、写 GitHub 或运行构建/测试。

产品范围已确定，没有必须继续询问才能出设计的事项。实施前评审应确认：
- 接受同字段按提交接受顺序处理，离线编辑时间不决定赢家。
- 接受 GitHub 模式取消订阅保留保存条目，以及归档源支持带来的本地查询调整。
- 接受 V1 只支持 Windows/Android 的安全凭据路径；其他桌面系统不能明文降级，需要另行适配。
- 用真实目录样本验证固定 64 分片、50 MiB 软预算及首次导入耗时，再冻结具体性能上限。

这些是评审合同和验证任务，不是本轮实施授权。前期调研参考：[个人同步方案](D:/Work/sync_remote_projtcts/papr/docs/personal-rss-sync-options-2026-10-05.md:1)、[NewsNook 可借鉴机制](D:/Work/sync_remote_projtcts/papr/docs/newsnook-cloud-sync-research-2026-10-04.md:19)。
