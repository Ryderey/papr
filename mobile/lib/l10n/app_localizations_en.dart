// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

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
}
