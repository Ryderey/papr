import 'package:flutter/widgets.dart';

import '../core/exceptions.dart';
import 'app_localizations.dart';

export 'app_localizations.dart';

extension AppLocalizationsContext on BuildContext {
  AppLocalizations get l10n => AppLocalizations.of(this);
}

extension AppErrorLocalizations on AppLocalizations {
  String localizeError(Object error) {
    if (error is! AppException) return errorMessage(error.toString());
    return switch (error.code) {
      'emptyFolderName' => errorEmptyFolderName,
      'folderNameExists' => errorFolderNameExists,
      'invalidFolderOrder' => errorInvalidFolderOrder,
      'emptyFeedTitle' => errorEmptyFeedTitle,
      'feedAlreadyExists' => errorFeedAlreadyExists,
      'emptyFeedUrl' => errorEmptyFeedUrl,
      'feedNotFound' => errorFeedNotFound,
      'invalidFeedUrl' => errorInvalidFeedUrl,
      'articleNotFound' => errorArticleNotFound,
      'emptyTagName' => errorEmptyTagName,
      'tagNameExists' => errorTagNameExists,
      'invalidTagColor' => errorInvalidTagColor,
      'invalidTagOrder' => errorInvalidTagOrder,
      'ruleNotFound' => errorRuleNotFound,
      'emptyRuleName' => errorEmptyRuleName,
      'emptyRuleQuery' => errorEmptyRuleQuery,
      'invalidRuleField' => errorInvalidRuleField,
      'invalidRuleAction' => errorInvalidRuleAction,
      'highlightNotFound' => errorHighlightNotFound,
      'emptyHighlight' => errorEmptyHighlight,
      'invalidHighlightOffset' => errorInvalidHighlightOffset,
      'invalidHighlightColor' => errorInvalidHighlightColor,
      'articleUrlMissing' => errorArticleUrlMissing,
      'noExtractableContent' => errorNoExtractableContent,
      'invalidReadingFont' ||
      'invalidReadingFontSize' ||
      'invalidReadingLineHeight' ||
      'invalidReadingWidth' =>
        errorInvalidReadingSettings,
      'invalidAiProfile' || 'tooManyAiProfiles' => errorInvalidAiProfile,
      'noAiCredential' => errorNoAiCredential,
      'invalidAiCredential' ||
      'credentialWriteFailed' ||
      'credentialReadFailed' ||
      'credentialDeleteFailed' =>
        errorAiCredentialStore,
      'aiAuth' => errorAiAuth,
      'aiRateLimited' => errorNetwork,
      'aiNetwork' => errorNetwork,
      'aiParse' => errorParse,
      'aiNoVisibleOutput' => errorAiNoVisibleOutput,
      _ => switch (error.kind) {
          AppErrorKind.network => errorNetwork,
          AppErrorKind.parse => errorParse,
          _ =>
            error.detail == null ? unknownError : errorMessage(error.detail!),
        },
    };
  }
}
