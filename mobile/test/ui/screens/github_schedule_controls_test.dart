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
}

class _Repository extends GithubSyncRepository {
  final Ref reference;
  final _ScheduleState values;
  _Repository(this.reference, this.values) : super(reference);
  @override
  Future<bridge.GithubSchedule> schedule() async => values.value;
  @override
  Future<void> setSchedule(bridge.GithubSchedule value) async {
    if (values.fail) {
      throw const AppException(
          AppErrorKind.sync, 'githubInvalidSchedule', null);
    }
    values.value = value;
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
  testWidgets(
      'manual mode preserves intervals and disables automatic selectors',
      (tester) async {
    final values = _ScheduleState();
    await _mount(tester, values);
    await tester.tap(find.byType(Switch));
    final container = ProviderScope.containerOf(
        tester.element(find.byType(GithubScheduleControls)));
    expect(
        (await container.read(githubScheduleProvider.future)).enabled, isFalse);
    await tester.pumpAndSettle();
    expect(values.value.enabled, isFalse);
    expect(tester.widget<Switch>(find.byType(Switch)).value, isFalse);
    expect(values.value.uploadDelaySecs, 30);
    expect(values.value.backgroundIntervalMinutes, 60);
    for (final choice in tester.widgetList<DropdownButtonFormField<int>>(
        find.byType(DropdownButtonFormField<int>))) {
      expect(choice.onChanged, isNull);
    }
    expect(find.text('Automatic sync is off. Use Sync now when needed.'),
        findsOneWidget);
  });
  testWidgets(
      'failed setting save keeps the previous automatic mode and reports an error',
      (tester) async {
    final values = _ScheduleState()..fail = true;
    await _mount(tester, values);
    await tester.tap(find.byType(Switch));
    await tester.pumpAndSettle();
    expect(values.value.enabled, isTrue);
    expect(tester.widget<Switch>(find.byType(Switch)).value, isTrue);
    expect(find.text('Invalid sync scheduling settings.'), findsOneWidget);
  });

  testWidgets(
      'interval save preserves other settings and failed change restores selection',
      (tester) async {
    final values = _ScheduleState();
    await _mount(tester, values);
    final choices = find.byType(DropdownButtonFormField<int>);
    tester.widget<DropdownButtonFormField<int>>(choices.first).onChanged!(120);
    await tester.pumpAndSettle();
    expect(values.value.uploadDelaySecs, 120);
    expect(values.value.cloudIntervalMinutes, 10);
    expect(
        tester.widget<DropdownButtonFormField<int>>(choices.first).initialValue,
        120);
    values.fail = true;
    tester.widget<DropdownButtonFormField<int>>(choices.first).onChanged!(60);
    await tester.pumpAndSettle();
    expect(values.value.uploadDelaySecs, 120);
    expect(
        tester.widget<DropdownButtonFormField<int>>(choices.first).initialValue,
        120);
    expect(find.text('Invalid sync scheduling settings.'), findsOneWidget);
  });
}
