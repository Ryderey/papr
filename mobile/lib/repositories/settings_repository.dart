import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../core/di.dart';
import '../services/papr_core_service.dart';
import '../services/background_refresh_service.dart';
import '../models/appearance_presets.dart';

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
  final bool notificationsEnabled;
  final bool notificationQuietHours;
  final bridge.ReadingSettings reading;
  final bridge.VisualSettings visual;
  final bool savingAppearance;

  const AppearanceState({
    required this.theme,
    required this.language,
    required this.refreshIntervalMin,
    required this.notificationsEnabled,
    required this.notificationQuietHours,
    required this.reading,
    this.visual = defaultVisualSettings,
    this.savingAppearance = false,
  });

  const AppearanceState.defaults()
      : theme = 'system',
        visual = defaultVisualSettings,
        savingAppearance = false,
        language = 'en',
        refreshIntervalMin = 30,
        notificationsEnabled = false,
        notificationQuietHours = false,
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
    int? refreshIntervalMin,
    bool? notificationsEnabled,
    bool? notificationQuietHours,
    bridge.ReadingSettings? reading,
    bridge.VisualSettings? visual,
    bool? savingAppearance,
  }) {
    return AppearanceState(
      theme: theme ?? this.theme,
      language: language ?? this.language,
      refreshIntervalMin: refreshIntervalMin ?? this.refreshIntervalMin,
      notificationsEnabled: notificationsEnabled ?? this.notificationsEnabled,
      notificationQuietHours:
          notificationQuietHours ?? this.notificationQuietHours,
      reading: reading ?? this.reading,
      visual: visual ?? this.visual,
      savingAppearance: savingAppearance ?? this.savingAppearance,
    );
  }
}

class AppearanceController extends AsyncNotifier<AppearanceState> {
  Future<void> _appearanceWrites = Future.value();
  Future<void> _reconcile(int interval) async {
    final core = await ref.read(paprCoreBridgeProvider.future);
    await ref
        .read(backgroundRefreshServiceProvider)
        .reconcileCore(core, refreshIntervalMin: interval);
  }

  @override
  Future<AppearanceState> build() async {
    final snapshot = await ref.watch(settingsRepositoryProvider).getSettings();
    final settings = AppearanceState(
      theme: snapshot.theme,
      language: snapshot.language,
      refreshIntervalMin: snapshot.refreshIntervalMin.toInt(),
      notificationsEnabled: snapshot.notificationsEnabled,
      notificationQuietHours: snapshot.notificationQuietHours,
      reading: snapshot.reading,
      visual: snapshot.visual,
    );
    try {
      await _reconcile(settings.refreshIntervalMin);
    } catch (_) {
      // Settings remain usable when Android temporarily rejects scheduling.
    }
    return settings;
  }

  Future<void> setTheme(String theme) async {
    await _saveAppearance(theme: theme);
  }

  Future<void> applyAppearancePreset(AppearancePreset preset) async {
    await _saveAppearance(preset: preset);
  }

  Future<void> setVisual(bridge.VisualSettings visual) =>
      _saveAppearance(visual: visual);

  Future<void> _saveAppearance(
      {String? theme,
      bridge.VisualSettings? visual,
      bridge.ReadingSettings? reading,
      AppearancePreset? preset}) {
    final operation = _appearanceWrites.then((_) => _persistAppearance(
        theme: theme, visual: visual, reading: reading, preset: preset));
    _appearanceWrites = operation.catchError((Object _) {});
    return operation;
  }

  Future<void> _persistAppearance(
      {String? theme,
      bridge.VisualSettings? visual,
      bridge.ReadingSettings? reading,
      AppearancePreset? preset}) async {
    final previous = state.asData?.value;
    if (previous == null) {
      throw StateError('Appearance settings are not loaded');
    }
    final nextTheme = preset?.theme ?? theme ?? previous.theme;
    final nextVisual = preset?.visual ?? visual ?? previous.visual;
    final nextReading = preset?.readingSettings(previous.reading) ??
        reading ??
        previous.reading;
    state = AsyncData(previous.copyWith(savingAppearance: true));
    try {
      await ref.read(settingsRepositoryProvider).setAppearance(
            theme: nextTheme,
            visual: nextVisual,
            reading: nextReading,
          );
      final current = state.asData?.value ?? previous;
      state = AsyncData(current.copyWith(
          theme: nextTheme,
          visual: nextVisual,
          reading: nextReading,
          savingAppearance: false));
    } catch (_) {
      final current = state.asData?.value ?? previous;
      state = AsyncData(current.copyWith(savingAppearance: false));
      rethrow;
    }
  }

  Future<void> setLanguage(String language) async {
    final previous = state.asData?.value ?? const AppearanceState.defaults();
    state = AsyncData(previous.copyWith(language: language));
    try {
      await ref.read(settingsRepositoryProvider).setLanguage(language);
    } catch (error) {
      state = AsyncData((state.asData?.value ?? previous)
          .copyWith(language: previous.language));
      rethrow;
    }
  }

  Future<void> setReading(bridge.ReadingSettings reading) async {
    await _saveAppearance(reading: reading);
  }

  Future<void> setAutoRefresh(bool enabled) => _setBackground(
        refreshIntervalMin: enabled ? 30 : refreshOffMinutes,
      );

  Future<void> setRefreshInterval(int minutes) =>
      _setBackground(refreshIntervalMin: minutes);

  Future<bool> setNotificationsEnabled(bool enabled) async {
    if (enabled &&
        !await ref
            .read(backgroundRefreshServiceProvider)
            .requestNotificationPermission()) {
      return false;
    }
    await _setBackground(notificationsEnabled: enabled);
    return true;
  }

  Future<void> setNotificationQuietHours(bool enabled) =>
      _setBackground(notificationQuietHours: enabled);

  Future<void> resetPreferences() async {
    const defaults = AppearanceState.defaults();
    await _saveAppearance(
        theme: defaults.theme,
        visual: defaults.visual,
        reading: defaults.reading);
    await setLanguage(defaults.language);
    await _setBackground(
      refreshIntervalMin: defaults.refreshIntervalMin,
      notificationsEnabled: defaults.notificationsEnabled,
      notificationQuietHours: defaults.notificationQuietHours,
    );
  }

  Future<void> _setBackground({
    int? refreshIntervalMin,
    bool? notificationsEnabled,
    bool? notificationQuietHours,
  }) async {
    final previous = state.asData?.value ?? const AppearanceState.defaults();
    final next = previous.copyWith(
      refreshIntervalMin: refreshIntervalMin,
      notificationsEnabled: notificationsEnabled,
      notificationQuietHours: notificationQuietHours,
    );
    state = AsyncData(next);
    try {
      await ref.read(settingsRepositoryProvider).setBackground(
            refreshIntervalMin: next.refreshIntervalMin,
            notificationsEnabled: next.notificationsEnabled,
            notificationQuietHours: next.notificationQuietHours,
          );
      await _reconcile(next.refreshIntervalMin);
    } catch (error, stackTrace) {
      try {
        await ref.read(settingsRepositoryProvider).setBackground(
              refreshIntervalMin: previous.refreshIntervalMin,
              notificationsEnabled: previous.notificationsEnabled,
              notificationQuietHours: previous.notificationQuietHours,
            );
        await _reconcile(previous.refreshIntervalMin);
      } catch (_) {
        // Preserve the original failure; startup reconciliation repairs the
        // schedule from persisted settings on the next app launch.
      }
      state = AsyncData((state.asData?.value ?? previous).copyWith(
        refreshIntervalMin: previous.refreshIntervalMin,
        notificationsEnabled: previous.notificationsEnabled,
        notificationQuietHours: previous.notificationQuietHours,
      ));
      Error.throwWithStackTrace(error, stackTrace);
    }
  }
}

class SettingsRepository {
  final Ref _ref;

  SettingsRepository(this._ref);

  Future<void> setAppearance(
      {required String theme,
      required bridge.VisualSettings visual,
      required bridge.ReadingSettings reading}) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.setAppearanceSettings(
          core: core, theme: theme, visual: visual, reading: reading);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

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

  Future<void> setBackground({
    required int refreshIntervalMin,
    required bool notificationsEnabled,
    required bool notificationQuietHours,
  }) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.setBackgroundSettings(
        core: core,
        refreshIntervalMin: refreshIntervalMin,
        notificationsEnabled: notificationsEnabled,
        notificationQuietHours: notificationQuietHours,
      );
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
