// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get githubScheduleUnsaved => '修改尚未生效，点击保存后应用。';

  @override
  String get githubScheduleSaved => '同步设置已保存';

  @override
  String get githubScheduleSaveFailed => '同步设置未保存成功，请重试。';

  @override
  String get githubAutomatic => '自动同步';

  @override
  String get githubUploadDelay => '本地改动上传延迟';

  @override
  String get githubCloudInterval => '云端更新检查周期';

  @override
  String get githubScheduleHint => '仅作用于本机，两次发布至少间隔 60 秒。';

  @override
  String get githubManualOnly => '已关闭自动同步，需要时点击立即同步。';

  @override
  String get errorGithubInvalidSchedule => '同步时间设置无效。';

  @override
  String get githubBackgroundInterval => '后台同步间隔';

  @override
  String get githubBackgroundHint => 'Android 可能因省电策略延迟后台执行。';

  @override
  String githubSeconds(int count) {
    return '$count 秒';
  }

  @override
  String get githubScope =>
      '同步订阅、文件夹、文章标题和链接，以及已读、星标和稍后读状态。普通文章保留 90 天，星标或稍后读长期保留。正文及应用配置留在本地。';

  @override
  String get githubLoading => '加载中…';

  @override
  String githubPending(int count) {
    return '待上传操作：$count';
  }

  @override
  String githubLastSuccess(String time) {
    return '上次同步：$time';
  }

  @override
  String get githubNever => '尚未同步';

  @override
  String githubRejected(int count) {
    return '需要关注的操作：$count';
  }

  @override
  String get githubUncertain => '上次发布结果尚不确定。请重试，客户端会先检查云端是否已接收。';

  @override
  String githubRetryAt(String time) {
    return '可重试时间：$time';
  }

  @override
  String get githubCancel => '取消';

  @override
  String get githubToken => 'GitHub 精细权限 Token';

  @override
  String get githubUpdateToken => '更新 Token';

  @override
  String get githubCredentialUpdated => 'Token 已更新';

  @override
  String get githubCompleted => '同步完成';

  @override
  String get githubOwner => '仓库所有者';

  @override
  String get githubRepo => '私有仓库名称';

  @override
  String get githubBranch => '分支（留空使用默认分支）';

  @override
  String get githubOtherBackend => '请先断开现有同步服务，再连接 GitHub。';

  @override
  String get githubSetup =>
      '使用已初始化的私有仓库，并创建仅可访问该仓库、具有 Contents 读写权限的精细权限 Token。';

  @override
  String get githubPreview => '预览连接';

  @override
  String githubPreviewCounts(
      int localFeeds, int remoteFeeds, int localArticles, int remoteArticles) {
    return '订阅：本地 $localFeeds，云端 $remoteFeeds。文章：本地 $localArticles，云端 $remoteArticles。';
  }

  @override
  String githubPreviewExcluded(int feeds, int articles, int warnings) {
    return '不上传：$feeds 个来源、$articles 篇文章。提示：$warnings 项。';
  }

  @override
  String get githubPreviewHint => '首次同步合并订阅与保存状态；已有云端订阅设置优先。文章正文不会上传。';

  @override
  String get githubConfirm => '确认连接';

  @override
  String get githubErrorUnknown => 'GitHub 同步失败。本地操作仍在队列中，请检查连接后重试。';

  @override
  String get errorGithubAuthenticationFailed => 'Token 无效或已过期，请更新 Token。';

  @override
  String get errorGithubPermissionDenied => 'Token 需要该私有仓库的 Contents 读写权限。';

  @override
  String get errorGithubRateLimited => '已达到 GitHub 限流，请等待显示的可重试时间。';

  @override
  String get errorGithubNetwork => '无法连接 GitHub，本地操作仍在队列中。';

  @override
  String get errorGithubRepositoryUnavailable =>
      '仓库或分支不可访问，请检查所有者、名称及 Token 权限。';

  @override
  String get errorGithubPrivateRepositoryRequired => '请选择私有仓库。';

  @override
  String get errorGithubOtherBackendConnected => '请先断开已有同步服务。';

  @override
  String get errorGithubPreviewChanged => '云端数据已变化，请重新预览后再连接。';

  @override
  String get errorGithubSyncBusy => '同步正在进行。';

  @override
  String get errorGithubHistoryRewritten => '云端历史发生异常变化，同步已停止；请检查仓库后重新连接。';

  @override
  String get errorGithubDatasetChanged => '该仓库中的同步数据集已变化，请检查后重新连接。';

  @override
  String get errorGithubRestoredDatabase => '检测到恢复的数据库，请核对本地数据后重新连接。';

  @override
  String get errorGithubDatabaseCloneDetected => '数据库来自另一个安装实例，请在当前设备重新连接。';

  @override
  String get errorGithubInstallationMissing => '安全存储中的设备身份缺失，请重新连接。';

  @override
  String get errorGithubCapacityExceeded => '同步数据超过大小限制。';

  @override
  String get errorGithubSyncCancelled => '同步已取消，待同步操作仍保留。';

  @override
  String get errorGithubConcurrentRetryLimit => '云端连续变化，请稍后重试。';

  @override
  String get errorGithubWriteRejected => 'GitHub 拒绝写入，请检查分支保护和 Token 权限。';

  @override
  String get appTitle => 'Papr';

  @override
  String get navArticles => '文章';

  @override
  String get navSubscriptions => '订阅';

  @override
  String get navSaved => '已保存';

  @override
  String get navSettings => '设置';

  @override
  String get articlesTitle => '文章';

  @override
  String get savedTitle => '已保存';

  @override
  String get subscriptionsTitle => '订阅';

  @override
  String get settingsTitle => '设置';

  @override
  String get themeLabel => '主题';

  @override
  String get themeSystem => '跟随系统';

  @override
  String get themeLight => '浅色';

  @override
  String get themeDark => '深色';

  @override
  String get languageLabel => '语言';

  @override
  String get languageEnglish => '英语';

  @override
  String get languageChinese => '简体中文';

  @override
  String get languageJapanese => '日语';

  @override
  String get refreshInterval => '刷新间隔';

  @override
  String get autoRefresh => '后台刷新';

  @override
  String get autoRefreshDescription => '在 Android 允许后台运行时刷新已到期订阅';

  @override
  String get newArticleNotifications => '新文章通知';

  @override
  String get newArticleNotificationsDescription => '后台刷新完成后显示一条汇总通知';

  @override
  String newArticleNotificationBody(int count) {
    return '$count 篇新文章';
  }

  @override
  String get notificationQuietHours => '夜间免打扰';

  @override
  String get notificationQuietHoursDescription => '22:00 至 08:00 之间不显示通知';

  @override
  String get notificationPermissionDenied => '未授予通知权限。';

  @override
  String get resetPreferences => '重置偏好设置';

  @override
  String get resetPreferencesTitle => '重置偏好设置？';

  @override
  String get resetPreferencesMessage => '将主题、语言、阅读和后台设置恢复默认值？订阅和账号会保留。';

  @override
  String get resetPreferencesDone => '偏好设置已重置。';

  @override
  String get resetPreferencesFailed => '无法重置偏好设置，请重试。';

  @override
  String get clearAllData => '清除全部数据';

  @override
  String get clearAllDataTitle => '清除全部应用数据？';

  @override
  String get clearAllDataMessage => '删除本地订阅、文章、设置、缓存文件和已存凭据？应用将关闭，且无法撤销。';

  @override
  String get clearAllDataFailed => '无法清除应用数据，请在 Android 设置中重试。';

  @override
  String minutes(int count) {
    return '$count 分钟';
  }

  @override
  String get articleTitle => '文章';

  @override
  String byAuthor(String author) {
    return '作者：$author';
  }

  @override
  String get noContent => '暂无正文';

  @override
  String get noArticles => '暂无文章';

  @override
  String get noSavedArticles => '暂无已保存文章';

  @override
  String get noFeeds => '暂无订阅，点击 + 添加。';

  @override
  String get selectSubscription => '选择一个订阅以查看文章';

  @override
  String errorMessage(String message) {
    return '错误：$message';
  }

  @override
  String get refreshTooltip => '刷新';

  @override
  String refreshComplete(int count) {
    return '刷新完成：新增 $count 篇文章。';
  }

  @override
  String refreshPartial(int newCount, int errorCount) {
    return '刷新完成：新增 $newCount 篇，$errorCount 个订阅失败。';
  }

  @override
  String refreshFailed(String message) {
    return '刷新失败：$message';
  }

  @override
  String get addFeed => '添加订阅';

  @override
  String get feedUrl => 'Feed 或网页地址';

  @override
  String get feedUrlHint => 'https://example.com/feed.xml';

  @override
  String get cancel => '取消';

  @override
  String get add => '添加';

  @override
  String get feedUrlRequired => '请输入 Feed 或网页地址';

  @override
  String get feedTitle => '订阅名称';

  @override
  String appearanceSaveFailed(String message) {
    return '无法保存外观设置：$message';
  }

  @override
  String get unknownError => '发生错误';

  @override
  String get manageFolders => '管理文件夹';

  @override
  String get folderName => '文件夹名称';

  @override
  String get createFolder => '新建文件夹';

  @override
  String get rename => '重命名';

  @override
  String get delete => '删除';

  @override
  String get deleteFolderTitle => '删除文件夹？';

  @override
  String get deleteFolderMessage => '该文件夹内的订阅将移至“未分类”。';

  @override
  String get uncategorized => '未分类';

  @override
  String get feedActions => '订阅操作';

  @override
  String get renameFeed => '重命名订阅';

  @override
  String get moveToFolder => '移动到文件夹';

  @override
  String get refreshIntervalTitle => '订阅刷新间隔';

  @override
  String get useGlobalInterval => '使用全局间隔';

  @override
  String get refreshOff => '不自动刷新';

  @override
  String get deleteFeedTitle => '删除订阅？';

  @override
  String deleteFeedMessage(String title) {
    return '删除“$title”及其已下载文章？';
  }

  @override
  String refreshOneComplete(String title, int count) {
    return '已更新“$title”：新增 $count 篇文章。';
  }

  @override
  String get directory => '订阅目录';

  @override
  String get search => '搜索';

  @override
  String get directoryHint => '搜索刊物或主题';

  @override
  String get noResults => '没有匹配的订阅';

  @override
  String get opml => 'OPML';

  @override
  String get importOpml => '导入 OPML 文本';

  @override
  String get exportOpml => '导出 OPML';

  @override
  String get opmlRequired => '所选 OPML 文件为空。';

  @override
  String opmlImported(int imported, int failed) {
    return '已导入 $imported 个订阅，$failed 个失败。';
  }

  @override
  String get opmlExported => 'OPML 文件已保存。';

  @override
  String get close => '关闭';

  @override
  String get errorEmptyFolderName => '文件夹名称不能为空。';

  @override
  String get errorFolderNameExists => '已存在同名文件夹。';

  @override
  String get errorInvalidFolderOrder => '文件夹顺序已在其他位置变更，请刷新后重试。';

  @override
  String get errorEmptyFeedTitle => '订阅名称不能为空。';

  @override
  String get errorFeedAlreadyExists => '该订阅已存在。';

  @override
  String get errorEmptyFeedUrl => '订阅地址不能为空。';

  @override
  String get errorFeedNotFound => '该地址未发现 Feed。';

  @override
  String get errorInvalidFeedUrl => '请输入有效的 Feed 或网页地址。';

  @override
  String get errorNetwork => '网络请求失败，请检查连接后重试。';

  @override
  String get errorParse => '无法读取订阅数据。';

  @override
  String get errorAiNoVisibleOutput => 'AI 未返回可显示的内容，请重新生成。';

  @override
  String get retry => '重试';

  @override
  String get articleActions => '文章操作';

  @override
  String get readerActions => '阅读器操作';

  @override
  String get markRead => '标为已读';

  @override
  String get markUnread => '标为未读';

  @override
  String get star => '收藏';

  @override
  String get unstar => '取消收藏';

  @override
  String get readLater => '稍后读';

  @override
  String get removeReadLater => '移出稍后读';

  @override
  String get searchArticles => '搜索标题和全文';

  @override
  String get listOptions => '列表选项';

  @override
  String get hideRead => '隐藏已读文章';

  @override
  String get oldestFirst => '最早优先';

  @override
  String get markAllRead => '将当前视图全部标为已读';

  @override
  String markedAllRead(int count) {
    return '已将 $count 篇文章标为已读。';
  }

  @override
  String get smartViews => '智能视图';

  @override
  String get allArticles => '全部文章';

  @override
  String get unreadArticles => '未读';

  @override
  String get starredArticles => '收藏';

  @override
  String get readLaterArticles => '稍后读';

  @override
  String get folders => '文件夹';

  @override
  String get tags => '标签';

  @override
  String get extractFulltext => '提取全文';

  @override
  String get reextractFulltext => '重新提取全文';

  @override
  String get fulltextExtracted => '全文提取完成。';

  @override
  String get openInBrowser => '在浏览器中打开';

  @override
  String get shareArticle => '分享';

  @override
  String get platformActionFailed => '没有可执行此操作的应用。';

  @override
  String get readingSettings => '阅读设置';

  @override
  String get readerFont => '阅读字体';

  @override
  String get fontSystem => '系统';

  @override
  String get fontSerif => '衬线';

  @override
  String get fontSans => '无衬线';

  @override
  String get fontSize => '字号';

  @override
  String get lineHeight => '行距';

  @override
  String get readingWidth => '阅读宽度';

  @override
  String get showReadingTime => '显示阅读时长';

  @override
  String get autoExtractFulltext => '自动提取全文';

  @override
  String get autoExtractFulltextDescription => '没有全文缓存时抓取原始网页。';

  @override
  String readMinutes(int count) {
    return '预计阅读 $count 分钟';
  }

  @override
  String get attachments => '附件';

  @override
  String get attachment => '附件';

  @override
  String get audioEpisodes => '音频节目';

  @override
  String get audioEpisode => 'Podcast 音频';

  @override
  String get playAudio => '播放';

  @override
  String get pauseAudio => '暂停';

  @override
  String get nowPlaying => '正在播放';

  @override
  String get noActivePlayback => '当前没有正在播放的音频。';

  @override
  String get rewind15 => '后退 15 秒';

  @override
  String get forward30 => '前进 30 秒';

  @override
  String get playbackSpeed => '播放速度';

  @override
  String get stopPlayback => '停止播放';

  @override
  String get errorInvalidPlaybackUrl => '音频地址无效。';

  @override
  String get errorPlaybackNetwork => '音频加载失败，请检查网络后重试。';

  @override
  String get errorPlaybackUnavailable => '当前设备无法播放此音频。';

  @override
  String get errorPlaybackFailed => '音频播放失败。';

  @override
  String get imageUnavailable => '图片无法加载';

  @override
  String get errorArticleNotFound => '文章已不存在。';

  @override
  String get errorArticleUrlMissing => '此文章没有原文链接。';

  @override
  String get errorNoExtractableContent => '未找到可阅读的全文。';

  @override
  String get errorInvalidReadingSettings => '阅读设置超出支持范围。';

  @override
  String get addHighlight => '添加高亮';

  @override
  String get highlights => '高亮';

  @override
  String get noHighlights => '暂时没有高亮。选择文章文字即可添加。';

  @override
  String get highlightColor => '颜色';

  @override
  String get highlightNote => '笔记';

  @override
  String get highlightCreated => '已添加高亮。';

  @override
  String get highlightSelectionUnavailable => '无法在本文中定位该选区，请重新选择。';

  @override
  String get highlightUnableToLocate => '部分高亮无法在当前正文中定位。';

  @override
  String get save => '保存';

  @override
  String get errorEmptyTagName => '标签名称不能为空。';

  @override
  String get errorTagNameExists => '已存在同名标签。';

  @override
  String get errorInvalidTagColor => '请选择支持的标签颜色。';

  @override
  String get errorInvalidTagOrder => '标签排序已在其他位置变更，请刷新后重试。';

  @override
  String get errorRuleNotFound => '该规则已不存在。';

  @override
  String get errorEmptyRuleName => '规则名称不能为空。';

  @override
  String get errorEmptyRuleQuery => '请至少输入一个规则关键词。';

  @override
  String get errorInvalidRuleField => '请选择支持的规则字段。';

  @override
  String get errorInvalidRuleAction => '请选择支持的规则动作。';

  @override
  String get errorHighlightNotFound => '该高亮已不存在。';

  @override
  String get errorEmptyHighlight => '请选择文字以创建高亮。';

  @override
  String get errorInvalidHighlightOffset => '该高亮位置无效。';

  @override
  String get errorInvalidHighlightColor => '请选择支持的高亮颜色。';

  @override
  String get manageTags => '管理标签';

  @override
  String get manageRules => '管理规则';

  @override
  String get tagName => '标签名称';

  @override
  String get createTag => '创建标签';

  @override
  String get newRule => '新建规则';

  @override
  String get ruleName => '规则名称';

  @override
  String get ruleKeywords => '关键词（以逗号分隔）';

  @override
  String get ruleField => '匹配字段';

  @override
  String get ruleAction => '动作';

  @override
  String get ruleEnabled => '已启用';

  @override
  String get rulePreview => '预览';

  @override
  String rulePreviewResult(int count) {
    return '匹配 $count 篇文章';
  }

  @override
  String get applyRule => '应用到已有文章';

  @override
  String get applySkipRuleTitle => '应用跳过规则？';

  @override
  String get applySkipRuleMessage => '将删除匹配的未保存文章；收藏、稍后读和含高亮的文章会保留。';

  @override
  String applyRuleComplete(Object count) {
    return '已应用到 $count 篇文章。';
  }

  @override
  String get titleField => '标题';

  @override
  String get authorField => '作者';

  @override
  String get contentField => '正文';

  @override
  String get anyField => '任意字段';

  @override
  String get skipAction => '跳过';

  @override
  String get readAction => '标为已读';

  @override
  String get starAction => '收藏';

  @override
  String get editTags => '编辑标签';

  @override
  String get newTagHint => '创建新标签';

  @override
  String get globalHighlights => '全部高亮';

  @override
  String get noGlobalHighlights => '暂时没有高亮。';

  @override
  String get allFeeds => '所有订阅';

  @override
  String get aiProfiles => 'AI 配置';

  @override
  String get noAiProfiles => '暂无 AI 配置。添加后即可使用摘要。';

  @override
  String get addAiProfile => '添加 AI 配置';

  @override
  String get editAiProfile => '编辑 AI 配置';

  @override
  String get deleteAiProfileTitle => '删除 AI 配置？';

  @override
  String get profileName => '配置名称';

  @override
  String get aiProtocol => '协议';

  @override
  String get aiModel => '模型';

  @override
  String get aiBaseUrl => 'Base URL';

  @override
  String get aiAuth => '认证方式';

  @override
  String get aiApiKey => 'API Key';

  @override
  String get aiApiKeyKeep => '留空可保留当前密钥。';

  @override
  String get openaiCompatible => 'OpenAI 兼容';

  @override
  String get anthropic => 'Anthropic Messages';

  @override
  String get bearerAuth => 'Bearer Token';

  @override
  String get xApiKeyAuth => 'x-api-key';

  @override
  String get noAuth => '无需认证';

  @override
  String get useForSummary => '用于摘要';

  @override
  String get testAiConnection => '测试连接';

  @override
  String get aiConnectionSucceeded => 'AI 服务连接正常。';

  @override
  String get errorInvalidAiProfile => '请完整填写 AI 配置的必填项。';

  @override
  String get errorNoAiCredential => '请输入此配置的 API Key。';

  @override
  String get errorAiCredentialStore => '无法安全访问 API Key。';

  @override
  String get errorAiAuth => 'AI 服务拒绝了当前认证信息。';

  @override
  String get aiSummary => 'AI 摘要';

  @override
  String get summaryNoProfile => '生成摘要前，请先配置并启用一个 AI 配置。';

  @override
  String get configureAiProfile => '配置 AI';

  @override
  String get summaryTemplate => '摘要模板';

  @override
  String get summaryTemplateClassic => '经典摘要';

  @override
  String get summaryTemplateNews => '新闻 5W1H';

  @override
  String get summaryTemplateDecision => '阅读决策';

  @override
  String get summaryTemplateFunnel => '渐进漏斗';

  @override
  String get summaryTemplateArgument => '观点拆解';

  @override
  String get summaryTemplateMinimal => '一句话';

  @override
  String get summaryTemplateLegacy => '旧版';

  @override
  String summaryCacheInfo(String template, String language) {
    return '缓存模板：$template · 语言：$language';
  }

  @override
  String get regenerate => '重新生成';

  @override
  String get stop => '停止';

  @override
  String get noSummaryYet => '暂无完整摘要。';

  @override
  String get askAboutSummary => '基于此摘要追问';

  @override
  String get summaryQuestionHint => '仅根据上方摘要提问';

  @override
  String get send => '发送';

  @override
  String get aiTranslation => 'AI 翻译';

  @override
  String get translationNoProfile => '翻译前，请先配置并启用一个 AI 配置。';

  @override
  String get targetLanguage => '目标语言';

  @override
  String get translateArticle => '翻译';

  @override
  String translationCacheInfo(String language) {
    return '缓存翻译：$language';
  }

  @override
  String get translationPreparing => '正在准备翻译…';

  @override
  String translationProgress(int completed, int total) {
    return '已翻译 $completed/$total 个段落';
  }

  @override
  String get noTranslationYet => '暂无完整翻译。';

  @override
  String get syncTitle => '阅读器同步';

  @override
  String get syncProvider => '服务';

  @override
  String get syncServerUrl => '服务器地址';

  @override
  String get syncUsername => '用户名';

  @override
  String get syncCredential => 'GReader 密码';

  @override
  String get syncFieldRequired => '请填写此项。';

  @override
  String get syncConnect => '连接';

  @override
  String get syncConnected => '已连接';

  @override
  String get syncNotConnected => '尚未连接阅读器服务。';

  @override
  String get syncTestSucceeded => '连接测试成功。';

  @override
  String get syncNow => '立即同步';

  @override
  String get syncCompleted => '同步完成。';

  @override
  String get syncDisconnect => '断开连接';

  @override
  String get syncDisconnectConfirm => '断开此阅读器服务并删除保存的凭据？';

  @override
  String get syncReplaceConnection => '更换连接';

  @override
  String get syncNeverCompleted => '尚未同步';

  @override
  String syncLastSuccess(String time) {
    return '上次同步：$time';
  }

  @override
  String get errorInvalidSyncProfile => '请检查服务地址、用户名和密码。';

  @override
  String get errorSyncCredentialMissing => '已保存的密码不可用。请重新连接后再同步。';

  @override
  String get errorSyncNotConnected => '请先连接阅读器服务。';

  @override
  String get errorSyncAuthFailed => '服务拒绝了认证信息，请检查 GReader 密码。';

  @override
  String get errorSyncUnavailable => '阅读器服务暂时不可用，请稍后重试。';

  @override
  String get errorSyncProviderFailed => '阅读器服务未能完成请求，请重试。';

  @override
  String get errorSyncInvalidResponse => '阅读器服务返回了意外的数据。';

  @override
  String get errorSyncTooManyItems => '单次同步的条目过多，请缩减服务端历史后重试。';

  @override
  String get errorSyncCredentialStore => '无法安全访问已保存的密码。';
}
