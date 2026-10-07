import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/core/exceptions.dart';
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/repositories/github_sync_repository.dart';
import 'package:papr_mobile/ui/screens/github_schedule_controls.dart';

class _ScheduleState {
  bridge.GithubSchedule value = const bridge.GithubSchedule(
      enabled: true,
      uploadDelaySecs: 30,
      cloudIntervalMinutes: 10,
      backgroundIntervalMinutes: 60);
  bool fail = false;
  int saves = 0;
  bool ignoreWrite = false;
  bool failRead = false;
}

class _Repository extends GithubSyncRepository {
  final Ref reference;
  final _ScheduleState values;
  _Repository(this.reference, this.values) : super(reference);
  @override
  Future<bridge.GithubSchedule> schedule() async {
    if (values.failRead) {
      throw const AppException(AppErrorKind.sync, 'githubNetwork', null);
    }
    return values.value;
  }

  @override
  Future<void> setSchedule(bridge.GithubSchedule value) async {
    values.saves++;
    if (values.fail) {
      throw const AppException(
          AppErrorKind.sync, 'githubInvalidSchedule', null);
    }
    if (!values.ignoreWrite) values.value = value;
    reference.invalidate(githubScheduleProvider);
  }
}

Future<void> _mount(WidgetTester tester, _ScheduleState values) async {
  await tester.pumpWidget(ProviderScope(
      overrides: [
        githubSyncRepositoryProvider
            .overrideWith((ref) => _Repository(ref, values)),
      ],
      child: MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          locale: const Locale('en'),
          home: const Scaffold(
              body: SingleChildScrollView(child: GithubScheduleControls())))));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('read retry preserves subsequent edits without writing them',
      (tester) async {
    final values = _ScheduleState();
    await _mount(tester, values);
    final choice = find.byType(DropdownButtonFormField<int>).first;
    tester.widget<DropdownButtonFormField<int>>(choice).onChanged!(60);
    await tester.pumpAndSettle();
    values.failRead = true;
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    tester.widget<DropdownButtonFormField<int>>(choice).onChanged!(120);
    await tester.pumpAndSettle();
    values.failRead = false;
    await tester.tap(find.text('Retry'));
    await tester.pumpAndSettle();
    expect(values.saves, 1);
    expect(values.value.uploadDelaySecs, 60);
    expect(
        tester.widget<DropdownButtonFormField<int>>(choice).initialValue, 120);
    expect(tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNotNull);
    expect(find.text('Retry'), findsNothing);
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(values.value.uploadDelaySecs, 120);
    expect(find.text('Sync settings saved'), findsOneWidget);
  });

  testWidgets(
      'readback failure keeps draft controls and retries on the same page',
      (tester) async {
    final values = _ScheduleState();
    await _mount(tester, values);
    final choice = find.byType(DropdownButtonFormField<int>).first;
    tester.widget<DropdownButtonFormField<int>>(choice).onChanged!(60);
    await tester.pumpAndSettle();
    values.failRead = true;
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(values.value.uploadDelaySecs, 60);
    expect(find.byType(DropdownButtonFormField<int>), findsNWidgets(3));
    expect(
        tester.widget<DropdownButtonFormField<int>>(choice).initialValue, 60);
    expect(tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNotNull);
    expect(find.text('Sync settings saved'), findsNothing);
    values.failRead = false;
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(find.text('Sync settings saved'), findsOneWidget);
    expect(tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNull);
  });

  testWidgets(
      'edits do not persist until Save, then manual mode survives reopening',
      (tester) async {
    final values = _ScheduleState();
    await _mount(tester, values);
    expect(tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNull);
    await tester.tap(find.byType(Switch));
    await tester.pumpAndSettle();
    expect(values.saves, 0);
    expect(values.value.enabled, isTrue);
    expect(tester.widget<Switch>(find.byType(Switch)).value, isFalse);
    expect(
        find.text('Changes take effect after you tap Save.'), findsOneWidget);
    for (final choice in tester.widgetList<DropdownButtonFormField<int>>(
        find.byType(DropdownButtonFormField<int>))) {
      expect(choice.onChanged, isNull);
    }
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(values.saves, 1);
    expect(values.value.enabled, isFalse);
    expect(values.value.uploadDelaySecs, 30);
    expect(values.value.backgroundIntervalMinutes, 60);
    expect(find.text('Sync settings saved'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
    await _mount(tester, values);
    expect(tester.widget<Switch>(find.byType(Switch)).value, isFalse);
    expect(find.text('Automatic sync is off. Use Sync now when needed.'),
        findsOneWidget);
    expect(tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNull);
  });

  testWidgets(
      'failed save retains draft for retry and leaves persisted mode unchanged',
      (tester) async {
    final values = _ScheduleState()..fail = true;
    await _mount(tester, values);
    await tester.tap(find.byType(Switch));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(values.value.enabled, isTrue);
    expect(tester.widget<Switch>(find.byType(Switch)).value, isFalse);
    expect(find.text('Invalid sync scheduling settings.'), findsOneWidget);
    expect(tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNotNull);
    values.fail = false;
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(values.value.enabled, isFalse);
    expect(find.text('Invalid sync scheduling settings.'), findsNothing);
  });

  testWidgets(
      'multiple interval edits save together and 60 seconds survives reopening',
      (tester) async {
    final values = _ScheduleState();
    await _mount(tester, values);
    final choices = find.byType(DropdownButtonFormField<int>);
    tester.widget<DropdownButtonFormField<int>>(choices.at(0)).onChanged!(60);
    await tester.pumpAndSettle();
    tester.widget<DropdownButtonFormField<int>>(choices.at(1)).onChanged!(30);
    await tester.pumpAndSettle();
    tester.widget<DropdownButtonFormField<int>>(choices.at(2)).onChanged!(120);
    await tester.pumpAndSettle();
    expect(values.saves, 0);
    expect(values.value.uploadDelaySecs, 30);
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(values.saves, 1);
    expect(values.value.uploadDelaySecs, 60);
    expect(values.value.cloudIntervalMinutes, 30);
    expect(values.value.backgroundIntervalMinutes, 120);
    await tester.pumpWidget(const SizedBox());
    await _mount(tester, values);
    expect(
        tester.widget<DropdownButtonFormField<int>>(choices.at(0)).initialValue,
        60);
    expect(
        tester.widget<DropdownButtonFormField<int>>(choices.at(1)).initialValue,
        30);
    expect(
        tester.widget<DropdownButtonFormField<int>>(choices.at(2)).initialValue,
        120);
    expect(tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNull);
  });

  testWidgets('closing unsaved edits discards them; reverting disables Save',
      (tester) async {
    final values = _ScheduleState();
    await _mount(tester, values);
    final choice = find.byType(DropdownButtonFormField<int>).first;
    tester.widget<DropdownButtonFormField<int>>(choice).onChanged!(60);
    await tester.pumpAndSettle();
    tester.widget<DropdownButtonFormField<int>>(choice).onChanged!(30);
    await tester.pumpAndSettle();
    expect(tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNull);
    tester.widget<DropdownButtonFormField<int>>(choice).onChanged!(60);
    await tester.pumpAndSettle();
    await tester.pumpWidget(const SizedBox());
    await _mount(tester, values);
    expect(
        tester.widget<DropdownButtonFormField<int>>(choice).initialValue, 30);
    expect(values.saves, 0);
  });

  testWidgets('unconfirmed persistence shows error and retains draft',
      (tester) async {
    final values = _ScheduleState()..ignoreWrite = true;
    await _mount(tester, values);
    final choice = find.byType(DropdownButtonFormField<int>).first;
    tester.widget<DropdownButtonFormField<int>>(choice).onChanged!(60);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(values.value.uploadDelaySecs, 30);
    expect(
        tester.widget<DropdownButtonFormField<int>>(choice).initialValue, 60);
    expect(find.text('Sync settings were not saved. Please retry.'),
        findsOneWidget);
    expect(find.text('Sync settings saved'), findsNothing);
    expect(tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNotNull);
  });
}
