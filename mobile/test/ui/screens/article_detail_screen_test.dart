import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
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
                  src="data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///ywAAAAAAQABAAACAUwAOw=="
                  width="2000"
                  height="1000"
                >
              ''',
            ),
          ),
        ],
        child: const MaterialApp(
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
          home: ArticleDetailScreen(articleId: 1),
        ),
      ),
    );

    await tester.pumpAndSettle();

    expect(find.text('No content'), findsOneWidget);
  });
}

bridge.ArticleDetail _article({String? contentHtml}) {
  return bridge.ArticleDetail(
    id: 1,
    feedId: 1,
    feedTitle: 'Feed',
    sourceType: bridge.SourceType.rss,
    title: 'Article title',
    url: 'https://example.com/articles/1',
    contentHtml: contentHtml,
    isRead: false,
    isStarred: false,
    readLater: false,
    enclosures: const [],
    tags: const [],
  );
}
