import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/repositories/ai_repository.dart';
import 'package:papr_mobile/ui/screens/ai_profiles_screen.dart';

void main() {
  testWidgets('AI profile actions fit narrow Japanese layout and label delete',
      (tester) async {
    await tester.binding.setSurfaceSize(const Size(320, 640));
    addTearDown(() => tester.binding.setSurfaceSize(null));

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          aiProfilesProvider.overrideWith(
            (ref) async => const [
              bridge.AiProfile(
                id: 'test',
                name: 'Test profile',
                protocol: bridge.AiProtocol.openaiChatCompletions,
                model: 'test-model',
                baseUrl: 'https://example.com',
                auth: bridge.AiAuthMode.none,
                headers: [],
                enabled: true,
                defaultFor: [bridge.AiPurpose.summary],
              ),
            ],
          ),
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
          home: const AiProfilesScreen(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(tester.takeException(), isNull);
    expect(find.byTooltip('削除'), findsOneWidget);
  });
}
