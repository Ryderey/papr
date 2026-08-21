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
      _ => switch (error.kind) {
          AppErrorKind.network => errorNetwork,
          AppErrorKind.parse => errorParse,
          _ =>
            error.detail == null ? unknownError : errorMessage(error.detail!),
        },
    };
  }
}
