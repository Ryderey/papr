import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
import '../../repositories/article_repository.dart';
import '../highlight_style.dart';
import 'article_detail_screen.dart';

final allHighlightsProvider = FutureProvider<List<bridge.Highlight>>((ref) {
  return ref.watch(articleRepositoryProvider).listAllHighlights();
});

class HighlightsScreen extends ConsumerWidget {
  const HighlightsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final highlights = ref.watch(allHighlightsProvider);
    return Scaffold(
      appBar: AppBar(title: Text(context.l10n.globalHighlights)),
      body: highlights.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, _) =>
            Center(child: Text(context.l10n.localizeError(error))),
        data: (items) => items.isEmpty
            ? Center(child: Text(context.l10n.noGlobalHighlights))
            : ListView.builder(
                itemCount: items.length,
                itemBuilder: (context, index) {
                  final highlight = items[index];
                  return ListTile(
                    leading: CircleAvatar(
                      backgroundColor: highlightColor(highlight.color),
                      radius: 10,
                    ),
                    title: Text(highlight.quote),
                    subtitle:
                        highlight.note.isEmpty ? null : Text(highlight.note),
                    onTap: () => Navigator.of(context).push(
                      MaterialPageRoute<void>(
                        builder: (_) => ArticleDetailScreen(
                          articleId: highlight.articleId.toInt(),
                        ),
                      ),
                    ),
                  );
                },
              ),
      ),
    );
  }
}
