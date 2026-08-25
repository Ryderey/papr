import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/repositories/article_repository.dart';
import 'package:papr_mobile/repositories/feed_repository.dart';
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

  testWidgets('previews a skip rule and applies it after confirmation',
      (tester) async {
    final calls = _RuleCalls();
    await tester.pumpWidget(
      _app(
        Stack(
          children: [
            const RuleManagerScreen(),
            _CountsProbe(calls),
          ],
        ),
        [
          rulesProvider.overrideWith((ref) async => const []),
          articleRepositoryProvider.overrideWith(
            (ref) => _RuleRepository(ref, calls),
          ),
          feedRepositoryProvider.overrideWith((ref) => _FeedRepository(ref)),
          articleCountsProvider.overrideWith((ref) async {
            calls.countLoads++;
            return const bridge.ArticleCounts(
              all: 2,
              unread: 2,
              starred: 0,
              readLater: 0,
            );
          }),
        ],
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Count loads: 1'), findsOneWidget);

    await tester.tap(find.byTooltip('New rule'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField).at(0), 'Filter newsletters');
    await tester.enterText(find.byType(TextField).at(1), 'newsletter');
    await tester.tap(find.text('Preview'));
    await tester.pumpAndSettle();

    expect(find.text('Matches 2 article(s)'), findsOneWidget);
    expect(find.text('Weekly newsletter'), findsOneWidget);
    expect(calls.previewInputs, hasLength(1));
    expect(calls.previewInputs.single.query, 'newsletter');

    await tester.tap(find.text('Apply to existing articles'));
    await tester.pumpAndSettle();
    expect(find.text('Apply skip rule?'), findsOneWidget);
    expect(
      find.text(
          'Matching unsaved articles will be removed. Starred, read-later, and highlighted articles are kept.'),
      findsOneWidget,
    );
    await tester.tap(
      find.descendant(
        of: find.byType(AlertDialog),
        matching: find.text('Apply to existing articles'),
      ),
    );
    await tester.pumpAndSettle();

    expect(calls.appliedInputs, hasLength(1));
    expect(calls.appliedInputs.single.action, 'skip');
    expect(find.text('Applied to 2 article(s).'), findsOneWidget);
    expect(find.text('Count loads: 2'), findsOneWidget);
  });

  testWidgets('rolls back an optimistic rule enable change on failure',
      (tester) async {
    final calls = _RuleCalls(updateCompleter: Completer<void>());
    await tester.pumpWidget(
      _app(
        const RuleManagerScreen(),
        [
          rulesProvider.overrideWith((ref) async => const [_enabledRule]),
          articleRepositoryProvider.overrideWith(
            (ref) => _RuleRepository(ref, calls),
          ),
        ],
      ),
    );
    await tester.pumpAndSettle();

    final toggle = find.byType(Switch);
    expect(tester.widget<Switch>(toggle).value, isTrue);
    await tester.tap(toggle);
    await tester.pump();
    expect(tester.widget<Switch>(toggle).value, isFalse);

    calls.updateCompleter!.completeError(StateError('update failed'));
    await tester.pumpAndSettle();
    expect(tester.widget<Switch>(toggle).value, isTrue);
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

const _enabledRule = bridge.Rule(
  id: 1,
  name: 'Keep newsletters',
  enabled: true,
  field: 'title',
  query: 'newsletter',
  action: 'read',
  position: 0,
);

class _RuleCalls {
  final List<bridge.RuleInput> previewInputs = [];
  final List<bridge.RuleInput> appliedInputs = [];
  final Completer<void>? updateCompleter;
  int countLoads = 0;

  _RuleCalls({this.updateCompleter});
}

class _RuleRepository extends ArticleRepository {
  final _RuleCalls calls;

  _RuleRepository(super.ref, this.calls);

  @override
  Future<bridge.RulePreview> previewRule(bridge.RuleInput input) async {
    calls.previewInputs.add(input);
    return const bridge.RulePreview(
      count: 2,
      samples: ['Weekly newsletter'],
    );
  }

  @override
  Future<int> applyRuleToExisting(bridge.RuleInput input) async {
    calls.appliedInputs.add(input);
    return 2;
  }

  @override
  Future<void> updateRule(int id, bridge.RuleInput input) async {
    await calls.updateCompleter?.future;
  }
}

class _FeedRepository extends FeedRepository {
  _FeedRepository(super.ref);

  @override
  Future<List<bridge.Feed>> listFeeds() async => const [];
}

class _CountsProbe extends ConsumerWidget {
  final _RuleCalls calls;

  const _CountsProbe(this.calls);

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.watch(articleCountsProvider);
    return IgnorePointer(
      child: Align(
        alignment: Alignment.topRight,
        child: Text('Count loads: ${calls.countLoads}'),
      ),
    );
  }
}

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
