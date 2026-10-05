import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/l10n/l10n.dart';
import 'package:papr_mobile/repositories/article_repository.dart';
import 'package:papr_mobile/ui/screens/article_detail_screen.dart';
import 'package:papr_mobile/ui/screens/article_list_screen.dart';

void main() {
  testWidgets(
      'sync refreshes an already mounted empty list and removes old rows',
      (tester) async {
    final data = _Data()..empty = true;
    final container = await _mount(tester, data, reader: false);
    expect(find.text('Empty'), findsOneWidget);
    data.empty = false;
    container.invalidate(articleRepositoryProvider);
    await tester.pumpAndSettle();
    expect(find.text('Original title'), findsOneWidget);
    data.title = 'Remote title';
    container.invalidate(articleRepositoryProvider);
    await tester.pumpAndSettle();
    expect(find.text('Remote title'), findsOneWidget);
    expect(find.text('Original title'), findsNothing);
    data.empty = true;
    container.invalidate(articleRepositoryProvider);
    await tester.pumpAndSettle();
    expect(find.text('Empty'), findsOneWidget);
  });

  testWidgets('sync retains the already loaded page range and scroll position',
      (tester) async {
    final data = _Data()..count = 51;
    final container = await _mount(tester, data, reader: false);
    await tester.fling(find.byType(ListView), const Offset(0, -8000), 5000);
    await tester.pumpAndSettle();
    expect(find.text('Original title 51'), findsOneWidget);
    data.title = 'Synced title';
    container.invalidate(articleRepositoryProvider);
    await tester.pumpAndSettle();
    expect(find.text('Synced title 51'), findsOneWidget);
    expect(data.requestedLimits, contains(52));
  });

  testWidgets(
      'sync refreshes reader flags without marking remote unread state read',
      (tester) async {
    final data = _Data();
    final container = await _mount(tester, data, reader: true);
    expect(find.byTooltip('Star'), findsOneWidget);
    data.starred = true;
    data.later = true;
    data.read = false;
    container.invalidate(articleRepositoryProvider);
    await tester.pumpAndSettle();
    expect(find.byTooltip('Unstar'), findsOneWidget);
    expect(find.byTooltip('Remove from read later'), findsOneWidget);
    expect(find.byTooltip('Mark read'), findsOneWidget);
    expect(data.readWrites, 0);
  });

  testWidgets('sync waits for a pending optimistic list write before reloading',
      (tester) async {
    final data = _Data()..write = Completer<void>();
    final container = await _mount(tester, data, reader: false);
    await tester.tap(find.byTooltip('Article actions'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Star'));
    await tester.pumpAndSettle();
    expect(find.byIcon(Icons.star), findsOneWidget);
    data.title = 'Synced title';
    container.invalidate(articleRepositoryProvider);
    await tester.pumpAndSettle();
    expect(find.text('Original title'), findsOneWidget);
    expect(find.byIcon(Icons.star), findsOneWidget);
    data.write!.complete();
    await tester.pumpAndSettle();
    expect(find.text('Synced title'), findsOneWidget);
    expect(find.byIcon(Icons.star), findsOneWidget);
  });

  testWidgets(
      'sync waits for a pending reader write and preserves its final intent',
      (tester) async {
    final data = _Data()..write = Completer<void>();
    final container = await _mount(tester, data, reader: true);
    await tester.tap(find.byTooltip('Star'));
    await tester.pumpAndSettle();
    expect(find.byTooltip('Unstar'), findsOneWidget);
    data.later = true;
    container.invalidate(articleRepositoryProvider);
    await tester.pumpAndSettle();
    expect(find.byTooltip('Unstar'), findsOneWidget);
    data.write!.complete();
    await tester.pumpAndSettle();
    expect(find.byTooltip('Unstar'), findsOneWidget);
    expect(find.byTooltip('Remove from read later'), findsOneWidget);
  });
}

Future<ProviderContainer> _mount(WidgetTester tester, _Data data,
    {required bool reader}) async {
  await tester.pumpWidget(ProviderScope(
    overrides: [
      articleRepositoryProvider.overrideWith((ref) => _Repository(ref, data))
    ],
    child: MaterialApp(
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: reader
          ? const ArticleDetailScreen(articleId: 1)
          : Scaffold(
              body: ArticleCollectionView(
                  filter:
                      articleFilter(kind: const bridge.ArticleFilterKind.all()),
                  emptyText: 'Empty')),
    ),
  ));
  await tester.pumpAndSettle();
  return ProviderScope.containerOf(tester.element(find.byType(Scaffold).first));
}

class _Data {
  bool empty = false, starred = false, later = false, read = true;
  String title = 'Original title';
  int readWrites = 0;
  int count = 1;
  final List<int?> requestedLimits = [];
  Completer<void>? write;
}

class _Repository extends ArticleRepository {
  final _Data data;
  _Repository(super.ref, this.data);
  @override
  Future<List<bridge.ArticleSummary>> listArticles(
      {bridge.ArticleFilter? filter}) async {
    data.requestedLimits.add(filter?.limit);
    if (data.empty) return [];
    final articles = [
      for (var i = 1; i <= data.count; i++)
        bridge.ArticleSummary(
            id: i,
            feedId: 1,
            feedTitle: 'Feed',
            sourceType: bridge.SourceType.rss,
            title: data.count == 1 ? data.title : '${data.title} $i',
            isRead: data.read,
            isStarred: data.starred,
            readLater: data.later)
    ];
    return articles
        .skip(filter?.offset ?? 0)
        .take(filter?.limit ?? articles.length)
        .toList();
  }

  @override
  Future<bridge.ArticleDetail> getArticleDetail(int id) async =>
      bridge.ArticleDetail(
          id: 1,
          feedId: 1,
          feedTitle: 'Feed',
          sourceType: bridge.SourceType.rss,
          title: data.title,
          contentHtml: '<p>Cached body</p>',
          isRead: data.read,
          isStarred: data.starred,
          readLater: data.later,
          enclosures: const [],
          tags: const []);
  @override
  Future<void> setStarred(int id, bool value) async {
    await data.write?.future;
    data.starred = value;
  }

  @override
  Future<void> setRead(int id, bool value) async {
    data.readWrites++;
    data.read = value;
  }

  @override
  Future<List<bridge.Highlight>> listHighlights(int id) async => const [];
  @override
  Future<List<bridge.ResolvedHighlight>> resolveHighlights(
          int id, String text) async =>
      const [];
}
