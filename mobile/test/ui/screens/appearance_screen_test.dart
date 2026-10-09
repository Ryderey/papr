import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/core/di.dart';
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/models/appearance_presets.dart';
import 'package:papr_mobile/repositories/settings_repository.dart';
import 'package:papr_mobile/ui/app_theme.dart';
import 'package:papr_mobile/ui/screens/appearance_screen.dart';

AppearancePreset _preset(String id) =>
    appearancePresets.firstWhere((preset) => preset.id == id);

void main() {
  testWidgets(
      'all five manual appearance controls persist without a save error',
      (tester) async {
    const initial = AppearanceState.defaults();
    final repository = _MemoryRepository(initial);
    await tester.pumpWidget(ProviderScope(overrides: [
      appearanceProvider.overrideWith(() => _InitialAppearance(initial)),
      settingsRepositoryProvider.overrideWith((ref) => repository),
    ], child: const _Harness()));
    await tester.pumpAndSettle();
    for (final choice in [
      ('Theme', 'System', 'Dark'),
      ('Accent color', 'Clay', 'Pine'),
      ('Dark background', 'Default', 'Dimmer'),
      ('Density', 'Cozy', 'Compact'),
      ('Article list style', 'Card', 'List'),
    ]) {
      await tester.scrollUntilVisible(find.text(choice.$1), 400);
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.text(choice.$2).last);
      await tester.pumpAndSettle();
      await tester.tap(find.text(choice.$2).last);
      await tester.pumpAndSettle();
      await tester.tap(find.text(choice.$3).last);
      await tester.pumpAndSettle();
      expect(find.textContaining('Could not save appearance'), findsNothing);
      expect(tester.takeException(), isNull);
    }
    expect(repository.calls, 5);
    expect(repository.snapshot.theme, 'dark');
    expect(repository.snapshot.visual.accent, 'pine');
    expect(repository.snapshot.visual.darkShade, 'dimmer');
    expect(repository.snapshot.visual.density, 'compact');
    expect(repository.snapshot.visual.viewMode, 'list');
  });
  testWidgets('initial read failure offers retry without applying defaults',
      (tester) async {
    final repository = _MemoryRepository(const AppearanceState.defaults());
    await tester.pumpWidget(ProviderScope(overrides: [
      appearanceProvider.overrideWith(_FailingAppearance.new),
      settingsRepositoryProvider.overrideWith((ref) => repository),
    ], child: const _Harness()));
    await tester.pumpAndSettle();
    expect(find.text('Retry'), findsOneWidget);
    expect(find.byKey(const ValueKey('appearance-preset-paper')), findsNothing);
    expect(repository.calls, 0);
    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();
    expect(repository.calls, 0);
    expect(find.text('Retry'), findsOneWidget);
  });
  test('all presets preserve reading behavior and save the complete appearance',
      () async {
    const defaults = AppearanceState.defaults();
    final initial = defaults.copyWith(
        reading: copyReadingSettings(defaults.reading,
            autoExtract: true, showReadingTime: false));
    final repository = _MemoryRepository(initial);
    final container = _container(repository, initial);
    addTearDown(container.dispose);
    await container.read(appearanceProvider.future);
    for (final preset in appearancePresets) {
      await container
          .read(appearanceProvider.notifier)
          .applyAppearancePreset(preset);
      final saved = container.read(appearanceProvider).requireValue;
      expect(matchingAppearancePreset(saved.theme, saved.visual, saved.reading),
          preset.id);
      expect(saved.reading.autoExtract, isTrue);
      expect(saved.reading.showReadingTime, isFalse);
      expect(repository.snapshot.visual, preset.visual);
      expect(repository.snapshot.reading, saved.reading);
    }
    expect(repository.calls, appearancePresets.length);
  });

  test('failed save retains appearance and does not poison the next save',
      () async {
    const initial = AppearanceState.defaults();
    final repository = _MemoryRepository(initial)..fail = true;
    final container = _container(repository, initial);
    addTearDown(container.dispose);
    await container.read(appearanceProvider.future);
    final controller = container.read(appearanceProvider.notifier);
    await expectLater(controller.applyAppearancePreset(_preset('dusk')),
        throwsStateError);
    final unchanged = container.read(appearanceProvider).requireValue;
    expect(unchanged.theme, initial.theme);
    expect(unchanged.visual, initial.visual);
    expect(unchanged.reading, initial.reading);
    expect(unchanged.savingAppearance, isFalse);
    repository.fail = false;
    await controller.applyAppearancePreset(_preset('dusk'));
    expect(container.read(appearanceProvider).requireValue.theme, 'dark');
  });

  test('queued preset preserves a reading behavior change saved ahead of it',
      () async {
    const initial = AppearanceState.defaults();
    final repository = _MemoryRepository(initial);
    final container = _container(repository, initial);
    addTearDown(container.dispose);
    await container.read(appearanceProvider.future);
    final controller = container.read(appearanceProvider.notifier);
    final behavior = controller.setReading(copyReadingSettings(initial.reading,
        autoExtract: true, showReadingTime: false));
    final preset = controller.applyAppearancePreset(_preset('midnight'));
    await Future.wait([behavior, preset]);
    final saved = container.read(appearanceProvider).requireValue;
    expect(saved.reading.autoExtract, isTrue);
    expect(saved.reading.showReadingTime, isFalse);
    expect(matchingAppearancePreset(saved.theme, saved.visual, saved.reading),
        'midnight');
  });

  test('quick changes serialize and reopening reads the persisted appearance',
      () async {
    const initial = AppearanceState.defaults();
    final repository = _MemoryRepository(initial)..gate = Completer<void>();
    final container = _container(repository, initial);
    await container.read(appearanceProvider.future);
    final controller = container.read(appearanceProvider.notifier);
    final first = controller.applyAppearancePreset(appearancePresets[1]);
    final second = controller.applyAppearancePreset(_preset('midnight'));
    await Future<void>.delayed(Duration.zero);
    expect(repository.calls, 1);
    expect(container.read(appearanceProvider).requireValue.savingAppearance,
        isTrue);
    expect(container.read(appearanceProvider).requireValue.theme, 'system');
    repository.gate!.complete();
    await Future.wait([first, second]);
    expect(repository.calls, 2);
    container.dispose();
    final reopened = ProviderContainer(overrides: [
      settingsRepositoryProvider.overrideWith((ref) => repository),
      paprCoreBridgeProvider.overrideWith((ref) async =>
          throw StateError('No background scheduler in this test')),
    ]);
    addTearDown(reopened.dispose);
    final saved = await reopened.read(appearanceProvider.future);
    expect(matchingAppearancePreset(saved.theme, saved.visual, saved.reading),
        'midnight');
  });

  test(
      'manual edits become custom and dormant light-mode dark shade is ignored',
      () async {
    const initial = AppearanceState.defaults();
    final repository = _MemoryRepository(initial);
    final container = _container(repository, initial);
    addTearDown(container.dispose);
    await container.read(appearanceProvider.future);
    final controller = container.read(appearanceProvider.notifier);
    await controller.applyAppearancePreset(appearancePresets.first);
    await controller.setVisual(
        copyVisualSettings(appearancePresets.first.visual, darkShade: 'black'));
    var saved = container.read(appearanceProvider).requireValue;
    expect(matchingAppearancePreset(saved.theme, saved.visual, saved.reading),
        'paper');
    await controller
        .setReading(copyReadingSettings(saved.reading, fontSize: 20));
    saved = container.read(appearanceProvider).requireValue;
    expect(matchingAppearancePreset(saved.theme, saved.visual, saved.reading),
        isNull);
  });

  testWidgets(
      'selects a preset and updates the actual theme and selected state',
      (tester) async {
    await tester.runAsync(_loadPreviewFonts);
    await tester.binding.setSurfaceSize(const Size(390, 844));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    const initial = AppearanceState.defaults();
    final repository = _MemoryRepository(initial);
    await tester.pumpWidget(ProviderScope(overrides: [
      appearanceProvider.overrideWith(() => _InitialAppearance(initial)),
      settingsRepositoryProvider.overrideWith((ref) => repository),
    ], child: const _Harness()));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const ValueKey('appearance-preset-paper')));
    await tester.pumpAndSettle();
    expect(find.text('Current: Paper'), findsOneWidget);
    expect(repository.calls, 1);
    await _screenshot(tester, 'android-appearance-light');
    final dusk = find.byKey(const ValueKey('appearance-preset-dusk'));
    await tester.scrollUntilVisible(dusk, 300);
    await tester.pumpAndSettle();
    await tester.tap(dusk);
    await tester.pumpAndSettle();
    expect(repository.snapshot.theme, 'dark');
    expect(Theme.of(tester.element(dusk)).brightness, Brightness.dark);
    expect(tester.getSemantics(dusk).flagsCollection.isSelected,
        ui.Tristate.isTrue);
    await _screenshot(tester, 'android-appearance-dark');
    expect(tester.takeException(), isNull);
  });

  testWidgets(
      'all localized presets fit narrow screens with large Japanese text',
      (tester) async {
    await tester.binding.setSurfaceSize(const Size(320, 640));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    const initial = AppearanceState.defaults();
    await tester.pumpWidget(ProviderScope(overrides: [
      appearanceProvider.overrideWith(() => _InitialAppearance(initial)),
    ], child: const _Harness(locale: Locale('ja'), scale: 2)));
    await tester.pumpAndSettle();
    for (final preset in appearancePresets) {
      await tester.scrollUntilVisible(
          find.byKey(ValueKey('appearance-preset-${preset.id}')), 250);
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
    }
    await tester.scrollUntilVisible(find.text('細かな調整'), 250);
    await tester.pumpAndSettle();
    expect(find.text('細かな調整'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}

ProviderContainer _container(
        _MemoryRepository repository, AppearanceState initial) =>
    ProviderContainer(overrides: [
      appearanceProvider.overrideWith(() => _InitialAppearance(initial)),
      settingsRepositoryProvider.overrideWith((ref) => repository),
    ]);

class _InitialAppearance extends AppearanceController {
  final AppearanceState initial;
  _InitialAppearance(this.initial);
  @override
  Future<AppearanceState> build() async => initial;
}

class _FailingAppearance extends AppearanceController {
  @override
  Future<AppearanceState> build() async =>
      throw StateError('Settings read failed');
}

class _MemoryRepository implements SettingsRepository {
  bridge.SettingsSnapshot snapshot;
  bool fail = false;
  int calls = 0;
  Completer<void>? gate;
  _MemoryRepository(AppearanceState state)
      : snapshot = bridge.SettingsSnapshot(
          theme: state.theme,
          language: state.language,
          refreshIntervalMin: state.refreshIntervalMin,
          notificationsEnabled: state.notificationsEnabled,
          notificationQuietHours: state.notificationQuietHours,
          reading: state.reading,
          visual: state.visual,
        );
  @override
  dynamic noSuchMethod(Invocation invocation) => throw UnimplementedError(
      'Unexpected repository call: ${invocation.memberName}');
  @override
  Future<bridge.SettingsSnapshot> getSettings() async => snapshot;
  @override
  Future<void> setAppearance(
      {required String theme,
      required bridge.VisualSettings visual,
      required bridge.ReadingSettings reading}) async {
    calls++;
    if (gate != null) await gate!.future;
    if (fail) throw StateError('Appearance save failed');
    snapshot = bridge.SettingsSnapshot(
        theme: theme,
        language: snapshot.language,
        refreshIntervalMin: snapshot.refreshIntervalMin,
        notificationsEnabled: snapshot.notificationsEnabled,
        notificationQuietHours: snapshot.notificationQuietHours,
        reading: reading,
        visual: visual);
  }
}

class _Harness extends ConsumerWidget {
  final Locale locale;
  final double scale;
  const _Harness({this.locale = const Locale('en'), this.scale = 1});
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(appearanceProvider).asData?.value ??
        const AppearanceState.defaults();
    return MaterialApp(
        locale: locale,
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        theme: paprTheme(state.visual, Brightness.light),
        darkTheme: paprTheme(state.visual, Brightness.dark),
        themeMode: state.theme == 'dark' ? ThemeMode.dark : ThemeMode.light,
        builder: (context, child) => RepaintBoundary(
            key: const ValueKey('appearance-capture'),
            child: MediaQuery(
                data: MediaQuery.of(context)
                    .copyWith(textScaler: TextScaler.linear(scale)),
                child: child!)),
        home: const AppearanceScreen());
  }
}

Future<void> _screenshot(WidgetTester tester, String name) async {
  const directory = String.fromEnvironment('PAPR_APPEARANCE_SCREENSHOT_DIR');
  if (directory.isEmpty) return;
  final boundary = tester.renderObject<RenderRepaintBoundary>(
      find.byKey(const ValueKey('appearance-capture')));
  await tester.runAsync(() async {
    final image = await boundary.toImage(pixelRatio: 2);
    final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
    await File('$directory/$name.png')
        .writeAsBytes(bytes!.buffer.asUint8List());
    image.dispose();
  });
}

Future<void> _loadPreviewFonts() async {
  const directory = String.fromEnvironment('PAPR_APPEARANCE_FONT_DIR');
  if (directory.isEmpty) return;
  for (final entry in {
    'Roboto': 'roboto-regular.ttf',
    'sans-serif': 'roboto-regular.ttf',
    'MaterialIcons': 'materialicons-regular.otf',
  }.entries) {
    final bytes = await File('$directory/${entry.value}').readAsBytes();
    final loader = FontLoader(entry.key)
      ..addFont(Future.value(ByteData.sublistView(bytes)));
    await loader.load();
  }
  const serif = String.fromEnvironment('PAPR_APPEARANCE_SERIF_FONT');
  if (serif.isNotEmpty) {
    final bytes = await File(serif).readAsBytes();
    final loader = FontLoader('serif')
      ..addFont(Future.value(ByteData.sublistView(bytes)));
    await loader.load();
  }
}
