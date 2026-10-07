# Papr 个人桌面与 Android 同步方案调研

日期：2026-10-05。需求：两端均使用 Papr，同步订阅、文件夹、文章列表/标题/链接、星标与独立的稍后读；正文各端自行获取，不同步主题、字体、AI 设置。本轮仅研究，不修改代码、部署服务或变更账号。以下依据官方文档/源码与 Papr 本地实现；在线主分支行为仍须在实际采用的发布版本验证。

## 1. FreshRSS 与 Miniflux 的关键区别

**如选现成 RSS 后端，FreshRSS 更贴合“星标和稍后读分别保存”的需求。**这属于服务端能力判断，当前 Papr 客户端仍需开发。

FreshRSS 的 GReader `edit-tag` 能添加/移除任意文章标签，和 starred 独立；`tag/list` 区分 folder 与 tag。可约定 `user/-/label/Papr.ReadLater` 映射本地 `read_later`，保留独立星标。该名称应避开文件夹名，并按标签类型识别。[FreshRSS 官方 API 源码](https://github.com/FreshRSS/FreshRSS/blob/edge/p/api/greader.php)

FreshRSS 文章响应含服务器 ID、标题、链接、发布时间、所属 feed 与标签；Papr 可以仅导入列表元数据和状态，正文继续按链接自行获取。API 响应仍可能附带摘要/内容，忽略正文并不等于服务器不会存 RSS 内容。[文章响应源码](https://github.com/FreshRSS/FreshRSS/blob/edge/app/Models/Entry.php#L1168)

Miniflux 的 GReader `edit-tag` 当前支持 read/unread、starred；其他不支持标签会报错。不能通过现有 GReader 适配把稍后读作为独立标签同步，也不宜拿 unread 代替稍后读。[Miniflux GReader 官方实现说明](https://github.com/miniflux/v2/blob/main/internal/googlereader/README.md#post-readerapi0edit-tag)

不能扩大结论为“Miniflux 完全没有 tags”：最新原生 API 的文章导入接受 tags，但公开文章更新文档仅列 title/content；`save` 是发送第三方服务，bookmark 对应 starred。没有依据认为现有 GReader 接口已具备第二个可反复设置/清除的稍后读位。[Miniflux 原生 API](https://miniflux.app/docs/api.html)

| 操作 | FreshRSS GReader | Miniflux GReader | Papr 现状 |
|---|---|---|---|
| 订阅新增 | subscription/edit subscribe | 同名动作 | 已有基本新增 |
| 订阅改名/移动 | subscription/edit edit，title/分类 | edit，t 改名、a 移动 | 桌面主要追加；core 仅为未归类源补分类 |
| 取消订阅 | unsubscribe | unsubscribe | core 墓碑被确认但未远端删除，桌面未完整同步 |
| 文件夹改名/删除 | rename-tag / disable-tag | 同名端点；至少留一个分类 | 未实现完整双向生命周期 |
| 已读/星标 | edit-tag | edit-tag | 两条同步实现已有基本支持 |
| 独立稍后读 | 自定义文章 label 可映射 | GReader 不支持该标签写入 | 尚未加入同步 |
| 文章列表 | 有标题链接及服务器 ID | 有标题链接及服务器 ID | 当前 pull 只匹配已有本地文章，不导入缺失列表 |

服务端操作依据：[FreshRSS API 文档与源码入口](https://freshrss.github.io/FreshRSS/en/developers/06_GoogleReader_API.html)、[Miniflux subscription/edit、rename-tag、disable-tag 说明](https://github.com/miniflux/v2/blob/main/internal/googlereader/README.md#post-readerapi0subscriptionedit)。FreshRSS 分类删除会将源移回默认分类；Miniflux 移至剩余分类，且其 subscription/edit 未实现 remove label。两者都不能直接视为 Papr 任意嵌套文件夹、顺序等模型的完整映射。

## 2. 当前 Papr 的客户端缺口

Papr 有两条实现，不能只改一个文件便声称桌面和 Android 一致：桌面使用 [src-tauri/src/sync.rs](src-tauri/src/sync.rs:320)，Flutter 方向使用 [papr-core GReader port](crates/papr-core/src/sync/greader.rs:190) 与 [SyncService](crates/papr-core/src/services/sync.rs:190)。

- **文章列表未导入**：桌面响应结构不读取文章 title/origin/published，仅在本地 URL 匹配成功时更新 read/starred；core 同样只产出状态变更，DB 匹配不到现有 URL 就跳过。两端 RSS 拉取时间/上游历史窗口不同，列表会不同；部署后端本身不能解决。[桌面 Item](src-tauri/src/sync.rs:194)、[桌面应用 pull](src-tauri/src/sync.rs:456)、[core Item](crates/papr-core/src/sync/greader.rs:468)、[core DB](crates/papr-core/src/db.rs:2248)
- **稍后读仅本地**：数据库已有 `read_later`，core 同步批次只筛 read/starred，pull DB 也只允许这两字段；桌面 setter 只更新列。需要把稍后读写入可靠待发队列并接标签映射。[core setter](crates/papr-core/src/db.rs:1673)、[core 出站筛选](crates/papr-core/src/db.rs:2095)、[桌面 setter](src-tauri/src/db.rs:1018)
- **文件夹/订阅不是完整镜像**：桌面拉取按文件夹名匹配，仅给无分类源补分类；core 类似。改名、已归类源移动、删除可能漂移或被重新补回。core 明确确认 Folder 与 tombstone 操作但不发对应远端请求，pull 也跳过墓碑。[桌面归类](src-tauri/src/sync.rs:398)、[core push](crates/papr-core/src/sync/greader.rs:243)、[core pull 删除](crates/papr-core/src/db.rs:2202)
- **历史覆盖有限**：桌面只拉最近 1000 条，没有跟进 continuation；core 分页最多 2 万条且为全量状态快照，不能当成永久增量游标。应独立拉取所有星标和稍后读，避免较旧保存条目漏同步。[桌面窗口](src-tauri/src/sync.rs:460)、[core 上限](crates/papr-core/src/sync/greader.rs:123)

## 3. 若采用 FreshRSS，建议的最小扩展范围

1. 让服务器的文章 ID + feed ID 成为共同列表身份，pull 后 upsert 本地文章元数据；保留必要标题、URL、日期，正文获取继续用 Papr 原路径。先定义列表同步窗口和收藏/稍后读的长期保留范围，不把“同步列表”误写成“永久保存所有历史”。
2. read/starred/read_later 分别进入出站日志；FreshRSS 上 read_later 映射保留名称的 article tag，并在 pull 恢复、取消时保持三个状态相互独立。
3. 为订阅/文件夹新增、改名、移动、删除补双向操作与远端 ID 映射；离线删除保留待确认记录，不能将服务器暂缺误作用户删除，也不能将未执行的远端删除直接确认。
4. 统一两端协议行为，复用现有 core 事务、日志与 pending 保护；验证同时编辑、请求在途修改、旧保存文章、服务器删除/清理、首次合并、账号/服务器切换。

这是基于现状的实现建议，尚未实施或运行验证。核心取舍：FreshRSS 提供持续抓取与共同文章目录，但仍需服务器维护；单纯同步两个 Papr 本地副本可用存储后端实现，列表发现与元数据上传责任则由客户端承担。GitHub 和服务器环境比较见下文。

## 4. 验证结果

本轮交叉阅读了两家官方接口资料/当前源码和 Papr 桌面/core 源码；仅静态研究。未部署 FreshRSS/Miniflux、未进行真实账号 API 测试、未修改业务代码。采用前应固定后端发布版本，进行桌面与 Android 两端端到端验证，重点证明列表导入、稍后读取消、订阅删除与文件夹移动。
## GitHub 私有仓库方案：当前需求下可行

以下是拟议设计，不是 Papr 已有功能。用户已明确电脑和 Android 都使用 Papr；同步订阅、分类文件夹、文章列表（标题、链接）、星标和独立的稍后读状态；正文由各端获取，排除主题、字体、AI 配置。

### 使用方式与同步边界

将一个专用 GitHub 私有仓库作为版本化的元数据存储，Papr 通过 HTTPS REST API 读取及更新 JSON，无需在客户端运行 Git，也不需要 GitHub Actions、npm 或自建账号服务器。GitHub Free 个人账号可创建不限数量的私有仓库，部分协作功能有限，但本方案不依赖这些功能。[GitHub 官方套餐](https://docs.github.com/en/get-started/learning-about-github/githubs-plans)

数据至少包括：
- 订阅：跨设备稳定 ID、源地址、名称、所属文件夹。
- 文件夹：稳定 ID、名称、顺序；重命名不能当作新建。
- 文章目录：所属源、稳定文章标识、标题、链接、发布时间；另一端尚无此文章时应创建元数据记录。
- 文章状态：星标、稍后读分别存储；建议已读也纳入同一模型。
- 删除标记：取消订阅、删除文件夹等需要传播，不能仅在快照中省略。

不上传 SQLite 整库、正文、图片、主题、字体或 AI 凭据。文章标识应优先结合稳定源标识及 feed GUID，缺失时按明确的 URL 规则回退，不能沿用各端自增整数。GUID/URL 变化、同一文章被多个源收录应有明确定义。

### 并发处理与 NewsNook 可借鉴点

Contents API 更新文件必须提供被替换文件的 blob SHA，并有 409 冲突响应；它提供冲突检测，不能自动完成阅读状态的业务合并。客户端应保留“上次确认的快照 + 本地待提交变更”，读远端后合并，再带 SHA 更新；冲突时重读、重新合并并有限重试。多文件写入需要考虑不完整版本，不能假定每个文件依次更新就是整体事务。[GitHub Contents API](https://docs.github.com/en/rest/repos/contents)

例如电脑给文章加星标，同时手机将它加入稍后读，合并后两个字段都应为真。不同字段可独立合并；同一字段的相反操作、删除与编辑冲突需要制定确定的规则，不能靠整份 JSON 最后覆盖，也不能简单对布尔值取 OR，否则取消星标无法传播。离线改动要持久化，远端应用要避免反复回传。

NewsNook 的本地投影、已确认 shadow、待发送队列、初次同步选择、崩溃恢复和失败退避值得借鉴。服务端 per-user revision、账号体系和 PostgreSQL 不必照搬：GitHub 提供文件版本检测，但客户端需承担更多合并工作。NewsNook V1 不同步文章目录与阅读状态，不能直接满足这里的同步范围。[已有 NewsNook 调研](docs/newsnook-cloud-sync-research-2026-10-04.md:5)

### 费用、限制与适用规模

- 服务费：使用 GitHub Free 私有仓库，无额外订阅费；存在 Papr 开发及后续适配成本。
- 认证请求通常为每账号每小时 5,000 次，两端及其他应用共用额度。生成内容的请求通常另受每分钟 80 次、每小时 500 次限制；部分端点更低，二级限制可变。应把多次阅读操作合并提交，并处理 403/429、Retry-After 和退避。[GitHub REST API 限流](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)
- Contents API 读取文件：不超过 1 MB 支持全部功能；1–100 MB 需用 raw/object 模式；超过 100 MB 不支持该接口。1 MB 不是整个仓库的容量上限。随着文章目录增长应按源或时间分片，并约束普通文章保留范围，星标和稍后读可单独长期保留。[GitHub Contents API](https://docs.github.com/en/rest/repos/contents)
- 不建议每篇文章一个文件或每次点击一个 commit。状态更新会积累 Git 历史；删除当前文件内容不等于清除历史中的阅读记录。
- 无即时推送保证。适合启动、回到前台、手动同步与本地变化后短暂延迟批量上传。Android 后台同步频率受系统调度影响。
- 私有仓库意味着账号权限控制，不意味着端到端加密。使用仅授权该仓库 Contents 读写的 fine-grained PAT，保存在系统凭据库/Android 安全存储中，不写入仓库、普通配置或日志。[权限要求](https://docs.github.com/en/rest/repos/contents)
- 未实测用户电脑与手机访问 GitHub API 的连接质量；选择前要测真实网络可达性。
- GitHub 只保存由客户端上传的文章目录，不负责抓取 RSS。两端都不开 Papr 时，不会产生新文章记录；之后源若已移除旧条目，可能无法补回。正文未同步，也就无法保证原站删除后仍可阅读全文。

### 与自建 FreshRSS 的取舍

| 维度 | GitHub 私有仓库 | 腾讯云服务器上 FreshRSS |
|---|---|---|
| 额外订阅费用 | GitHub Free 可满足轻量个人元数据同步 | 软件免费，复用现有服务器；续费、带宽仍按腾讯云套餐 |
| 日常维护 | 无需维护自己的服务进程/数据库 | 需要系统、HTTPS、应用更新和备份 |
| 文章抓取 | Papr 各端抓取后上传目录 | 服务器定时集中抓取，客户端关闭期间也可持续运行 |
| 稍后读 | 自定义独立字段，需客户端实现 | 可映射独立文章标签，需客户端实现 |
| 文章列表一致 | 需要新目录上传、下载和本地元数据导入 | 需要补齐现有 GReader 文章目录导入 |
| 首期开发 | 新增提供方及并发合并逻辑 | 可扩展现有 GReader，但桌面与 Core 两条实现仍需处理 |
| 主要边界 | API 限流、历史增长、无集中抓取 | 服务器运维、清理保留策略、客户端兼容性 |

按“只自己用、元数据和状态、免服务器维护”排序，GitHub 值得优先考虑。若更重视“两端关闭时仍持续收集文章”，FreshRSS 更合适。这是需求取舍；不能把 GitHub 服务可用等同于当前 Papr 已可直接启用。

## 腾讯云服务器适配判断

用户提供：新加坡，2 vCPU，3.6 GiB RAM（约 3.0 GiB available），59 GiB 磁盘（52 GiB 空闲），轻负载。按服务结构推断，这一配置足以作为单人 FreshRSS + SQLite 的起点，不需要先购买额外数据库；未经部署和压力测试，不能给出保证的订阅数或文章容量。FreshRSS 官方支持 SQLite。[FreshRSS 安装说明](https://freshrss.github.io/FreshRSS/en/admins/06_LinuxInstall.html)

主要问题是 CentOS Linux 7 已于 2024-06-30 结束生命周期，当前 Docker 官方 CentOS 安装支持列表为 CentOS Stream 9/10。不建议为新服务基于旧系统固定旧运行时。[CentOS 官方](https://www.centos.org/centos-linux/)、[Docker 官方 CentOS 要求](https://docs.docker.com/engine/install/centos/)

若未来选择自建，建议先确认服务器数据并备份，再考虑受支持的系统，例如 Ubuntu 24.04 LTS（标准安全维护到 2029 年 5 月，Docker 支持）。使用 FreshRSS、SQLite、定时刷新与 HTTPS 的简单组合，无需 npm。重装系统属于单独的实施操作，本轮没有执行。[Ubuntu 生命周期](https://ubuntu.com/about/release-cycle)、[Docker Ubuntu 要求](https://docs.docker.com/engine/install/ubuntu/)

提供的 eth0 地址 10.3.0.8 是私网地址，不能据此确认设备能从公网接入；未来部署前还要核对腾讯云公网 IP/映射、带宽、安全组及 HTTPS 入口。本轮未连接服务器，也未检查账单，不能据已有硬件推定续费或出网免费。

## 本轮范围与后续验证

本轮只静态阅读本地源码与官方文档，新增调研文档；没有修改业务代码、安装软件、重装系统、创建 GitHub 仓库或访问凭据，没有执行构建、设备测试及服务部署。

未来方案验收至少覆盖：一端新增订阅/目录在另一端出现；文章标题和链接可导入；星标与稍后读互不覆盖且可取消；文件夹改名/移动/取消订阅可传播；两端离线后并发修改可合并；请求失败/进程退出后待上传状态仍可恢复。正文单独获取失败不应阻断目录及状态同步。
