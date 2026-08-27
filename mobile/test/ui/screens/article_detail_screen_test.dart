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
    expect(find.byTooltip('AI summary'), findsOneWidget);
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

  testWidgets('keeps starred and read-later state after reopening the article',
      (tester) async {
    final flags = _ArticleFlags();
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          articleRepositoryProvider.overrideWith(
            (ref) => _MemoryArticleRepository(ref, flags),
          ),
          articleDetailProvider(1).overrideWith(
            (ref) async => _article(
              contentHtml: '<p>Body</p>',
              isStarred: flags.isStarred,
              readLater: flags.readLater,
            ),
          ),
        ],
        child: const MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          home: _ArticleRouteHarness(),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Open article'));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Star'));
    await tester.tap(find.byTooltip('Read later'));
    await tester.pumpAndSettle();
    await tester.pageBack();
    await tester.pumpAndSettle();
    await tester.tap(find.text('Open article'));
    await tester.pumpAndSettle();

    expect(find.byTooltip('Unstar'), findsOneWidget);
    expect(find.byTooltip('Remove from read later'), findsOneWidget);
    expect(find.byTooltip('Edit tags'), findsNothing);
  });
}

bridge.ArticleDetail _article({
  String? contentHtml,
  String? url = 'https://example.com/articles/1',
  bool isStarred = false,
  bool readLater = false,
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
    isStarred: isStarred,
    readLater: readLater,
    enclosures: const [],
    tags: const [],
  );
}

class _ArticleRouteHarness extends StatelessWidget {
  const _ArticleRouteHarness();

  @override
  Widget build(BuildContext context) => Scaffold(
        body: Center(
          child: FilledButton(
            onPressed: () => Navigator.of(context).push(
              MaterialPageRoute<void>(
                builder: (_) => const ArticleDetailScreen(articleId: 1),
              ),
            ),
            child: const Text('Open article'),
          ),
        ),
      );
}

class _ArticleFlags {
  bool isStarred = false;
  bool readLater = false;
}

class _MemoryArticleRepository extends ArticleRepository {
  final _ArticleFlags flags;

  _MemoryArticleRepository(super.ref, this.flags);

  @override
  Future<void> setStarred(int articleId, bool value) async {
    flags.isStarred = value;
  }

  @override
  Future<void> setReadLater(int articleId, bool value) async {
    flags.readLater = value;
  }

  @override
  Future<List<bridge.Highlight>> listHighlights(int articleId) async =>
      const [];

  @override
  Future<List<bridge.ResolvedHighlight>> resolveHighlights(
    int articleId,
    String text,
  ) async =>
      const [];
}
