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

  const AppearanceState({
    required this.theme,
    required this.language,
    required this.refreshIntervalMin,
  });

  const AppearanceState.defaults()
      : theme = 'system',
        language = 'en',
        refreshIntervalMin = 30;

  AppearanceState copyWith({String? theme, String? language}) {
    return AppearanceState(
      theme: theme ?? this.theme,
      language: language ?? this.language,
      refreshIntervalMin: refreshIntervalMin,
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
}
