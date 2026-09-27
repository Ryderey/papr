import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/repositories/settings_repository.dart';
import 'package:papr_mobile/repositories/sync_repository.dart';
import 'package:papr_mobile/ui/screens/settings_screen.dart';

void main() {
  testWidgets('exposes AI profiles but not removed tag management',
      (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          appearanceProvider.overrideWith(_TestAppearanceController.new),
          syncStatusProvider.overrideWith(
            (ref) async => const bridge.SyncStatus(backgroundDue: false),
          ),
        ],
        child: const MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: SettingsScreen(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Manage tags'), findsNothing);
    expect(find.text('AI profiles'), findsOneWidget);
    expect(find.text('Reader sync'), findsOneWidget);
    expect(find.text('Manage rules'), findsOneWidget);
    expect(find.text('Background refresh'), findsOneWidget);
    expect(find.text('New article notifications'), findsOneWidget);
    expect(find.text('Night quiet hours'), findsOneWidget);
    await tester.ensureVisible(find.text('Reader sync'));
    await tester.tap(find.text('Reader sync'));
    await tester.pumpAndSettle();
    expect(find.text('No reader service connected.'), findsOneWidget);
    expect(find.text('FreshRSS'), findsOneWidget);
    expect(find.text('Miniflux'), findsNothing);
    expect(find.text('Connect'), findsOneWidget);
  });

  testWidgets('reset and clear require confirmation', (tester) async {
    const channel = MethodChannel('com.papr.papr_mobile/platform');
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    var clearCalls = 0;
    messenger.setMockMethodCallHandler(channel, (call) async {
      if (call.method == 'clearApplicationData') clearCalls += 1;
      return true;
    });
    addTearDown(() => messenger.setMockMethodCallHandler(channel, null));
    final appearance = _TestAppearanceController();
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          appearanceProvider.overrideWith(() => appearance),
        ],
        child: const MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: SettingsScreen(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.scrollUntilVisible(find.text('Reset preferences'), 300);
    await tester.tap(find.text('Reset preferences'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    expect(appearance.resetCalls, 0);

    await tester.tap(find.text('Reset preferences'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Reset preferences'));
    await tester.pumpAndSettle();
    expect(appearance.resetCalls, 1);

    await tester.pump(const Duration(seconds: 5));
    await tester.pumpAndSettle();
    await tester.scrollUntilVisible(find.text('Clear all data'), 300);
    await tester.tap(find.text('Clear all data'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    expect(clearCalls, 0);

    await tester.tap(find.text('Clear all data'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Clear all data'));
    await tester.pumpAndSettle();
    expect(clearCalls, 1);
  });

  testWidgets('settings fit a narrow screen with large Japanese text',
      (tester) async {
    await tester.binding.setSurfaceSize(const Size(320, 640));
    addTearDown(() => tester.binding.setSurfaceSize(null));

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          appearanceProvider.overrideWith(_TestAppearanceController.new),
        ],
        child: MaterialApp(
          locale: const Locale('ja'),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          builder: (context, child) => MediaQuery(
            data: MediaQuery.of(context).copyWith(
              textScaler: const TextScaler.linear(2),
            ),
            child: child!,
          ),
          home: const SettingsScreen(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(tester.takeException(), isNull);
    await tester.scrollUntilVisible(find.byIcon(Icons.delete_forever_outlined), 300);
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
  });
}

class _TestAppearanceController extends AppearanceController {
  int resetCalls = 0;

  @override
  Future<AppearanceState> build() async => const AppearanceState.defaults();

  @override
  Future<void> resetPreferences() async {
    resetCalls += 1;
  }
}
