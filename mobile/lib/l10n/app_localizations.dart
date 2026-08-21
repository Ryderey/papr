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
