// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get githubScheduleUnsaved => 'Changes take effect after you tap Save.';

  @override
  String get githubScheduleSaved => 'Sync settings saved';

  @override
  String get githubScheduleSaveFailed =>
      'Sync settings were not saved. Please retry.';

  @override
  String get githubAutomatic => 'Automatic sync';

  @override
  String get githubUploadDelay => 'Upload local changes after';

  @override
  String get githubCloudInterval => 'Check cloud updates every';

  @override
  String get githubScheduleHint =>
      'Device-only settings. Publications stay at least 60 seconds apart.';

  @override
  String get githubManualOnly =>
      'Automatic sync is off. Use Sync now when needed.';

  @override
  String get errorGithubInvalidSchedule => 'Invalid sync scheduling settings.';

  @override
  String get githubBackgroundInterval => 'Background sync interval';

  @override
  String get githubBackgroundHint =>
      'Android may delay background work to save battery.';

  @override
  String githubSeconds(int count) {
    return '$count seconds';
  }

  @override
  String get githubScope =>
      'Sync subscriptions, folders, article list, and reading states; bodies and settings stay local. Starred and Read Later items are kept indefinitely, the rest for 90 days.';

  @override
  String get githubLoading => 'Loading…';

  @override
  String githubPending(int count) {
    return 'Pending uploads: $count';
  }

  @override
  String githubLastSuccess(String time) {
    return 'Last sync: $time';
  }

  @override
  String get githubNever => 'Never';

  @override
  String githubRejected(int count) {
    return 'Operations requiring attention: $count';
  }

  @override
  String get githubUncertain =>
      'Last publication result is uncertain; retry checks the cloud first.';

  @override
  String githubRetryAt(String time) {
    return 'Retry after $time';
  }

  @override
  String get githubCancel => 'Cancel';

  @override
  String get githubToken => 'GitHub fine-grained token';

  @override
  String get githubUpdateToken => 'Update token';

  @override
  String get githubCredentialUpdated => 'Token updated';

  @override
  String get githubCompleted => 'Sync completed';

  @override
  String get githubOwner => 'Repository owner';

  @override
  String get githubRepo => 'Private repository name';

  @override
  String get githubBranch => 'Branch (blank uses default)';

  @override
  String get githubOtherBackend =>
      'Disconnect the existing sync provider before connecting GitHub.';

  @override
  String get githubSetup =>
      'Create an initialized private repository and a fine-grained token with Contents: read and write for this repository only.';

  @override
  String get githubPreview => 'Preview connection';

  @override
  String githubPreviewCounts(
      int localFeeds, int remoteFeeds, int localArticles, int remoteArticles) {
    return 'Subscriptions: local $localFeeds, cloud $remoteFeeds. Articles: local $localArticles, cloud $remoteArticles.';
  }

  @override
  String githubPreviewExcluded(int feeds, int articles, int warnings) {
    return 'Excluded: $feeds sources, $articles articles. Warnings: $warnings.';
  }

  @override
  String get githubPreviewHint =>
      'First sync merges subscriptions and saved states. Existing cloud subscriptions stay authoritative.';

  @override
  String get githubConfirm => 'Confirm connection';

  @override
  String get githubErrorUnknown =>
      'GitHub sync failed. Your local changes remain queued; check the connection and retry.';

  @override
  String get errorGithubAuthenticationFailed =>
      'Token expired or invalid. Update the token.';

  @override
  String get errorGithubPermissionDenied =>
      'Token needs Contents read and write for this private repository.';

  @override
  String get errorGithubRateLimited =>
      'GitHub rate limit reached. Wait until the displayed retry time.';

  @override
  String get errorGithubNetwork =>
      'Could not reach GitHub. Your changes remain queued.';

  @override
  String get errorGithubRepositoryUnavailable =>
      'Repository or branch unavailable. Check owner, repository and token access.';

  @override
  String get errorGithubPrivateRepositoryRequired =>
      'Use a private repository.';

  @override
  String get errorGithubOtherBackendConnected =>
      'Disconnect the existing sync provider first.';

  @override
  String get errorGithubPreviewChanged =>
      'Cloud changed since preview. Generate a new preview before connecting.';

  @override
  String get errorGithubSyncBusy => 'Sync is already running.';

  @override
  String get errorGithubHistoryRewritten =>
      'Cloud history changed unexpectedly. Sync is paused; reconnect after checking the repository.';

  @override
  String get errorGithubDatasetChanged =>
      'This repository contains a different sync dataset. Reconnect after checking the repository.';

  @override
  String get errorGithubRestoredDatabase =>
      'A restored database was detected. Reconnect after reviewing local data.';

  @override
  String get errorGithubDatabaseCloneDetected =>
      'This database belongs to another installation. Reconnect on this device.';

  @override
  String get errorGithubInstallationMissing =>
      'Secure installation identity is missing. Reconnect on this device.';

  @override
  String get errorGithubCapacityExceeded =>
      'Sync data exceeds the configured size limit.';

  @override
  String get errorGithubSyncCancelled =>
      'Sync cancelled. Pending changes are preserved.';

  @override
  String get errorGithubConcurrentRetryLimit =>
      'Cloud changed repeatedly. Retry later.';

  @override
  String get errorGithubWriteRejected =>
      'GitHub rejected this write. Check branch protection and token permissions.';

  @override
  String get appTitle => 'Papr';

  @override
  String get navArticles => 'Articles';

  @override
  String get navSubscriptions => 'Subscriptions';

  @override
  String get navSaved => 'Saved';

  @override
  String get navSettings => 'Settings';

  @override
  String get articlesTitle => 'Articles';

  @override
  String get savedTitle => 'Saved';

  @override
  String get subscriptionsTitle => 'Subscriptions';

  @override
  String get settingsTitle => 'Settings';

  @override
  String get themeLabel => 'Theme';

  @override
  String get themeSystem => 'System';

  @override
  String get themeLight => 'Light';

  @override
  String get themeDark => 'Dark';

  @override
  String get languageLabel => 'Language';

  @override
  String get languageEnglish => 'English';

  @override
  String get languageChinese => 'Simplified Chinese';

  @override
  String get languageJapanese => 'Japanese';

  @override
  String get refreshInterval => 'Refresh interval';

  @override
  String get autoRefresh => 'Background refresh';

  @override
  String get autoRefreshDescription =>
      'Refresh due subscriptions when Android allows background work';

  @override
  String get newArticleNotifications => 'New article notifications';

  @override
  String get newArticleNotificationsDescription =>
      'Show one summary after a background refresh';

  @override
  String newArticleNotificationBody(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count new articles',
      one: '1 new article',
    );
    return '$_temp0';
  }

  @override
  String get notificationQuietHours => 'Night quiet hours';

  @override
  String get notificationQuietHoursDescription =>
      'Do not notify between 22:00 and 08:00';

  @override
  String get notificationPermissionDenied =>
      'Notification permission was not granted.';

  @override
  String get resetPreferences => 'Reset preferences';

  @override
  String get resetPreferencesTitle => 'Reset preferences?';

  @override
  String get resetPreferencesMessage =>
      'Restore theme, language, reading and background settings to their defaults? Subscriptions and accounts will remain.';

  @override
  String get resetPreferencesDone => 'Preferences reset.';

  @override
  String get resetPreferencesFailed =>
      'Could not reset preferences. Try again.';

  @override
  String get clearAllData => 'Clear all data';

  @override
  String get clearAllDataTitle => 'Clear all app data?';

  @override
  String get clearAllDataMessage =>
      'Delete local subscriptions, articles, settings, cached files and stored credentials? The app will close. This cannot be undone.';

  @override
  String get clearAllDataFailed =>
      'Could not clear app data. Try again in Android settings.';

  @override
  String minutes(int count) {
    return '$count min';
  }

  @override
  String get articleTitle => 'Article';

  @override
  String byAuthor(String author) {
    return 'By $author';
  }

  @override
  String get noContent => 'No content';

  @override
  String get noArticles => 'No articles yet';

  @override
  String get noSavedArticles => 'No saved articles yet';

  @override
  String get noFeeds => 'No subscriptions yet. Tap + to add one.';

  @override
  String get selectSubscription => 'Select a subscription to view its articles';

  @override
  String errorMessage(String message) {
    return 'Error: $message';
  }

  @override
  String get refreshTooltip => 'Refresh';

  @override
  String refreshComplete(int count) {
    return 'Refresh complete: $count new article(s).';
  }

  @override
  String refreshPartial(int newCount, int errorCount) {
    return 'Refresh complete: $newCount new, $errorCount failed.';
  }

  @override
  String refreshFailed(String message) {
    return 'Refresh failed: $message';
  }

  @override
  String get addFeed => 'Add subscription';

  @override
  String get feedUrl => 'Feed or website URL';

  @override
  String get feedUrlHint => 'https://example.com/feed.xml';

  @override
  String get cancel => 'Cancel';

  @override
  String get add => 'Add';

  @override
  String get feedUrlRequired => 'Enter a feed or website URL';

  @override
  String get feedTitle => 'Subscription title';

  @override
  String appearanceSaveFailed(String message) {
    return 'Could not save appearance: $message';
  }

  @override
  String get unknownError => 'Something went wrong';

  @override
  String get manageFolders => 'Manage folders';

  @override
  String get folderName => 'Folder name';

  @override
  String get createFolder => 'Create folder';

  @override
  String get rename => 'Rename';

  @override
  String get delete => 'Delete';

  @override
  String get deleteFolderTitle => 'Delete folder?';

  @override
  String get deleteFolderMessage =>
      'Subscriptions in this folder will move to Uncategorized.';

  @override
  String get uncategorized => 'Uncategorized';

  @override
  String get feedActions => 'Subscription actions';

  @override
  String get renameFeed => 'Rename subscription';

  @override
  String get moveToFolder => 'Move to folder';

  @override
  String get refreshIntervalTitle => 'Subscription refresh interval';

  @override
  String get useGlobalInterval => 'Use global interval';

  @override
  String get refreshOff => 'Do not refresh automatically';

  @override
  String get deleteFeedTitle => 'Delete subscription?';

  @override
  String deleteFeedMessage(String title) {
    return 'Delete $title and its downloaded articles?';
  }

  @override
  String refreshOneComplete(String title, int count) {
    return 'Updated $title: $count new article(s).';
  }

  @override
  String get directory => 'Directory';

  @override
  String get search => 'Search';

  @override
  String get directoryHint => 'Search publications or topics';

  @override
  String get noResults => 'No matching subscriptions';

  @override
  String get opml => 'OPML';

  @override
  String get importOpml => 'Import OPML text';

  @override
  String get exportOpml => 'Export OPML';

  @override
  String get opmlRequired => 'The selected OPML file is empty.';

  @override
  String opmlImported(int imported, int failed) {
    return 'Imported $imported subscription(s); $failed failed.';
  }

  @override
  String get opmlExported => 'OPML file saved.';

  @override
  String get close => 'Close';

  @override
  String get errorEmptyFolderName => 'Folder name cannot be empty.';

  @override
  String get errorFolderNameExists => 'A folder with that name already exists.';

  @override
  String get errorInvalidFolderOrder =>
      'Folder order changed elsewhere. Reload and try again.';

  @override
  String get errorEmptyFeedTitle => 'Subscription title cannot be empty.';

  @override
  String get errorFeedAlreadyExists => 'This subscription already exists.';

  @override
  String get errorEmptyFeedUrl => 'Subscription URL cannot be empty.';

  @override
  String get errorFeedNotFound => 'No feed was found at that address.';

  @override
  String get errorInvalidFeedUrl => 'Enter a valid feed or website URL.';

  @override
  String get errorNetwork =>
      'The network request failed. Check your connection and try again.';

  @override
  String get errorParse => 'The subscription data could not be read.';

  @override
  String get errorAiNoVisibleOutput =>
      'The AI returned no visible content. Try generating again.';

  @override
  String get retry => 'Retry';

  @override
  String get articleActions => 'Article actions';

  @override
  String get readerActions => 'Reader actions';

  @override
  String get markRead => 'Mark read';

  @override
  String get markUnread => 'Mark unread';

  @override
  String get star => 'Star';

  @override
  String get unstar => 'Unstar';

  @override
  String get readLater => 'Read later';

  @override
  String get removeReadLater => 'Remove from read later';

  @override
  String get searchArticles => 'Search title and full text';

  @override
  String get listOptions => 'List options';

  @override
  String get hideRead => 'Hide read articles';

  @override
  String get oldestFirst => 'Oldest first';

  @override
  String get markAllRead => 'Mark current view read';

  @override
  String markedAllRead(int count) {
    return 'Marked $count article(s) read.';
  }

  @override
  String get smartViews => 'Smart views';

  @override
  String get allArticles => 'All articles';

  @override
  String get unreadArticles => 'Unread';

  @override
  String get starredArticles => 'Starred';

  @override
  String get readLaterArticles => 'Read later';

  @override
  String get folders => 'Folders';

  @override
  String get tags => 'Tags';

  @override
  String get extractFulltext => 'Extract full text';

  @override
  String get reextractFulltext => 'Extract again';

  @override
  String get fulltextExtracted => 'Full text extracted.';

  @override
  String get openInBrowser => 'Open in browser';

  @override
  String get shareArticle => 'Share';

  @override
  String get platformActionFailed => 'No app is available for that action.';

  @override
  String get readingSettings => 'Reading settings';

  @override
  String get readerFont => 'Reader font';

  @override
  String get fontSystem => 'System';

  @override
  String get fontSerif => 'Serif';

  @override
  String get fontSans => 'Sans';

  @override
  String get fontSize => 'Font size';

  @override
  String get lineHeight => 'Line spacing';

  @override
  String get readingWidth => 'Reading width';

  @override
  String get showReadingTime => 'Show reading time';

  @override
  String get autoExtractFulltext => 'Automatically extract full text';

  @override
  String get autoExtractFulltextDescription =>
      'Fetch the source page when no extracted copy is cached.';

  @override
  String readMinutes(int count) {
    return '$count min read';
  }

  @override
  String get attachments => 'Attachments';

  @override
  String get attachment => 'Attachment';

  @override
  String get audioEpisodes => 'Audio episodes';

  @override
  String get audioEpisode => 'Podcast audio';

  @override
  String get playAudio => 'Play';

  @override
  String get pauseAudio => 'Pause';

  @override
  String get nowPlaying => 'Now playing';

  @override
  String get noActivePlayback => 'No audio is playing.';

  @override
  String get rewind15 => 'Rewind 15 seconds';

  @override
  String get forward30 => 'Forward 30 seconds';

  @override
  String get playbackSpeed => 'Playback speed';

  @override
  String get stopPlayback => 'Stop playback';

  @override
  String get errorInvalidPlaybackUrl => 'The audio address is invalid.';

  @override
  String get errorPlaybackNetwork =>
      'The audio could not be loaded. Check your network and try again.';

  @override
  String get errorPlaybackUnavailable => 'This device cannot play the audio.';

  @override
  String get errorPlaybackFailed => 'Audio playback failed.';

  @override
  String get imageUnavailable => 'Image unavailable';

  @override
  String get errorArticleNotFound => 'The article no longer exists.';

  @override
  String get errorArticleUrlMissing => 'This article has no source URL.';

  @override
  String get errorNoExtractableContent => 'No readable full text was found.';

  @override
  String get errorInvalidReadingSettings =>
      'The reading setting is outside the supported range.';

  @override
  String get addHighlight => 'Add highlight';

  @override
  String get highlights => 'Highlights';

  @override
  String get noHighlights =>
      'No highlights yet. Select text in the article to add one.';

  @override
  String get highlightColor => 'Color';

  @override
  String get highlightNote => 'Note';

  @override
  String get highlightCreated => 'Highlight added.';

  @override
  String get highlightSelectionUnavailable =>
      'That selection cannot be anchored in this article. Try selecting it again.';

  @override
  String get highlightUnableToLocate =>
      'Some highlights could not be located in the current article text.';

  @override
  String get save => 'Save';

  @override
  String get errorEmptyTagName => 'Tag name cannot be empty.';

  @override
  String get errorTagNameExists => 'A tag with that name already exists.';

  @override
  String get errorInvalidTagColor => 'Choose a supported tag color.';

  @override
  String get errorInvalidTagOrder =>
      'Tag order changed elsewhere. Reload and try again.';

  @override
  String get errorRuleNotFound => 'This rule no longer exists.';

  @override
  String get errorEmptyRuleName => 'Rule name cannot be empty.';

  @override
  String get errorEmptyRuleQuery => 'Enter at least one rule keyword.';

  @override
  String get errorInvalidRuleField => 'Choose a supported rule field.';

  @override
  String get errorInvalidRuleAction => 'Choose a supported rule action.';

  @override
  String get errorHighlightNotFound => 'This highlight no longer exists.';

  @override
  String get errorEmptyHighlight => 'Select some text to create a highlight.';

  @override
  String get errorInvalidHighlightOffset =>
      'That highlight position is invalid.';

  @override
  String get errorInvalidHighlightColor =>
      'Choose a supported highlight color.';

  @override
  String get manageTags => 'Manage tags';

  @override
  String get manageRules => 'Manage rules';

  @override
  String get tagName => 'Tag name';

  @override
  String get createTag => 'Create tag';

  @override
  String get newRule => 'New rule';

  @override
  String get ruleName => 'Rule name';

  @override
  String get ruleKeywords => 'Keywords (comma separated)';

  @override
  String get ruleField => 'Match field';

  @override
  String get ruleAction => 'Action';

  @override
  String get ruleEnabled => 'Enabled';

  @override
  String get rulePreview => 'Preview';

  @override
  String rulePreviewResult(int count) {
    return 'Matches $count article(s)';
  }

  @override
  String get applyRule => 'Apply to existing articles';

  @override
  String get applySkipRuleTitle => 'Apply skip rule?';

  @override
  String get applySkipRuleMessage =>
      'Matching unsaved articles will be removed. Starred, read-later, and highlighted articles are kept.';

  @override
  String applyRuleComplete(Object count) {
    return 'Applied to $count article(s).';
  }

  @override
  String get titleField => 'Title';

  @override
  String get authorField => 'Author';

  @override
  String get contentField => 'Content';

  @override
  String get anyField => 'Any field';

  @override
  String get skipAction => 'Skip';

  @override
  String get readAction => 'Mark read';

  @override
  String get starAction => 'Star';

  @override
  String get editTags => 'Edit tags';

  @override
  String get newTagHint => 'Create a new tag';

  @override
  String get globalHighlights => 'All highlights';

  @override
  String get noGlobalHighlights => 'No highlights yet.';

  @override
  String get allFeeds => 'All subscriptions';

  @override
  String get aiProfiles => 'AI profiles';

  @override
  String get noAiProfiles => 'No AI profile yet. Add one to use summaries.';

  @override
  String get addAiProfile => 'Add AI profile';

  @override
  String get editAiProfile => 'Edit AI profile';

  @override
  String get deleteAiProfileTitle => 'Delete AI profile?';

  @override
  String get profileName => 'Profile name';

  @override
  String get aiProtocol => 'Protocol';

  @override
  String get aiModel => 'Model';

  @override
  String get aiBaseUrl => 'Base URL';

  @override
  String get aiAuth => 'Authentication';

  @override
  String get aiApiKey => 'API key';

  @override
  String get aiApiKeyKeep => 'Leave empty to keep the current key.';

  @override
  String get openaiCompatible => 'OpenAI compatible';

  @override
  String get anthropic => 'Anthropic Messages';

  @override
  String get bearerAuth => 'Bearer token';

  @override
  String get xApiKeyAuth => 'x-api-key';

  @override
  String get noAuth => 'No authentication';

  @override
  String get useForSummary => 'Use for summaries';

  @override
  String get testAiConnection => 'Test connection';

  @override
  String get aiConnectionSucceeded => 'AI service connection succeeded.';

  @override
  String get errorInvalidAiProfile =>
      'Complete the required AI profile fields.';

  @override
  String get errorNoAiCredential => 'Enter an API key for this profile.';

  @override
  String get errorAiCredentialStore =>
      'The API key could not be accessed securely.';

  @override
  String get errorAiAuth => 'The AI service rejected the current credentials.';

  @override
  String get aiSummary => 'AI summary';

  @override
  String get summaryNoProfile =>
      'Configure an enabled AI profile before generating a summary.';

  @override
  String get configureAiProfile => 'Configure AI profile';

  @override
  String get summaryTemplate => 'Summary template';

  @override
  String get summaryTemplateClassic => 'Classic';

  @override
  String get summaryTemplateNews => '5W1H news';

  @override
  String get summaryTemplateDecision => 'Reading decision';

  @override
  String get summaryTemplateFunnel => 'Progressive funnel';

  @override
  String get summaryTemplateArgument => 'Argument analysis';

  @override
  String get summaryTemplateMinimal => 'One sentence';

  @override
  String get summaryTemplateLegacy => 'Legacy';

  @override
  String summaryCacheInfo(String template, String language) {
    return 'Cached template: $template · Language: $language';
  }

  @override
  String get regenerate => 'Regenerate';

  @override
  String get stop => 'Stop';

  @override
  String get noSummaryYet => 'No complete summary yet.';

  @override
  String get askAboutSummary => 'Ask about this summary';

  @override
  String get summaryQuestionHint => 'Ask using only the summary above';

  @override
  String get send => 'Send';

  @override
  String get aiTranslation => 'AI translation';

  @override
  String get translationNoProfile =>
      'Configure and enable an AI profile before translating.';

  @override
  String get targetLanguage => 'Target language';

  @override
  String get translateArticle => 'Translate';

  @override
  String translationCacheInfo(String language) {
    return 'Cached translation: $language';
  }

  @override
  String get translationPreparing => 'Preparing translation…';

  @override
  String translationProgress(int completed, int total) {
    return 'Translated $completed of $total sections';
  }

  @override
  String get noTranslationYet => 'No complete translation yet.';

  @override
  String get syncTitle => 'Reader sync';

  @override
  String get syncProvider => 'Service';

  @override
  String get syncServerUrl => 'Server URL';

  @override
  String get syncUsername => 'Username';

  @override
  String get syncCredential => 'GReader password';

  @override
  String get syncFieldRequired => 'This field is required.';

  @override
  String get syncConnect => 'Connect';

  @override
  String get syncConnected => 'Connected';

  @override
  String get syncNotConnected => 'No reader service connected.';

  @override
  String get syncTestSucceeded => 'Connection succeeded.';

  @override
  String get syncNow => 'Sync now';

  @override
  String get syncCompleted => 'Sync complete.';

  @override
  String get syncDisconnect => 'Disconnect';

  @override
  String get syncDisconnectConfirm =>
      'Disconnect this reader service and remove its saved credential?';

  @override
  String get syncReplaceConnection => 'Replace connection';

  @override
  String get syncNeverCompleted => 'Never synced';

  @override
  String syncLastSuccess(String time) {
    return 'Last sync: $time';
  }

  @override
  String get errorInvalidSyncProfile =>
      'Check the service URL, username, and password.';

  @override
  String get errorSyncCredentialMissing =>
      'The saved password is unavailable. Reconnect to continue syncing.';

  @override
  String get errorSyncNotConnected => 'Connect a reader service first.';

  @override
  String get errorSyncAuthFailed =>
      'The service rejected these credentials. Check the GReader password.';

  @override
  String get errorSyncUnavailable =>
      'The reader service is unavailable. Try again later.';

  @override
  String get errorSyncProviderFailed =>
      'The reader service could not complete the request. Try again.';

  @override
  String get errorSyncInvalidResponse =>
      'The reader service returned an unexpected response.';

  @override
  String get errorSyncTooManyItems =>
      'Too many items to sync in one pass. Reduce the service history and retry.';

  @override
  String get errorSyncCredentialStore =>
      'The saved password could not be accessed securely.';
}
