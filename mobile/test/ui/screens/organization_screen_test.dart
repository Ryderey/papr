import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/repositories/article_repository.dart';
import 'package:papr_mobile/ui/screens/article_detail_screen.dart';
import 'package:papr_mobile/ui/screens/highlights_screen.dart';
import 'package:papr_mobile/ui/screens/organization_screen.dart';

void main() {
  testWidgets('opens the tag creation flow from the tag manager',
      (tester) async {
    await tester.pumpWidget(
      _app(
        const TagManagerScreen(),
        [
          articleTagsProvider.overrideWith((ref) async => const [
                bridge.TagSummary(
                  id: 1,
                  name: 'Work',
                  color: 'teal',
                  articleCount: 2,
                  position: 0,
                ),
              ]),
        ],
      ),
    );
    await tester.pumpAndSettle();

    expect(find.byType(ReorderableListView), findsOneWidget);
    expect(find.text('Work'), findsOneWidget);
    await tester.tap(find.byTooltip('Create tag'));
    await tester.pumpAndSettle();
    expect(find.text('Tag name'), findsOneWidget);
    expect(find.text('Save'), findsOneWidget);
  });

  testWidgets('shows persisted rules in the rule manager', (tester) async {
    await tester.pumpWidget(
      _app(
        const RuleManagerScreen(),
        [
          rulesProvider.overrideWith((ref) async => const [
                bridge.Rule(
                  id: 1,
                  name: 'Skip newsletters',
                  enabled: true,
                  field: 'title',
                  query: 'newsletter',
                  action: 'skip',
                  position: 0,
                ),
              ]),
        ],
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Skip newsletters'), findsOneWidget);
    expect(find.text('title: newsletter → skip'), findsOneWidget);
    expect(find.byType(Switch), findsOneWidget);
  });

  testWidgets('opens the article from the global highlights list',
      (tester) async {
    await tester.pumpWidget(
      _app(
        const HighlightsScreen(),
        [
          allHighlightsProvider.overrideWith((ref) async => [_highlight]),
          articleDetailProvider(1).overrideWith((ref) async => _article),
        ],
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Selected sentence'));
    await tester.pumpAndSettle();
    expect(find.text('Article'), findsOneWidget);
    expect(find.textContaining('Article body', findRichText: true),
        findsOneWidget);
  });
}

Widget _app(Widget home, List<Override> overrides) => ProviderScope(
      overrides: overrides,
      child: MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: home,
      ),
    );

const _highlight = bridge.Highlight(
  id: 1,
  articleId: 1,
  quote: 'Selected sentence',
  prefix: '',
  suffix: '',
  textOffset: 0,
  color: 'yellow',
  note: 'Remember this',
  createdAt: '2026-08-25T00:00:00Z',
);

const _article = bridge.ArticleDetail(
  id: 1,
  feedId: 1,
  feedTitle: 'Feed',
  sourceType: bridge.SourceType.rss,
  title: 'Article title',
  contentHtml: '<p>Article body</p>',
  isRead: true,
  isStarred: false,
  readLater: false,
  enclosures: [],
  tags: [],
);
