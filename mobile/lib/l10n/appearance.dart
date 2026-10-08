import 'app_localizations.dart';

extension AppearanceLocalization on AppLocalizations {
  String presetName(String id) => switch (id) {
        'paper' => presetPaperName,
        'pine' => presetPineName,
        'ink' => presetInkName,
        'dusk' => presetDuskName,
        'midnight' => presetMidnightName,
        'focus' => presetFocusName,
        _ => appearanceCustom,
      };

  String presetDescription(String id) => switch (id) {
        'paper' => presetPaperDescription,
        'pine' => presetPineDescription,
        'ink' => presetInkDescription,
        'dusk' => presetDuskDescription,
        'midnight' => presetMidnightDescription,
        'focus' => presetFocusDescription,
        _ => '',
      };
}
