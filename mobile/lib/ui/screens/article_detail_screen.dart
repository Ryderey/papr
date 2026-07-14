import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../repositories/article_repository.dart';

final articleDetailProvider = FutureProvider.family<bridge.ArticleDetail, int>(
  (ref, articleId) async {
    final repo = ref.watch(articleRepositoryProvider);
    return repo.getArticleDetail(articleId);
  },
);

class ArticleDetailScreen extends ConsumerWidget {
  final int articleId;

  const ArticleDetailScreen({super.key, required this.articleId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final article = ref.watch(articleDetailProvider(articleId));

    return Scaffold(
      appBar: AppBar(
        title: const Text('Article'),
      ),
      body: article.when(
        data: (detail) => SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                detail.title,
                style: Theme.of(context).textTheme.headlineSmall,
              ),
              const SizedBox(height: 8),
              if (detail.author != null)
                Text('By ${detail.author}', style: Theme.of(context).textTheme.bodySmall),
              const SizedBox(height: 16),
              Text(
                detail.extractedHtml ?? detail.contentHtml ?? 'No content',
              ),
            ],
          ),
        ),
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, stack) => Center(child: Text('Error: $err')),
      ),
    );
  }
}
