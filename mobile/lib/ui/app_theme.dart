import 'package:flutter/material.dart';
import '../bridge/generated/generated.dart' as bridge;

ThemeData paprTheme(bridge.VisualSettings visual, Brightness brightness) {
  final dark = brightness == Brightness.dark;
  final accent = switch ((visual.accent, dark)) {
    ('pine', false) => const Color(0xFF28634B),
    ('pine', true) => const Color(0xFF8CC3A7),
    ('indigo', false) => const Color(0xFF474BA4),
    ('indigo', true) => const Color(0xFFA8ACF2),
    ('ink', false) => const Color(0xFF2B2620),
    ('ink', true) => const Color(0xFFDAD6D1),
    (_, true) => const Color(0xFFDA8564),
    _ => const Color(0xFF914D33),
  };
  final (panel, container, reader) = switch ((dark, visual.darkShade)) {
    (true, 'black') => (
        const Color(0xFF050403),
        const Color(0xFF0D0A0A),
        const Color(0xFF15100F)
      ),
    (true, 'dimmer') => (
        const Color(0xFF0D0A0A),
        const Color(0xFF161312),
        const Color(0xFF1C1715)
      ),
    (true, _) => (
        const Color(0xFF161312),
        const Color(0xFF1F1C1B),
        const Color(0xFF25201F)
      ),
    _ => (
        const Color(0xFFF6F3EC),
        const Color(0xFFFFFEFA),
        const Color(0xFFFBF9F3)
      ),
  };
  final colors =
      ColorScheme.fromSeed(seedColor: accent, brightness: brightness).copyWith(
    primary: accent,
    onPrimary: dark ? const Color(0xFF1A1816) : Colors.white,
    surface: reader,
    surfaceContainerLowest: panel,
    surfaceContainerLow: panel,
    surfaceContainer: container,
    surfaceContainerHigh: container,
    surfaceContainerHighest: Color.alphaBlend(
        dark ? const Color(0x12FFFFFF) : const Color(0x08141210), container),
    onSurface: dark ? const Color(0xFFEBE7E4) : const Color(0xFF1A1816),
    onSurfaceVariant: dark ? const Color(0xFFBDB5AC) : const Color(0xFF6D675E),
    outlineVariant: dark ? const Color(0xFF49413D) : const Color(0xFFDAD5CC),
    surfaceTint: Colors.transparent,
  );
  return ThemeData(
    useMaterial3: true,
    brightness: brightness,
    colorScheme: colors,
    scaffoldBackgroundColor: reader,
    appBarTheme: AppBarTheme(
        backgroundColor: reader,
        surfaceTintColor: Colors.transparent,
        elevation: 0),
    cardTheme: CardThemeData(
        color: container,
        elevation: 0,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14))),
    navigationBarTheme: NavigationBarThemeData(backgroundColor: panel),
    dividerTheme:
        DividerThemeData(color: colors.outlineVariant, thickness: 0.5),
  );
}
