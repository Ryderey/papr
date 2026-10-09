import 'app_localizations.dart';

extension AppearanceLocalization on AppLocalizations {
  String presetName(String id) => switch (id) {
        'paper' => presetPaperName,
        'pine' => presetPineName,
        'ink' => presetInkName,
        'orchid' => presetOrchidName,
        'dusk' => presetDuskName,
        'midnight' => presetMidnightName,
        'focus' => presetFocusName,
        'glacier' => presetGlacierName,
        _ => appearanceCustom,
      };

  String presetDescription(String id) => switch (id) {
        'paper' => presetPaperDescription,
        'pine' => presetPineDescription,
        'ink' => presetInkDescription,
        'orchid' => presetOrchidDescription,
        'dusk' => presetDuskDescription,
        'midnight' => presetMidnightDescription,
        'focus' => presetFocusDescription,
        'glacier' => presetGlacierDescription,
        _ => '',
      };
}
