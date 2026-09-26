import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
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
}

class _TestAppearanceController extends AppearanceController {
  @override
  Future<AppearanceState> build() async => const AppearanceState.defaults();
}
