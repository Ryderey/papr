import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../repositories/article_repository.dart';
import 'article_detail_screen.dart';

final articleListProvider = FutureProvider.family<List<bridge.ArticleSummary>, bridge.Feed>(
  (ref, feed) async {
    final repo = ref.watch(articleRepositoryProvider);
    return repo.listArticles(
      filter: bridge.ArticleFilter(
        kind: bridge.ArticleFilterKind.feed(feedId: feed.id),
        limit: 100,
        offset: null,
      ),
    );
  },
);

class ArticleListScreen extends ConsumerWidget {
  final bridge.Feed feed;

  const ArticleListScreen({super.key, required this.feed});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final articles = ref.watch(articleListProvider(feed));

    return Scaffold(
      appBar: AppBar(
        title: Text(feed.title),
      ),
      body: articles.when(
        data: (items) => ListView.builder(
          itemCount: items.length,
          itemBuilder: (context, index) {
            final article = items[index];
            return ListTile(
              title: Text(
                article.title,
                style: TextStyle(
                  fontWeight: article.isRead ? FontWeight.normal : FontWeight.bold,
                ),
              ),
              subtitle: article.snippet != null ? Text(article.snippet!) : null,
              onTap: () {
                Navigator.of(context).push(
                  MaterialPageRoute(
                    builder: (_) => ArticleDetailScreen(articleId: article.id.toInt()),
                  ),
                );
              },
            );
          },
        ),
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, stack) => Center(child: Text('Error: $err')),
      ),
    );
  }
}
