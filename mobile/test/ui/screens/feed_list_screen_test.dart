import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/ui/screens/feed_list_screen.dart';

void main() {
  const folder = bridge.Folder(id: 7, name: 'News', position: 0);
  const feed = bridge.Feed(
    id: 11,
    feedUrl: 'https://example.com/feed.xml',
    title: 'Example feed',
    folderId: 7,
    sourceType: bridge.SourceType.rss,
    unreadCount: 3,
    customTitle: false,
  );

  Widget buildApp() {
    return ProviderScope(
      overrides: [
        feedListProvider.overrideWith((ref) async => const [feed]),
        folderListProvider.overrideWith((ref) async => const [folder]),
      ],
      child: const MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: FeedListScreen(),
      ),
    );
  }

  testWidgets('shows folder metadata and feed management actions',
      (tester) async {
    await tester.pumpWidget(buildApp());
    await tester.pumpAndSettle();

    expect(find.text('Example feed'), findsOneWidget);
    expect(find.textContaining('News'), findsOneWidget);

    await tester.tap(find.byTooltip('Subscription actions'));
    await tester.pumpAndSettle();
    expect(find.text('Move to folder'), findsOneWidget);
    expect(find.text('Subscription refresh interval'), findsOneWidget);

    await tester.tap(find.text('Delete'));
    await tester.pumpAndSettle();
    expect(find.text('Delete subscription?'), findsOneWidget);
    expect(find.textContaining('Example feed'), findsWidgets);

    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
  });

  testWidgets('opens URL and directory add flow', (tester) async {
    await tester.pumpWidget(buildApp());
    await tester.pumpAndSettle();

    await tester.tap(find.byType(FloatingActionButton));
    await tester.pumpAndSettle();

    expect(find.text('Feed or website URL'), findsOneWidget);
    expect(find.text('Directory'), findsOneWidget);
    expect(find.text('Search publications or topics'), findsOneWidget);
  });

  testWidgets('opens reorderable folder management', (tester) async {
    await tester.pumpWidget(buildApp());
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Manage folders'));
    await tester.pumpAndSettle();

    expect(find.byType(ReorderableListView), findsOneWidget);
    expect(find.text('News'), findsOneWidget);
    expect(find.text('Create folder'), findsOneWidget);
  });
}
