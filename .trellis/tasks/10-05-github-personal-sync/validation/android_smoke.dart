import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:workmanager/workmanager.dart';
import 'package:papr_mobile/app.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/bridge/generated/frb_generated.dart';
import 'package:papr_mobile/core/config.dart';
import 'package:papr_mobile/services/platform_service.dart';

const workerTask = 'papr.sync.smoke.once';
const workerRef = 'papr.sync.smoke_worker';
const marker = 'Papr emulator test marker';

@pragma('vm:entry-point')
void smokeDispatcher() {
  Workmanager().executeTask((name, _) async {
    if (name != workerTask) return true;
    DartPluginRegistrant.ensureInitialized();
    try {
      if (await platformService.getSyncCredential(workerRef) != marker) {
        throw StateError('worker credential round trip failed');
      }
      if (!await platformService.deleteSyncCredential(workerRef)) {
        throw StateError('worker credential cleanup failed');
      }
      await RustLib.init();
      final core = await bridge.initPaprCore(config: await buildCoreConfig());
      if ((await bridge.githubStatus(core: core)).profile != null) {
        throw StateError('test database unexpectedly connected');
      }
      debugPrint('PAPR_SYNC_SMOKE_WORKER_PASS');
      return true;
    } catch (_) {
      debugPrint('PAPR_SYNC_SMOKE_WORKER_FAIL');
      return false;
    }
  });
}

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  const foregroundRef = 'papr.sync.smoke_foreground';
  try {
    if (!await platformService.setSyncCredential(foregroundRef, marker)) {
      throw StateError('foreground credential write failed');
    }
    final value = await platformService.getSyncCredential(foregroundRef);
    final removed = await platformService.deleteSyncCredential(foregroundRef);
    if (value != marker || !removed)
      throw StateError('foreground credential round trip failed');
    await RustLib.init();
    final core = await bridge.initPaprCore(config: await buildCoreConfig());
    await bridge.setBackgroundSettings(
      core: core,
      refreshIntervalMin: 525600,
      notificationsEnabled: false,
      notificationQuietHours: false,
    );
    if ((await bridge.githubStatus(core: core)).profile != null) {
      throw StateError('test database unexpectedly connected');
    }
    if (!await platformService.setSyncCredential(workerRef, marker)) {
      throw StateError('worker marker write failed');
    }
    await Workmanager().initialize(smokeDispatcher);
    await Workmanager().registerOneOffTask(workerTask, workerTask);
    debugPrint('PAPR_SYNC_SMOKE_FOREGROUND_PASS');
    runApp(const ProviderScope(child: PaprApp()));
  } catch (_) {
    debugPrint('PAPR_SYNC_SMOKE_FOREGROUND_FAIL');
    runApp(const MaterialApp(home: Scaffold(body: Text('Sync smoke failed'))));
  }
}
