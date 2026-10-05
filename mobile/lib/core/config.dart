import 'dart:io';

import 'package:path_provider/path_provider.dart';

import '../../bridge/generated/generated.dart' as bridge;

/// Builds the runtime configuration for `papr-core` on this platform.
Future<bridge.PaprCoreConfig> buildCoreConfig() async {
  final docs = await getApplicationDocumentsDirectory();
  final dataDir = Directory('${docs.path}/papr');
  await dataDir.create(recursive: true);

  return bridge.PaprCoreConfig(
    dataDir: dataDir.path,
    databasePath: '${dataDir.path}/papr.db',
    cacheDir: '${dataDir.path}/cache',
    logDir: '${dataDir.path}/logs',
    logLevel: 'info',
    platform: bridge.Platform.android,
  );
}
