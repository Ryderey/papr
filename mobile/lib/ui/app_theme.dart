import 'package:flutter/material.dart';
import '../bridge/generated/generated.dart' as bridge;

ThemeData paprTheme(bridge.VisualSettings visual, Brightness brightness) {
  final dark = brightness == Brightness.dark;
  // These are not the desktop OKLCH values copied across. In light mode the
  // accent fills buttons that carry white labels (onPrimary: Colors.white), so
  // every light accent is darkened until white text clears 4.5:1 — the shipped
  // four sit at 6.3-7.5:1 and the newer six at ~4.5:1. Dark mode carries the
  // near-black onPrimary instead, so it keeps the desktop lightness.
  final accent = switch ((visual.accent, dark)) {
    ('pine', false) => const Color(0xFF28634B),
    ('pine', true) => const Color(0xFF8CC3A7),
    ('indigo', false) => const Color(0xFF474BA4),
    ('indigo', true) => const Color(0xFFA8ACF2),
    ('ink', false) => const Color(0xFF2B2620),
    ('ink', true) => const Color(0xFFDAD6D1),
    ('mauve', false) => const Color(0xFF9361B6),
    ('mauve', true) => const Color(0xFFCE9FF1),
    ('frost', false) => const Color(0xFF117EAE),
    ('frost', true) => const Color(0xFF62BBEB),
    ('leaf', false) => const Color(0xFF3D854B),
    ('leaf', true) => const Color(0xFF7CC186),
    ('amber', false) => const Color(0xFF9F6D12),
    ('amber', true) => const Color(0xFFDCA85E),
    ('rose', false) => const Color(0xFFC05170),
    ('rose', true) => const Color(0xFFFD8FAA),
    ('slate', false) => const Color(0xFF6C7789),
    ('slate', true) => const Color(0xFFA6B2C5),
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
