import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../repositories/settings_repository.dart';

final settingsProvider = FutureProvider<bridge.SettingsSnapshot>((ref) async {
  final repo = ref.watch(settingsRepositoryProvider);
  return repo.getSettings();
});

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(settingsProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
      ),
      body: settings.when(
        data: (snapshot) => ListView(
          children: [
            ListTile(
              title: const Text('Theme'),
              subtitle: Text(snapshot.theme),
            ),
            ListTile(
              title: const Text('Language'),
              subtitle: Text(snapshot.language),
            ),
            ListTile(
              title: const Text('Refresh interval'),
              subtitle: Text('${snapshot.refreshIntervalMin} min'),
            ),
          ],
        ),
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, stack) => Center(child: Text('Error: $err')),
      ),
    );
  }
}
