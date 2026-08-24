import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/repositories/article_repository.dart';
import 'package:papr_mobile/ui/screens/article_detail_screen.dart';

void main() {
  testWidgets('renders article HTML and inline images', (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          articleDetailProvider(1).overrideWith(
            (ref) async => _article(
              contentHtml: '''
                <p>Rendered <strong>body</strong></p>
                <img
                  src="https://example.com/image.jpg"
                  width="2000"
                  height="1000"
                >
              ''',
            ),
          ),
        ],
        child: const MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: ArticleDetailScreen(articleId: 1),
        ),
      ),
    );

    await tester.pumpAndSettle();

    expect(find.textContaining('<p>'), findsNothing);
    expect(find.textContaining('Rendered body', findRichText: true),
        findsOneWidget);
    expect(find.byType(Image), findsOneWidget);
  });

  testWidgets('shows a placeholder for blank article HTML', (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          articleDetailProvider(1).overrideWith(
            (ref) async => _article(contentHtml: '   '),
          ),
        ],
        child: const MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: ArticleDetailScreen(articleId: 1),
        ),
      ),
    );

    await tester.pumpAndSettle();

    expect(find.text('No content'), findsOneWidget);
  });

  testWidgets('wraps reader text in a selectable highlight area',
      (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          articleDetailProvider(1).overrideWith(
            (ref) async => _article(contentHtml: '<p>Select this body</p>'),
          ),
          articleTagsProvider.overrideWith((ref) async => const []),
        ],
        child: const MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: ArticleDetailScreen(articleId: 1),
        ),
      ),
    );

    await tester.pumpAndSettle();

    expect(find.byType(SelectionArea), findsOneWidget);
    expect(find.text('Highlights'), findsOneWidget);
    expect(find.textContaining('No highlights yet'), findsOneWidget);
  });

  testWidgets('disables browser and sharing when source URL is absent',
      (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          articleDetailProvider(1).overrideWith(
            (ref) async => _article(contentHtml: '<p>Cached</p>', url: null),
          ),
        ],
        child: const MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: ArticleDetailScreen(articleId: 1),
        ),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Reader actions'));
    await tester.pumpAndSettle();

    final browser = tester.widget<Widget>(
      find.ancestor(
        of: find.text('Open in browser'),
        matching: find.byWidgetPredicate((widget) => widget is PopupMenuItem),
      ),
    );
    final share = tester.widget<Widget>(
      find.ancestor(
        of: find.text('Share'),
        matching: find.byWidgetPredicate((widget) => widget is PopupMenuItem),
      ),
    );
    expect((browser as dynamic).enabled, isFalse);
    expect((share as dynamic).enabled, isFalse);
  });
}

bridge.ArticleDetail _article({
  String? contentHtml,
  String? url = 'https://example.com/articles/1',
}) {
  return bridge.ArticleDetail(
    id: 1,
    feedId: 1,
    feedTitle: 'Feed',
    sourceType: bridge.SourceType.rss,
    title: 'Article title',
    url: url,
    contentHtml: contentHtml,
    isRead: true,
    isStarred: false,
    readLater: false,
    enclosures: const [],
    tags: const [],
  );
}
