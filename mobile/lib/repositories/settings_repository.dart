import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../core/di.dart';
import '../services/papr_core_service.dart';
import '../services/background_refresh_service.dart';

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

  const AppearanceState({
    required this.theme,
    required this.language,
    required this.refreshIntervalMin,
    required this.notificationsEnabled,
    required this.notificationQuietHours,
    required this.reading,
  });

  const AppearanceState.defaults()
      : theme = 'system',
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
  }) {
    return AppearanceState(
      theme: theme ?? this.theme,
      language: language ?? this.language,
      refreshIntervalMin: refreshIntervalMin ?? this.refreshIntervalMin,
      notificationsEnabled: notificationsEnabled ?? this.notificationsEnabled,
      notificationQuietHours:
          notificationQuietHours ?? this.notificationQuietHours,
      reading: reading ?? this.reading,
    );
  }
}

class AppearanceController extends AsyncNotifier<AppearanceState> {
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
    );
    try {
      await ref
          .read(backgroundRefreshServiceProvider)
          .reconcile(settings.refreshIntervalMin);
    } catch (_) {
      // Settings remain usable when Android temporarily rejects scheduling.
    }
    return settings;
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
      await ref
          .read(backgroundRefreshServiceProvider)
          .reconcile(next.refreshIntervalMin);
    } catch (error, stackTrace) {
      try {
        await ref.read(settingsRepositoryProvider).setBackground(
              refreshIntervalMin: previous.refreshIntervalMin,
              notificationsEnabled: previous.notificationsEnabled,
              notificationQuietHours: previous.notificationQuietHours,
            );
        await ref
            .read(backgroundRefreshServiceProvider)
            .reconcile(previous.refreshIntervalMin);
      } catch (_) {
        // Preserve the original failure; startup reconciliation repairs the
        // schedule from persisted settings on the next app launch.
      }
      state = AsyncData(previous);
      Error.throwWithStackTrace(error, stackTrace);
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
