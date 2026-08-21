import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/repositories/article_repository.dart';
import 'package:papr_mobile/ui/navigation/app_shell.dart';
import 'package:papr_mobile/ui/screens/article_list_screen.dart';

void main() {
  Widget buildApp() {
    final all = articleFilter(kind: const bridge.ArticleFilterKind.all());
    final starred =
        articleFilter(kind: const bridge.ArticleFilterKind.starred());
    return ProviderScope(
      overrides: [
        articlePageProvider(all).overrideWith((ref) async => const []),
        articlePageProvider(starred).overrideWith((ref) async => const []),
        articleCountProvider(all).overrideWith((ref) async => 0),
        articleCountProvider(starred).overrideWith((ref) async => 0),
        articleCountsProvider.overrideWith(
          (ref) async => const bridge.ArticleCounts(
            all: 0,
            unread: 0,
            starred: 0,
            readLater: 0,
          ),
        ),
        articleTagsProvider.overrideWith((ref) async => const []),
      ],
      child: const MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: AppShell(),
      ),
    );
  }

  testWidgets('uses bottom navigation on phones', (tester) async {
    await tester.binding.setSurfaceSize(const Size(400, 800));
    addTearDown(() => tester.binding.setSurfaceSize(null));

    await tester.pumpWidget(buildApp());
    await tester.pump();

    expect(find.byType(NavigationBar), findsOneWidget);
    expect(find.byType(NavigationRail), findsNothing);
    expect(find.text('Articles'), findsWidgets);
    expect(find.text('Subscriptions'), findsOneWidget);
    expect(find.text('Saved'), findsOneWidget);
    expect(find.text('Settings'), findsOneWidget);
  });

  testWidgets('uses a navigation rail on tablets', (tester) async {
    await tester.binding.setSurfaceSize(const Size(800, 1000));
    addTearDown(() => tester.binding.setSurfaceSize(null));

    await tester.pumpWidget(buildApp());
    await tester.pump();

    expect(find.byType(NavigationRail), findsOneWidget);
    expect(find.byType(NavigationBar), findsNothing);
  });
}
