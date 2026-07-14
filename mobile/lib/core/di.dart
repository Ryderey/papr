import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../bridge/generated/generated.dart' as bridge;
import '../services/papr_core_service.dart';
import 'config.dart';

/// The async-initialised FRB core handle.
final paprCoreServiceProvider = FutureProvider<PaprCoreService>((ref) async {
  final config = await buildCoreConfig();
  final core = await bridge.initPaprCore(config: config);
  return PaprCoreService(core);
});

/// Raw core bridge, exposed as an AsyncValue for UI consumption.
final paprCoreBridgeProvider = FutureProvider<bridge.PaprCoreBridge>((ref) async {
  final service = await ref.watch(paprCoreServiceProvider.future);
  return service.bridge;
});
