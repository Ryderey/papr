import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../l10n/l10n.dart';
import '../../l10n/appearance.dart';
import '../../models/appearance_presets.dart';
import '../../repositories/settings_repository.dart';
import '../app_theme.dart';

class AppearanceScreen extends ConsumerWidget {
  const AppearanceScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appearance = ref.watch(appearanceProvider);
    final settings =
        appearance.asData?.value ?? const AppearanceState.defaults();
    final busy = appearance.isLoading || settings.savingAppearance;
    final controller = ref.read(appearanceProvider.notifier);
    final selected = matchingAppearancePreset(
        settings.theme, settings.visual, settings.reading);
    final l10n = context.l10n;
    if (appearance.hasError && !appearance.hasValue) {
      return Scaffold(
        appBar: AppBar(title: Text(l10n.appearanceTitle)),
        body: Center(
            child: Padding(
                padding: const EdgeInsets.all(20),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(l10n.localizeError(appearance.error!)),
                    const SizedBox(height: 12),
                    OutlinedButton(
                        onPressed: () => ref.invalidate(appearanceProvider),
                        child: Text(l10n.retry)),
                  ],
                ))),
      );
    }
    return Scaffold(
      appBar: AppBar(title: Text(l10n.appearanceTitle)),
      body: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          Text(l10n.appearancePresetsTitle,
              style: Theme.of(context).textTheme.headlineSmall),
          const SizedBox(height: 8),
          Text(l10n.appearancePresetsDescription,
              style: Theme.of(context).textTheme.bodyMedium),
          const SizedBox(height: 12),
          Semantics(
              liveRegion: true,
              child: Text(
                selected == null
                    ? l10n.appearanceCustom
                    : l10n.appearanceCurrent(l10n.presetName(selected)),
                style: TextStyle(color: Theme.of(context).colorScheme.primary),
              )),
          if (busy)
            const Padding(
                padding: EdgeInsets.only(top: 12),
                child: LinearProgressIndicator()),
          const SizedBox(height: 20),
          LayoutBuilder(builder: (context, constraints) {
            final columns = constraints.maxWidth >= 520 &&
                    MediaQuery.textScalerOf(context).scale(1) < 1.5
                ? 2
                : 1;
            final width = (constraints.maxWidth - (columns - 1) * 16) / columns;
            return Wrap(spacing: 16, runSpacing: 16, children: [
              for (final preset in appearancePresets)
                SizedBox(
                    width: width,
                    child: _PresetCard(
                      preset: preset,
                      selected: selected == preset.id,
                      onTap: busy
                          ? null
                          : () => _save(context,
                              () => controller.applyAppearancePreset(preset)),
                    )),
            ]);
          }),
          const SizedBox(height: 28),
          Text(l10n.appearanceAdjustments,
              style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 16),
          _choice(
              context,
              l10n.themeLabel,
              settings.theme,
              {
                'system': l10n.themeSystem,
                'light': l10n.themeLight,
                'dark': l10n.themeDark,
              },
              busy,
              (value) => controller.setTheme(value)),
          _choice(
              context,
              l10n.appearanceAccent,
              settings.visual.accent,
              {
                'clay': l10n.accentClay,
                'pine': l10n.accentPine,
                'indigo': l10n.accentIndigo,
                'ink': l10n.accentInk,
                'mauve': l10n.accentMauve,
                'frost': l10n.accentFrost,
                'leaf': l10n.accentLeaf,
                'amber': l10n.accentAmber,
                'rose': l10n.accentRose,
                'slate': l10n.accentSlate,
              },
              busy,
              (value) => controller.setVisual(
                  copyVisualSettings(settings.visual, accent: value))),
          if (settings.theme != 'light')
            _choice(
                context,
                l10n.appearanceDarkShade,
                settings.visual.darkShade,
                {
                  'default': l10n.darkShadeDefault,
                  'dimmer': l10n.darkShadeDimmer,
                  'black': l10n.darkShadeBlack,
                },
                busy,
                (value) => controller.setVisual(
                    copyVisualSettings(settings.visual, darkShade: value))),
          _choice(
              context,
              l10n.appearanceDensity,
              settings.visual.density,
              {
                'compact': l10n.densityCompact,
                'cozy': l10n.densityCozy,
                'spacious': l10n.densitySpacious,
              },
              busy,
              (value) => controller.setVisual(
                  copyVisualSettings(settings.visual, density: value))),
          _choice(
              context,
              l10n.appearanceListStyle,
              settings.visual.viewMode,
              {
                'list': l10n.viewModeList,
                'card': l10n.viewModeCard,
              },
              busy,
              (value) => controller.setVisual(
                  copyVisualSettings(settings.visual, viewMode: value))),
        ],
      ),
    );
  }

  Widget _choice(
          BuildContext context,
          String label,
          String value,
          Map<String, String> choices,
          bool busy,
          Future<void> Function(String) change) =>
      Padding(
          padding: const EdgeInsets.only(bottom: 16),
          child: DropdownButtonFormField<String>(
            key: ValueKey('$label-$value'),
            initialValue: value,
            isExpanded: true,
            decoration: InputDecoration(
                labelText: label, border: const OutlineInputBorder()),
            items: [
              for (final choice in choices.entries)
                DropdownMenuItem(value: choice.key, child: Text(choice.value))
            ],
            onChanged: busy
                ? null
                : (next) {
                    if (next != null) _save(context, () => change(next));
                  },
          ));

  Future<void> _save(
      BuildContext context, Future<void> Function() action) async {
    try {
      await action();
    } catch (error) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(
          content: Text(context.l10n
              .appearanceSaveFailed(context.l10n.localizeError(error))),
        ));
      }
    }
  }
}

class _PresetCard extends StatelessWidget {
  final AppearancePreset preset;
  final bool selected;
  final VoidCallback? onTap;
  const _PresetCard({required this.preset, required this.selected, this.onTap});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = Theme.of(context).colorScheme;
    return Semantics(
      key: ValueKey('appearance-preset-${preset.id}'),
      button: true,
      selected: selected,
      enabled: onTap != null,
      child: Material(
        color: colors.surfaceContainerLow,
        shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(16),
            side: BorderSide(
                color: selected ? colors.primary : colors.outlineVariant,
                width: selected ? 2 : 1)),
        clipBehavior: Clip.antiAlias,
        child: InkWell(
            onTap: onTap,
            child: Padding(
                padding: const EdgeInsets.all(12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    ExcludeSemantics(
                        child: Theme(
                      data: paprTheme(
                          preset.visual,
                          preset.theme == 'dark'
                              ? Brightness.dark
                              : Brightness.light),
                      child: _ReadingPreview(preset: preset),
                    )),
                    const SizedBox(height: 12),
                    Row(children: [
                      Expanded(
                          child: Text(l10n.presetName(preset.id),
                              style: Theme.of(context).textTheme.titleMedium)),
                      if (selected)
                        Icon(Icons.check_circle_outline, color: colors.primary),
                    ]),
                    const SizedBox(height: 4),
                    Text(l10n.presetDescription(preset.id)),
                  ],
                ))),
      ),
    );
  }
}

class _ReadingPreview extends StatelessWidget {
  final AppearancePreset preset;
  const _ReadingPreview({required this.preset});

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return Container(
      width: double.infinity,
      padding: EdgeInsets.all(preset.visual.density == 'spacious' ? 22 : 16),
      decoration: BoxDecoration(
          color: colors.surface, borderRadius: BorderRadius.circular(10)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [
          Text('Papr',
              style: TextStyle(
                  color: colors.primary, fontWeight: FontWeight.w600)),
          const Spacer(),
          Icon(Icons.bookmark_outline, size: 18, color: colors.primary),
        ]),
        const SizedBox(height: 12),
        Text(context.l10n.appearancePreviewTitle,
            style: TextStyle(
              fontFamily: preset.font == 'serif' ? 'serif' : 'sans-serif',
              fontSize: preset.fontSize + 5,
              height: 1.2,
              color: colors.onSurface,
              fontWeight: FontWeight.w600,
            )),
        const SizedBox(height: 10),
        Text(context.l10n.appearancePreviewBody,
            style: TextStyle(
              fontFamily: preset.font == 'serif' ? 'serif' : 'sans-serif',
              height: preset.lineHeight,
              color: colors.onSurfaceVariant,
            )),
      ]),
    );
  }
}
