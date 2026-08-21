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
                const Divider(),
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 16, 16, 4),
                  child: Text(
                    l10n.readingSettings,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
                ListTile(
                  title: Text(l10n.readerFont),
                  trailing: DropdownButton<String>(
                    value: settings.reading.font,
                    onChanged: appearance.isLoading
                        ? null
                        : (value) {
                            if (value != null) {
                              _save(
                                context,
                                () => ref
                                    .read(appearanceProvider.notifier)
                                    .setReading(
                                      copyReadingSettings(
                                        settings.reading,
                                        font: value,
                                      ),
                                    ),
                              );
                            }
                          },
                    items: [
                      DropdownMenuItem(
                        value: 'system',
                        child: Text(l10n.fontSystem),
                      ),
                      DropdownMenuItem(
                        value: 'serif',
                        child: Text(l10n.fontSerif),
                      ),
                      DropdownMenuItem(
                        value: 'sans',
                        child: Text(l10n.fontSans),
                      ),
                    ],
                  ),
                ),
                _ReadingSlider(
                  label: l10n.fontSize,
                  value: settings.reading.fontSize,
                  min: 14,
                  max: 24,
                  onChanged: (value) => _save(
                    context,
                    () => ref.read(appearanceProvider.notifier).setReading(
                          copyReadingSettings(
                            settings.reading,
                            fontSize: value,
                          ),
                        ),
                  ),
                ),
                _ReadingSlider(
                  label: l10n.lineHeight,
                  value: settings.reading.lineHeight,
                  min: 1.3,
                  max: 2,
                  divisions: 14,
                  onChanged: (value) => _save(
                    context,
                    () => ref.read(appearanceProvider.notifier).setReading(
                          copyReadingSettings(
                            settings.reading,
                            lineHeight: value,
                          ),
                        ),
                  ),
                ),
                _ReadingSlider(
                  label: l10n.readingWidth,
                  value: settings.reading.contentWidth,
                  min: 320,
                  max: 840,
                  divisions: 13,
                  onChanged: (value) => _save(
                    context,
                    () => ref.read(appearanceProvider.notifier).setReading(
                          copyReadingSettings(
                            settings.reading,
                            contentWidth: value,
                          ),
                        ),
                  ),
                ),
                SwitchListTile(
                  title: Text(l10n.showReadingTime),
                  value: settings.reading.showReadingTime,
                  onChanged: appearance.isLoading
                      ? null
                      : (value) => _save(
                            context,
                            () => ref
                                .read(appearanceProvider.notifier)
                                .setReading(
                                  copyReadingSettings(
                                    settings.reading,
                                    showReadingTime: value,
                                  ),
                                ),
                          ),
                ),
                SwitchListTile(
                  title: Text(l10n.autoExtractFulltext),
                  subtitle: Text(l10n.autoExtractFulltextDescription),
                  value: settings.reading.autoExtract,
                  onChanged: appearance.isLoading
                      ? null
                      : (value) => _save(
                            context,
                            () => ref
                                .read(appearanceProvider.notifier)
                                .setReading(
                                  copyReadingSettings(
                                    settings.reading,
                                    autoExtract: value,
                                  ),
                                ),
                          ),
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

class _ReadingSlider extends StatefulWidget {
  final String label;
  final double value;
  final double min;
  final double max;
  final int? divisions;
  final ValueChanged<double> onChanged;

  const _ReadingSlider({
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    this.divisions,
    required this.onChanged,
  });

  @override
  State<_ReadingSlider> createState() => _ReadingSliderState();
}

class _ReadingSliderState extends State<_ReadingSlider> {
  late double _value = widget.value;

  @override
  void didUpdateWidget(covariant _ReadingSlider oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.value != widget.value) _value = widget.value;
  }

  @override
  Widget build(BuildContext context) {
    return ListTile(
      title: Text(widget.label),
      subtitle: Slider(
        value: _value.clamp(widget.min, widget.max),
        min: widget.min,
        max: widget.max,
        divisions: widget.divisions,
        label: _value.toStringAsFixed(_value < 10 ? 1 : 0),
        onChanged: (value) => setState(() => _value = value),
        onChangeEnd: widget.onChanged,
      ),
    );
  }
}
