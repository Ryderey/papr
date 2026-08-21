import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_widget_from_html_core/flutter_widget_from_html_core.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
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
        title: Text(context.l10n.articleTitle),
      ),
      body: article.when(
        data: (detail) {
          final content = detail.extractedHtml ?? detail.contentHtml;
          final hasContent = content?.trim().isNotEmpty ?? false;

          return SingleChildScrollView(
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
                  Text(
                    context.l10n.byAuthor(detail.author!),
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                const SizedBox(height: 16),
                if (!hasContent)
                  Text(context.l10n.noContent)
                else
                  HtmlWidget(
                    content!,
                    baseUrl:
                        detail.url == null ? null : Uri.tryParse(detail.url!),
                    customStylesBuilder: (element) => element.localName == 'img'
                        ? {'max-width': '100%', 'height': 'auto'}
                        : null,
                    textStyle: Theme.of(context).textTheme.bodyMedium,
                  ),
              ],
            ),
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, stack) =>
            Center(child: Text(context.l10n.errorMessage(err.toString()))),
      ),
    );
  }
}
