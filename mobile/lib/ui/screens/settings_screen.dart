import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../l10n/l10n.dart';
import '../../repositories/settings_repository.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appearance = ref.watch(appearanceProvider);
    final settings =
        appearance.asData?.value ?? const AppearanceState.defaults();
    final l10n = context.l10n;

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.settingsTitle),
      ),
      body: Column(
        children: [
          if (appearance.isLoading) const LinearProgressIndicator(),
          Expanded(
            child: ListView(
              children: [
                ListTile(
                  title: Text(l10n.themeLabel),
                  trailing: DropdownButton<String>(
                    value: settings.theme,
                    onChanged: appearance.isLoading
                        ? null
                        : (value) {
                            if (value != null) {
                              _save(
                                context,
                                () => ref
                                    .read(appearanceProvider.notifier)
                                    .setTheme(value),
                              );
                            }
                          },
                    items: [
                      DropdownMenuItem(
                        value: 'system',
                        child: Text(l10n.themeSystem),
                      ),
                      DropdownMenuItem(
                        value: 'light',
                        child: Text(l10n.themeLight),
                      ),
                      DropdownMenuItem(
                        value: 'dark',
                        child: Text(l10n.themeDark),
                      ),
                    ],
                  ),
                ),
                ListTile(
                  title: Text(l10n.languageLabel),
                  trailing: DropdownButton<String>(
                    value: settings.language,
                    onChanged: appearance.isLoading
                        ? null
                        : (value) {
                            if (value != null) {
                              _save(
                                context,
                                () => ref
                                    .read(appearanceProvider.notifier)
                                    .setLanguage(value),
                              );
                            }
                          },
                    items: [
                      DropdownMenuItem(
                        value: 'en',
                        child: Text(l10n.languageEnglish),
                      ),
                      DropdownMenuItem(
                        value: 'zh',
                        child: Text(l10n.languageChinese),
                      ),
                      DropdownMenuItem(
                        value: 'ja',
                        child: Text(l10n.languageJapanese),
                      ),
                    ],
                  ),
                ),
                ListTile(
                  title: Text(l10n.refreshInterval),
                  subtitle: Text(l10n.minutes(settings.refreshIntervalMin)),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _save(BuildContext context, Future<void> Function() save) async {
    try {
      await save();
    } catch (error) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              context.l10n.appearanceSaveFailed(
                context.l10n.localizeError(error),
              ),
            ),
          ),
        );
      }
    }
  }
}
