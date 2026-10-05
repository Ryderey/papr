import 'package:flutter/widgets.dart';

import '../core/exceptions.dart';
import 'app_localizations.dart';

export 'app_localizations.dart';

extension AppLocalizationsContext on BuildContext {
  AppLocalizations get l10n => AppLocalizations.of(this);
}

extension AppErrorLocalizations on AppLocalizations {
  String localizeSyncCode(String code) => switch (code) {
        'invalidSyncProfile' => errorInvalidSyncProfile,
        'syncCredentialMissing' => errorSyncCredentialMissing,
        'syncNotConnected' => errorSyncNotConnected,
        'syncAuthFailed' => errorSyncAuthFailed,
        'syncUnavailable' => errorSyncUnavailable,
        'syncProviderFailed' => errorSyncProviderFailed,
        'syncInvalidResponse' ||
        'invalidSyncChange' ||
        'invalidSyncPushCursor' =>
          errorSyncInvalidResponse,
        'syncTooManyItems' => errorSyncTooManyItems,
        _ => unknownError,
      };

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
        error.kind == AppErrorKind.sync
            ? errorSyncCredentialStore
            : errorAiCredentialStore,
      'aiAuth' => errorAiAuth,
      'aiRateLimited' => errorNetwork,
      'aiNetwork' => errorNetwork,
      'aiParse' => errorParse,
      'aiNoVisibleOutput' => errorAiNoVisibleOutput,
      'invalidSyncProfile' ||
      'syncCredentialMissing' ||
      'syncNotConnected' ||
      'syncAuthFailed' ||
      'syncUnavailable' ||
      'syncProviderFailed' ||
      'syncInvalidResponse' ||
      'invalidSyncChange' ||
      'invalidSyncPushCursor' ||
      'syncTooManyItems' =>
        localizeSyncCode(error.code),
      'invalidSyncCredential' => errorSyncCredentialStore,
      _ => switch (error.kind) {
          AppErrorKind.sync => unknownError,
          AppErrorKind.network => errorNetwork,
          AppErrorKind.parse => errorParse,
          _ =>
            error.detail == null ? unknownError : errorMessage(error.detail!),
        },
    };
  }
}
