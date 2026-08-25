import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_ja.dart';
import 'app_localizations_zh.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
      : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
    delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalCupertinoLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
  ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('ja'),
    Locale('zh')
  ];

  /// No description provided for @appTitle.
  ///
  /// In en, this message translates to:
  /// **'Papr'**
  String get appTitle;

  /// No description provided for @navArticles.
  ///
  /// In en, this message translates to:
  /// **'Articles'**
  String get navArticles;

  /// No description provided for @navSubscriptions.
  ///
  /// In en, this message translates to:
  /// **'Subscriptions'**
  String get navSubscriptions;

  /// No description provided for @navSaved.
  ///
  /// In en, this message translates to:
  /// **'Saved'**
  String get navSaved;

  /// No description provided for @navSettings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get navSettings;

  /// No description provided for @articlesTitle.
  ///
  /// In en, this message translates to:
  /// **'Articles'**
  String get articlesTitle;

  /// No description provided for @savedTitle.
  ///
  /// In en, this message translates to:
  /// **'Saved'**
  String get savedTitle;

  /// No description provided for @subscriptionsTitle.
  ///
  /// In en, this message translates to:
  /// **'Subscriptions'**
  String get subscriptionsTitle;

  /// No description provided for @settingsTitle.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settingsTitle;

  /// No description provided for @themeLabel.
  ///
  /// In en, this message translates to:
  /// **'Theme'**
  String get themeLabel;

  /// No description provided for @themeSystem.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get themeSystem;

  /// No description provided for @themeLight.
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get themeLight;

  /// No description provided for @themeDark.
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get themeDark;

  /// No description provided for @languageLabel.
  ///
  /// In en, this message translates to:
  /// **'Language'**
  String get languageLabel;

  /// No description provided for @languageEnglish.
  ///
  /// In en, this message translates to:
  /// **'English'**
  String get languageEnglish;

  /// No description provided for @languageChinese.
  ///
  /// In en, this message translates to:
  /// **'Simplified Chinese'**
  String get languageChinese;

  /// No description provided for @languageJapanese.
  ///
  /// In en, this message translates to:
  /// **'Japanese'**
  String get languageJapanese;

  /// No description provided for @refreshInterval.
  ///
  /// In en, this message translates to:
  /// **'Refresh interval'**
  String get refreshInterval;

  /// No description provided for @minutes.
  ///
  /// In en, this message translates to:
  /// **'{count} min'**
  String minutes(int count);

  /// No description provided for @articleTitle.
  ///
  /// In en, this message translates to:
  /// **'Article'**
  String get articleTitle;

  /// No description provided for @byAuthor.
  ///
  /// In en, this message translates to:
  /// **'By {author}'**
  String byAuthor(String author);

  /// No description provided for @noContent.
  ///
  /// In en, this message translates to:
  /// **'No content'**
  String get noContent;

  /// No description provided for @noArticles.
  ///
  /// In en, this message translates to:
  /// **'No articles yet'**
  String get noArticles;

  /// No description provided for @noSavedArticles.
  ///
  /// In en, this message translates to:
  /// **'No saved articles yet'**
  String get noSavedArticles;

  /// No description provided for @noFeeds.
  ///
  /// In en, this message translates to:
  /// **'No subscriptions yet. Tap + to add one.'**
  String get noFeeds;

  /// No description provided for @selectSubscription.
  ///
  /// In en, this message translates to:
  /// **'Select a subscription to view its articles'**
  String get selectSubscription;

  /// No description provided for @errorMessage.
  ///
  /// In en, this message translates to:
  /// **'Error: {message}'**
  String errorMessage(String message);

  /// No description provided for @refreshTooltip.
  ///
  /// In en, this message translates to:
  /// **'Refresh'**
  String get refreshTooltip;

  /// No description provided for @refreshComplete.
  ///
  /// In en, this message translates to:
  /// **'Refresh complete: {count} new article(s).'**
  String refreshComplete(int count);

  /// No description provided for @refreshPartial.
  ///
  /// In en, this message translates to:
  /// **'Refresh complete: {newCount} new, {errorCount} failed.'**
  String refreshPartial(int newCount, int errorCount);

  /// No description provided for @refreshFailed.
  ///
  /// In en, this message translates to:
  /// **'Refresh failed: {message}'**
  String refreshFailed(String message);

  /// No description provided for @addFeed.
  ///
  /// In en, this message translates to:
  /// **'Add subscription'**
  String get addFeed;

  /// No description provided for @feedUrl.
  ///
  /// In en, this message translates to:
  /// **'Feed or website URL'**
  String get feedUrl;

  /// No description provided for @feedUrlHint.
  ///
  /// In en, this message translates to:
  /// **'https://example.com/feed.xml'**
  String get feedUrlHint;

  /// No description provided for @cancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// No description provided for @add.
  ///
  /// In en, this message translates to:
  /// **'Add'**
  String get add;

  /// No description provided for @feedUrlRequired.
  ///
  /// In en, this message translates to:
  /// **'Enter a feed or website URL'**
  String get feedUrlRequired;

  /// No description provided for @feedTitle.
  ///
  /// In en, this message translates to:
  /// **'Subscription title'**
  String get feedTitle;

  /// No description provided for @appearanceSaveFailed.
  ///
  /// In en, this message translates to:
  /// **'Could not save appearance: {message}'**
  String appearanceSaveFailed(String message);

  /// No description provided for @unknownError.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong'**
  String get unknownError;

  /// No description provided for @manageFolders.
  ///
  /// In en, this message translates to:
  /// **'Manage folders'**
  String get manageFolders;

  /// No description provided for @folderName.
  ///
  /// In en, this message translates to:
  /// **'Folder name'**
  String get folderName;

  /// No description provided for @createFolder.
  ///
  /// In en, this message translates to:
  /// **'Create folder'**
  String get createFolder;

  /// No description provided for @rename.
  ///
  /// In en, this message translates to:
  /// **'Rename'**
  String get rename;

  /// No description provided for @delete.
  ///
  /// In en, this message translates to:
  /// **'Delete'**
  String get delete;

  /// No description provided for @deleteFolderTitle.
  ///
  /// In en, this message translates to:
  /// **'Delete folder?'**
  String get deleteFolderTitle;

  /// No description provided for @deleteFolderMessage.
  ///
  /// In en, this message translates to:
  /// **'Subscriptions in this folder will move to Uncategorized.'**
  String get deleteFolderMessage;

  /// No description provided for @uncategorized.
  ///
  /// In en, this message translates to:
  /// **'Uncategorized'**
  String get uncategorized;

  /// No description provided for @feedActions.
  ///
  /// In en, this message translates to:
  /// **'Subscription actions'**
  String get feedActions;

  /// No description provided for @renameFeed.
  ///
  /// In en, this message translates to:
  /// **'Rename subscription'**
  String get renameFeed;

  /// No description provided for @moveToFolder.
  ///
  /// In en, this message translates to:
  /// **'Move to folder'**
  String get moveToFolder;

  /// No description provided for @refreshIntervalTitle.
  ///
  /// In en, this message translates to:
  /// **'Subscription refresh interval'**
  String get refreshIntervalTitle;

  /// No description provided for @useGlobalInterval.
  ///
  /// In en, this message translates to:
  /// **'Use global interval'**
  String get useGlobalInterval;

  /// No description provided for @refreshOff.
  ///
  /// In en, this message translates to:
  /// **'Do not refresh automatically'**
  String get refreshOff;

  /// No description provided for @deleteFeedTitle.
  ///
  /// In en, this message translates to:
  /// **'Delete subscription?'**
  String get deleteFeedTitle;

  /// No description provided for @deleteFeedMessage.
  ///
  /// In en, this message translates to:
  /// **'Delete {title} and its downloaded articles?'**
  String deleteFeedMessage(String title);

  /// No description provided for @refreshOneComplete.
  ///
  /// In en, this message translates to:
  /// **'Updated {title}: {count} new article(s).'**
  String refreshOneComplete(String title, int count);

  /// No description provided for @directory.
  ///
  /// In en, this message translates to:
  /// **'Directory'**
  String get directory;

  /// No description provided for @search.
  ///
  /// In en, this message translates to:
  /// **'Search'**
  String get search;

  /// No description provided for @directoryHint.
  ///
  /// In en, this message translates to:
  /// **'Search publications or topics'**
  String get directoryHint;

  /// No description provided for @noResults.
  ///
  /// In en, this message translates to:
  /// **'No matching subscriptions'**
  String get noResults;

  /// No description provided for @opml.
  ///
  /// In en, this message translates to:
  /// **'OPML'**
  String get opml;

  /// No description provided for @importOpml.
  ///
  /// In en, this message translates to:
  /// **'Import OPML text'**
  String get importOpml;

  /// No description provided for @exportOpml.
  ///
  /// In en, this message translates to:
  /// **'Export OPML'**
  String get exportOpml;

  /// No description provided for @opmlRequired.
  ///
  /// In en, this message translates to:
  /// **'The selected OPML file is empty.'**
  String get opmlRequired;

  /// No description provided for @opmlImported.
  ///
  /// In en, this message translates to:
  /// **'Imported {imported} subscription(s); {failed} failed.'**
  String opmlImported(int imported, int failed);

  /// No description provided for @opmlExported.
  ///
  /// In en, this message translates to:
  /// **'OPML file saved.'**
  String get opmlExported;

  /// No description provided for @close.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get close;

  /// No description provided for @errorEmptyFolderName.
  ///
  /// In en, this message translates to:
  /// **'Folder name cannot be empty.'**
  String get errorEmptyFolderName;

  /// No description provided for @errorFolderNameExists.
  ///
  /// In en, this message translates to:
  /// **'A folder with that name already exists.'**
  String get errorFolderNameExists;

  /// No description provided for @errorInvalidFolderOrder.
  ///
  /// In en, this message translates to:
  /// **'Folder order changed elsewhere. Reload and try again.'**
  String get errorInvalidFolderOrder;

  /// No description provided for @errorEmptyFeedTitle.
  ///
  /// In en, this message translates to:
  /// **'Subscription title cannot be empty.'**
  String get errorEmptyFeedTitle;

  /// No description provided for @errorFeedAlreadyExists.
  ///
  /// In en, this message translates to:
  /// **'This subscription already exists.'**
  String get errorFeedAlreadyExists;

  /// No description provided for @errorEmptyFeedUrl.
  ///
  /// In en, this message translates to:
  /// **'Subscription URL cannot be empty.'**
  String get errorEmptyFeedUrl;

  /// No description provided for @errorFeedNotFound.
  ///
  /// In en, this message translates to:
  /// **'No feed was found at that address.'**
  String get errorFeedNotFound;

  /// No description provided for @errorInvalidFeedUrl.
  ///
  /// In en, this message translates to:
  /// **'Enter a valid feed or website URL.'**
  String get errorInvalidFeedUrl;

  /// No description provided for @errorNetwork.
  ///
  /// In en, this message translates to:
  /// **'The network request failed. Check your connection and try again.'**
  String get errorNetwork;

  /// No description provided for @errorParse.
  ///
  /// In en, this message translates to:
  /// **'The subscription data could not be read.'**
  String get errorParse;

  /// No description provided for @retry.
  ///
  /// In en, this message translates to:
  /// **'Retry'**
  String get retry;

  /// No description provided for @articleActions.
  ///
  /// In en, this message translates to:
  /// **'Article actions'**
  String get articleActions;

  /// No description provided for @readerActions.
  ///
  /// In en, this message translates to:
  /// **'Reader actions'**
  String get readerActions;

  /// No description provided for @markRead.
  ///
  /// In en, this message translates to:
  /// **'Mark read'**
  String get markRead;

  /// No description provided for @markUnread.
  ///
  /// In en, this message translates to:
  /// **'Mark unread'**
  String get markUnread;

  /// No description provided for @star.
  ///
  /// In en, this message translates to:
  /// **'Star'**
  String get star;

  /// No description provided for @unstar.
  ///
  /// In en, this message translates to:
  /// **'Unstar'**
  String get unstar;

  /// No description provided for @readLater.
  ///
  /// In en, this message translates to:
  /// **'Read later'**
  String get readLater;

  /// No description provided for @removeReadLater.
  ///
  /// In en, this message translates to:
  /// **'Remove from read later'**
  String get removeReadLater;

  /// No description provided for @searchArticles.
  ///
  /// In en, this message translates to:
  /// **'Search title and full text'**
  String get searchArticles;

  /// No description provided for @listOptions.
  ///
  /// In en, this message translates to:
  /// **'List options'**
  String get listOptions;

  /// No description provided for @hideRead.
  ///
  /// In en, this message translates to:
  /// **'Hide read articles'**
  String get hideRead;

  /// No description provided for @oldestFirst.
  ///
  /// In en, this message translates to:
  /// **'Oldest first'**
  String get oldestFirst;

  /// No description provided for @markAllRead.
  ///
  /// In en, this message translates to:
  /// **'Mark current view read'**
  String get markAllRead;

  /// No description provided for @markedAllRead.
  ///
  /// In en, this message translates to:
  /// **'Marked {count} article(s) read.'**
  String markedAllRead(int count);

  /// No description provided for @smartViews.
  ///
  /// In en, this message translates to:
  /// **'Smart views'**
  String get smartViews;

  /// No description provided for @allArticles.
  ///
  /// In en, this message translates to:
  /// **'All articles'**
  String get allArticles;

  /// No description provided for @unreadArticles.
  ///
  /// In en, this message translates to:
  /// **'Unread'**
  String get unreadArticles;

  /// No description provided for @starredArticles.
  ///
  /// In en, this message translates to:
  /// **'Starred'**
  String get starredArticles;

  /// No description provided for @readLaterArticles.
  ///
  /// In en, this message translates to:
  /// **'Read later'**
  String get readLaterArticles;

  /// No description provided for @folders.
  ///
  /// In en, this message translates to:
  /// **'Folders'**
  String get folders;

  /// No description provided for @tags.
  ///
  /// In en, this message translates to:
  /// **'Tags'**
  String get tags;

  /// No description provided for @extractFulltext.
  ///
  /// In en, this message translates to:
  /// **'Extract full text'**
  String get extractFulltext;

  /// No description provided for @reextractFulltext.
  ///
  /// In en, this message translates to:
  /// **'Extract again'**
  String get reextractFulltext;

  /// No description provided for @fulltextExtracted.
  ///
  /// In en, this message translates to:
  /// **'Full text extracted.'**
  String get fulltextExtracted;

  /// No description provided for @openInBrowser.
  ///
  /// In en, this message translates to:
  /// **'Open in browser'**
  String get openInBrowser;

  /// No description provided for @shareArticle.
  ///
  /// In en, this message translates to:
  /// **'Share'**
  String get shareArticle;

  /// No description provided for @platformActionFailed.
  ///
  /// In en, this message translates to:
  /// **'No app is available for that action.'**
  String get platformActionFailed;

  /// No description provided for @readingSettings.
  ///
  /// In en, this message translates to:
  /// **'Reading settings'**
  String get readingSettings;

  /// No description provided for @readerFont.
  ///
  /// In en, this message translates to:
  /// **'Reader font'**
  String get readerFont;

  /// No description provided for @fontSystem.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get fontSystem;

  /// No description provided for @fontSerif.
  ///
  /// In en, this message translates to:
  /// **'Serif'**
  String get fontSerif;

  /// No description provided for @fontSans.
  ///
  /// In en, this message translates to:
  /// **'Sans'**
  String get fontSans;

  /// No description provided for @fontSize.
  ///
  /// In en, this message translates to:
  /// **'Font size'**
  String get fontSize;

  /// No description provided for @lineHeight.
  ///
  /// In en, this message translates to:
  /// **'Line spacing'**
  String get lineHeight;

  /// No description provided for @readingWidth.
  ///
  /// In en, this message translates to:
  /// **'Reading width'**
  String get readingWidth;

  /// No description provided for @showReadingTime.
  ///
  /// In en, this message translates to:
  /// **'Show reading time'**
  String get showReadingTime;

  /// No description provided for @autoExtractFulltext.
  ///
  /// In en, this message translates to:
  /// **'Automatically extract full text'**
  String get autoExtractFulltext;

  /// No description provided for @autoExtractFulltextDescription.
  ///
  /// In en, this message translates to:
  /// **'Fetch the source page when no extracted copy is cached.'**
  String get autoExtractFulltextDescription;

  /// No description provided for @readMinutes.
  ///
  /// In en, this message translates to:
  /// **'{count} min read'**
  String readMinutes(int count);

  /// No description provided for @attachments.
  ///
  /// In en, this message translates to:
  /// **'Attachments'**
  String get attachments;

  /// No description provided for @attachment.
  ///
  /// In en, this message translates to:
  /// **'Attachment'**
  String get attachment;

  /// No description provided for @imageUnavailable.
  ///
  /// In en, this message translates to:
  /// **'Image unavailable'**
  String get imageUnavailable;

  /// No description provided for @errorArticleNotFound.
  ///
  /// In en, this message translates to:
  /// **'The article no longer exists.'**
  String get errorArticleNotFound;

  /// No description provided for @errorArticleUrlMissing.
  ///
  /// In en, this message translates to:
  /// **'This article has no source URL.'**
  String get errorArticleUrlMissing;

  /// No description provided for @errorNoExtractableContent.
  ///
  /// In en, this message translates to:
  /// **'No readable full text was found.'**
  String get errorNoExtractableContent;

  /// No description provided for @errorInvalidReadingSettings.
  ///
  /// In en, this message translates to:
  /// **'The reading setting is outside the supported range.'**
  String get errorInvalidReadingSettings;

  /// No description provided for @addHighlight.
  ///
  /// In en, this message translates to:
  /// **'Add highlight'**
  String get addHighlight;

  /// No description provided for @highlights.
  ///
  /// In en, this message translates to:
  /// **'Highlights'**
  String get highlights;

  /// No description provided for @noHighlights.
  ///
  /// In en, this message translates to:
  /// **'No highlights yet. Select text in the article to add one.'**
  String get noHighlights;

  /// No description provided for @highlightColor.
  ///
  /// In en, this message translates to:
  /// **'Color'**
  String get highlightColor;

  /// No description provided for @highlightNote.
  ///
  /// In en, this message translates to:
  /// **'Note'**
  String get highlightNote;

  /// No description provided for @highlightCreated.
  ///
  /// In en, this message translates to:
  /// **'Highlight added.'**
  String get highlightCreated;

  /// No description provided for @highlightSelectionUnavailable.
  ///
  /// In en, this message translates to:
  /// **'That selection cannot be anchored in this article. Try selecting it again.'**
  String get highlightSelectionUnavailable;

  /// No description provided for @highlightUnableToLocate.
  ///
  /// In en, this message translates to:
  /// **'Some highlights could not be located in the current article text.'**
  String get highlightUnableToLocate;

  /// No description provided for @save.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get save;

  /// No description provided for @errorEmptyTagName.
  ///
  /// In en, this message translates to:
  /// **'Tag name cannot be empty.'**
  String get errorEmptyTagName;

  /// No description provided for @errorTagNameExists.
  ///
  /// In en, this message translates to:
  /// **'A tag with that name already exists.'**
  String get errorTagNameExists;

  /// No description provided for @errorInvalidTagColor.
  ///
  /// In en, this message translates to:
  /// **'Choose a supported tag color.'**
  String get errorInvalidTagColor;

  /// No description provided for @errorInvalidTagOrder.
  ///
  /// In en, this message translates to:
  /// **'Tag order changed elsewhere. Reload and try again.'**
  String get errorInvalidTagOrder;

  /// No description provided for @errorRuleNotFound.
  ///
  /// In en, this message translates to:
  /// **'This rule no longer exists.'**
  String get errorRuleNotFound;

  /// No description provided for @errorEmptyRuleName.
  ///
  /// In en, this message translates to:
  /// **'Rule name cannot be empty.'**
  String get errorEmptyRuleName;

  /// No description provided for @errorEmptyRuleQuery.
  ///
  /// In en, this message translates to:
  /// **'Enter at least one rule keyword.'**
  String get errorEmptyRuleQuery;

  /// No description provided for @errorInvalidRuleField.
  ///
  /// In en, this message translates to:
  /// **'Choose a supported rule field.'**
  String get errorInvalidRuleField;

  /// No description provided for @errorInvalidRuleAction.
  ///
  /// In en, this message translates to:
  /// **'Choose a supported rule action.'**
  String get errorInvalidRuleAction;

  /// No description provided for @errorHighlightNotFound.
  ///
  /// In en, this message translates to:
  /// **'This highlight no longer exists.'**
  String get errorHighlightNotFound;

  /// No description provided for @errorEmptyHighlight.
  ///
  /// In en, this message translates to:
  /// **'Select some text to create a highlight.'**
  String get errorEmptyHighlight;

  /// No description provided for @errorInvalidHighlightOffset.
  ///
  /// In en, this message translates to:
  /// **'That highlight position is invalid.'**
  String get errorInvalidHighlightOffset;

  /// No description provided for @errorInvalidHighlightColor.
  ///
  /// In en, this message translates to:
  /// **'Choose a supported highlight color.'**
  String get errorInvalidHighlightColor;

  /// No description provided for @manageTags.
  ///
  /// In en, this message translates to:
  /// **'Manage tags'**
  String get manageTags;

  /// No description provided for @manageRules.
  ///
  /// In en, this message translates to:
  /// **'Manage rules'**
  String get manageRules;

  /// No description provided for @tagName.
  ///
  /// In en, this message translates to:
  /// **'Tag name'**
  String get tagName;

  /// No description provided for @createTag.
  ///
  /// In en, this message translates to:
  /// **'Create tag'**
  String get createTag;

  /// No description provided for @newRule.
  ///
  /// In en, this message translates to:
  /// **'New rule'**
  String get newRule;

  /// No description provided for @ruleName.
  ///
  /// In en, this message translates to:
  /// **'Rule name'**
  String get ruleName;

  /// No description provided for @ruleKeywords.
  ///
  /// In en, this message translates to:
  /// **'Keywords (comma separated)'**
  String get ruleKeywords;

  /// No description provided for @ruleField.
  ///
  /// In en, this message translates to:
  /// **'Match field'**
  String get ruleField;

  /// No description provided for @ruleAction.
  ///
  /// In en, this message translates to:
  /// **'Action'**
  String get ruleAction;

  /// No description provided for @ruleEnabled.
  ///
  /// In en, this message translates to:
  /// **'Enabled'**
  String get ruleEnabled;

  /// No description provided for @rulePreview.
  ///
  /// In en, this message translates to:
  /// **'Preview'**
  String get rulePreview;

  /// No description provided for @rulePreviewResult.
  ///
  /// In en, this message translates to:
  /// **'Matches {count} article(s)'**
  String rulePreviewResult(int count);

  /// No description provided for @applyRule.
  ///
  /// In en, this message translates to:
  /// **'Apply to existing articles'**
  String get applyRule;

  /// No description provided for @applySkipRuleTitle.
  ///
  /// In en, this message translates to:
  /// **'Apply skip rule?'**
  String get applySkipRuleTitle;

  /// No description provided for @applySkipRuleMessage.
  ///
  /// In en, this message translates to:
  /// **'Matching unsaved articles will be removed. Starred, read-later, and highlighted articles are kept.'**
  String get applySkipRuleMessage;

  /// No description provided for @applyRuleComplete.
  ///
  /// In en, this message translates to:
  /// **'Applied to {count} article(s).'**
  String applyRuleComplete(Object count);

  /// No description provided for @titleField.
  ///
  /// In en, this message translates to:
  /// **'Title'**
  String get titleField;

  /// No description provided for @authorField.
  ///
  /// In en, this message translates to:
  /// **'Author'**
  String get authorField;

  /// No description provided for @contentField.
  ///
  /// In en, this message translates to:
  /// **'Content'**
  String get contentField;

  /// No description provided for @anyField.
  ///
  /// In en, this message translates to:
  /// **'Any field'**
  String get anyField;

  /// No description provided for @skipAction.
  ///
  /// In en, this message translates to:
  /// **'Skip'**
  String get skipAction;

  /// No description provided for @readAction.
  ///
  /// In en, this message translates to:
  /// **'Mark read'**
  String get readAction;

  /// No description provided for @starAction.
  ///
  /// In en, this message translates to:
  /// **'Star'**
  String get starAction;

  /// No description provided for @editTags.
  ///
  /// In en, this message translates to:
  /// **'Edit tags'**
  String get editTags;

  /// No description provided for @newTagHint.
  ///
  /// In en, this message translates to:
  /// **'Create a new tag'**
  String get newTagHint;

  /// No description provided for @globalHighlights.
  ///
  /// In en, this message translates to:
  /// **'All highlights'**
  String get globalHighlights;

  /// No description provided for @noGlobalHighlights.
  ///
  /// In en, this message translates to:
  /// **'No highlights yet.'**
  String get noGlobalHighlights;

  /// No description provided for @allFeeds.
  ///
  /// In en, this message translates to:
  /// **'All subscriptions'**
  String get allFeeds;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'ja', 'zh'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'ja':
      return AppLocalizationsJa();
    case 'zh':
      return AppLocalizationsZh();
  }

  throw FlutterError(
      'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
      'an issue with the localizations generation tool. Please file an issue '
      'on GitHub with a reproducible sample app and the gen-l10n configuration '
      'that was used.');
}
