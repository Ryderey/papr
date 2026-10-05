import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/repositories/article_repository.dart';
import 'package:papr_mobile/ui/screens/article_browser_screen.dart';
import 'package:papr_mobile/ui/screens/article_list_screen.dart';
import 'package:papr_mobile/ui/screens/feed_list_screen.dart';

void main() {
  testWidgets('shows smart views, counts, and article metadata',
      (tester) async {
    final filter = articleFilter(kind: const bridge.ArticleFilterKind.all());
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          articlePageProvider(filter).overrideWith(
            (ref) async => [_article(1, title: 'A useful article')],
          ),
          articleCountProvider(filter).overrideWith((ref) async => 1),
          articleCountsProvider.overrideWith(
            (ref) async => const bridge.ArticleCounts(
              all: 1,
              unread: 1,
              starred: 0,
              readLater: 0,
            ),
          ),
          articleTagsProvider.overrideWith((ref) async => const []),
          feedListProvider.overrideWith((ref) async => const []),
          folderListProvider.overrideWith((ref) async => const []),
        ],
        child: const MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: ArticleBrowserScreen(savedOnly: false),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('A useful article'), findsOneWidget);
    expect(find.textContaining('Example Feed'), findsOneWidget);
    expect(find.byIcon(Icons.image_not_supported_outlined), findsOneWidget);

    await tester.tap(find.byTooltip('Open navigation menu'));
    await tester.pumpAndSettle();
    expect(find.text('All articles'), findsOneWidget);
    expect(find.text('Unread'), findsOneWidget);
    expect(find.text('Starred'), findsOneWidget);
    expect(find.text('Read later'), findsOneWidget);
  });

  testWidgets('loads the next bounded page at offset 50', (tester) async {
    final first = articleFilter(kind: const bridge.ArticleFilterKind.all());
    final second = articleFilter(
      kind: const bridge.ArticleFilterKind.all(),
      offset: 50,
    );
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          articlePageProvider(first).overrideWith(
            (ref) async => [
              for (var i = 0; i < articlePageSize; i++) _article(i),
            ],
          ),
          articlePageProvider(second).overrideWith(
            (ref) async => [_article(50, title: 'Second page')],
          ),
        ],
        child: MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: Scaffold(
            body: ArticleCollectionView(
              filter: first,
              emptyText: 'Empty',
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await tester.fling(find.byType(ListView), const Offset(0, -8000), 5000);
    await tester.pumpAndSettle();

    expect(find.text('Second page'), findsOneWidget);
  });
}

bridge.ArticleSummary _article(int id, {String? title}) {
  return bridge.ArticleSummary(
    id: id,
    feedId: 1,
    feedTitle: 'Example Feed',
    sourceType: bridge.SourceType.rss,
    title: title ?? 'Article $id',
    snippet: 'Summary $id',
    publishedAt: '2026-08-21T00:00:00Z',
    isRead: false,
    isStarred: false,
    readLater: false,
  );
}
