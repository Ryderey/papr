import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/repositories/settings_repository.dart';
import 'package:papr_mobile/ui/screens/settings_screen.dart';

void main() {
  testWidgets('exposes AI profiles but not removed tag management',
      (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          appearanceProvider.overrideWith(_TestAppearanceController.new),
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
    expect(find.text('Manage rules'), findsOneWidget);
    expect(find.text('Background refresh'), findsOneWidget);
    expect(find.text('New article notifications'), findsOneWidget);
    expect(find.text('Night quiet hours'), findsOneWidget);
  });
}

class _TestAppearanceController extends AppearanceController {
  @override
  Future<AppearanceState> build() async => const AppearanceState.defaults();
}
