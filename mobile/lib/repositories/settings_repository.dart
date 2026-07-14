import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../core/di.dart';
import '../services/papr_core_service.dart';

final settingsRepositoryProvider = Provider<SettingsRepository>((ref) {
  return SettingsRepository(ref);
});

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
}
