import '../bridge/generated/generated.dart' as bridge;

const defaultVisualSettings = bridge.VisualSettings(
  accent: 'clay',
  darkShade: 'default',
  density: 'cozy',
  viewMode: 'card',
);

class AppearancePreset {
  final String id;
  final String theme;
  final bridge.VisualSettings visual;
  final String font;
  final double fontSize;
  final double lineHeight;
  final double contentWidth;

  const AppearancePreset(this.id, this.theme, this.visual, this.font,
      this.fontSize, this.lineHeight, this.contentWidth);

  bridge.ReadingSettings readingSettings(bridge.ReadingSettings previous) =>
      bridge.ReadingSettings(
        font: font,
        fontSize: fontSize,
        lineHeight: lineHeight,
        contentWidth: contentWidth,
        showReadingTime: previous.showReadingTime,
        autoExtract: previous.autoExtract,
      );

  bool matches(String currentTheme, bridge.VisualSettings currentVisual,
          bridge.ReadingSettings reading) =>
      currentTheme == theme &&
      currentVisual.accent == visual.accent &&
      (theme == 'light' || currentVisual.darkShade == visual.darkShade) &&
      currentVisual.density == visual.density &&
      currentVisual.viewMode == visual.viewMode &&
      reading.font == font &&
      reading.fontSize == fontSize &&
      reading.lineHeight == lineHeight &&
      reading.contentWidth == contentWidth;
}

const appearancePresets = [
  AppearancePreset(
      'paper',
      'light',
      bridge.VisualSettings(
          accent: 'clay',
          darkShade: 'default',
          density: 'cozy',
          viewMode: 'list'),
      'serif',
      17,
      1.65,
      680),
  AppearancePreset(
      'pine',
      'light',
      bridge.VisualSettings(
          accent: 'pine',
          darkShade: 'default',
          density: 'spacious',
          viewMode: 'card'),
      'serif',
      18,
      1.75,
      680),
  AppearancePreset(
      'ink',
      'light',
      bridge.VisualSettings(
          accent: 'ink',
          darkShade: 'default',
          density: 'compact',
          viewMode: 'list'),
      'sans',
      16,
      1.55,
      720),
  AppearancePreset(
      'dusk',
      'dark',
      bridge.VisualSettings(
          accent: 'clay',
          darkShade: 'default',
          density: 'cozy',
          viewMode: 'list'),
      'serif',
      18,
      1.75,
      680),
  AppearancePreset(
      'midnight',
      'dark',
      bridge.VisualSettings(
          accent: 'indigo',
          darkShade: 'dimmer',
          density: 'cozy',
          viewMode: 'card'),
      'sans',
      17,
      1.65,
      720),
  AppearancePreset(
      'focus',
      'dark',
      bridge.VisualSettings(
          accent: 'pine',
          darkShade: 'black',
          density: 'compact',
          viewMode: 'list'),
      'sans',
      17,
      1.70,
      640),
];

String? matchingAppearancePreset(String theme, bridge.VisualSettings visual,
    bridge.ReadingSettings reading) {
  for (final preset in appearancePresets) {
    if (preset.matches(theme, visual, reading)) return preset.id;
  }
  return null;
}

bridge.VisualSettings copyVisualSettings(
  bridge.VisualSettings previous, {
  String? accent,
  String? darkShade,
  String? density,
  String? viewMode,
}) =>
    bridge.VisualSettings(
      accent: accent ?? previous.accent,
      darkShade: darkShade ?? previous.darkShade,
      density: density ?? previous.density,
      viewMode: viewMode ?? previous.viewMode,
    );
