import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../core/di.dart';
import '../services/papr_core_service.dart';

final settingsRepositoryProvider = Provider<SettingsRepository>((ref) {
  return SettingsRepository(ref);
});

final appearanceProvider =
    AsyncNotifierProvider<AppearanceController, AppearanceState>(
  AppearanceController.new,
);

class AppearanceState {
  final String theme;
  final String language;
  final int refreshIntervalMin;
  final bridge.ReadingSettings reading;

  const AppearanceState({
    required this.theme,
    required this.language,
    required this.refreshIntervalMin,
    required this.reading,
  });

  const AppearanceState.defaults()
      : theme = 'system',
        language = 'en',
        refreshIntervalMin = 30,
        reading = const bridge.ReadingSettings(
          font: 'system',
          fontSize: 17,
          lineHeight: 1.65,
          contentWidth: 680,
          showReadingTime: true,
          autoExtract: false,
        );

  AppearanceState copyWith({
    String? theme,
    String? language,
    bridge.ReadingSettings? reading,
  }) {
    return AppearanceState(
      theme: theme ?? this.theme,
      language: language ?? this.language,
      refreshIntervalMin: refreshIntervalMin,
      reading: reading ?? this.reading,
    );
  }
}

class AppearanceController extends AsyncNotifier<AppearanceState> {
  @override
  Future<AppearanceState> build() async {
    final snapshot = await ref.watch(settingsRepositoryProvider).getSettings();
    return AppearanceState(
      theme: snapshot.theme,
      language: snapshot.language,
      refreshIntervalMin: snapshot.refreshIntervalMin.toInt(),
      reading: snapshot.reading,
    );
  }

  Future<void> setTheme(String theme) async {
    final previous = state.asData?.value ?? const AppearanceState.defaults();
    state = AsyncData(previous.copyWith(theme: theme));
    try {
      await ref.read(settingsRepositoryProvider).setTheme(theme);
    } catch (error) {
      state = AsyncData(previous);
      rethrow;
    }
  }

  Future<void> setLanguage(String language) async {
    final previous = state.asData?.value ?? const AppearanceState.defaults();
    state = AsyncData(previous.copyWith(language: language));
    try {
      await ref.read(settingsRepositoryProvider).setLanguage(language);
    } catch (error) {
      state = AsyncData(previous);
      rethrow;
    }
  }

  Future<void> setReading(bridge.ReadingSettings reading) async {
    final previous = state.asData?.value ?? const AppearanceState.defaults();
    state = AsyncData(previous.copyWith(reading: reading));
    try {
      await ref.read(settingsRepositoryProvider).setReading(reading);
    } catch (error) {
      state = AsyncData(previous);
      rethrow;
    }
  }
}

class SettingsRepository {
  final Ref _ref;

  SettingsRepository(this._ref);

  Future<bridge.SettingsSnapshot> getSettings() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.getSettings(core: core);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> setTheme(String theme) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.setTheme(core: core, theme: theme);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> setLanguage(String language) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.setLanguage(core: core, language: language);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> setReading(bridge.ReadingSettings settings) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.setReadingSettings(core: core, settings: settings);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }
}

bridge.ReadingSettings copyReadingSettings(
  bridge.ReadingSettings settings, {
  String? font,
  double? fontSize,
  double? lineHeight,
  double? contentWidth,
  bool? showReadingTime,
  bool? autoExtract,
}) {
  return bridge.ReadingSettings(
    font: font ?? settings.font,
    fontSize: fontSize ?? settings.fontSize,
    lineHeight: lineHeight ?? settings.lineHeight,
    contentWidth: contentWidth ?? settings.contentWidth,
    showReadingTime: showReadingTime ?? settings.showReadingTime,
    autoExtract: autoExtract ?? settings.autoExtract,
  );
}
