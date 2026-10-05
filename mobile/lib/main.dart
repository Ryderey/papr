import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'app.dart';
import 'bridge/generated/frb_generated.dart';
import 'services/background_refresh_service.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await BackgroundRefreshService.initialize();
  await RustLib.init();
  runApp(const ProviderScope(child: PaprApp()));
}
