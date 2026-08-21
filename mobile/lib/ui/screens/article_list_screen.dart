import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
import '../../repositories/article_repository.dart';
import 'article_detail_screen.dart';

const articlePageSize = 50;

bridge.ArticleFilter articleFilter({
  required bridge.ArticleFilterKind kind,
  String? search,
  bool unreadOnly = false,
  bool oldestFirst = false,
  int? limit = articlePageSize,
  int? offset = 0,
}) {
  return bridge.ArticleFilter(
    kind: kind,
    search: search,
    unreadOnly: unreadOnly,
    oldestFirst: oldestFirst,
    limit: limit,
    offset: offset,
  );
}

class ArticleListScreen extends StatelessWidget {
  final bridge.Feed feed;

  const ArticleListScreen({super.key, required this.feed});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(feed.title)),
      body: ArticleCollectionView(
        filter: articleFilter(
          kind: bridge.ArticleFilterKind.feed(feedId: feed.id),
        ),
        emptyText: context.l10n.noArticles,
      ),
    );
  }
}

class ArticleCollectionView extends ConsumerStatefulWidget {
  final bridge.ArticleFilter filter;
  final String emptyText;
  final VoidCallback? onArticleChanged;

  const ArticleCollectionView({
    super.key,
    required this.filter,
    required this.emptyText,
    this.onArticleChanged,
  });

  @override
  ConsumerState<ArticleCollectionView> createState() =>
      _ArticleCollectionViewState();
}

class _ArticleCollectionViewState extends ConsumerState<ArticleCollectionView> {
  final _scrollController = ScrollController();
  List<bridge.ArticleSummary> _items = const [];
  Object? _error;
  bool _loading = false;
  bool _hasMore = true;
  int _generation = 0;

  @override
  void initState() {
    super.initState();
    _scrollController.addListener(_onScroll);
    _load(reset: true);
  }

  @override
  void didUpdateWidget(covariant ArticleCollectionView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.filter != widget.filter) _load(reset: true);
  }

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  void _onScroll() {
    if (_scrollController.position.extentAfter < 360) _load();
  }

  Future<void> _load({bool reset = false}) async {
    if ((!reset && _loading) || (!reset && !_hasMore)) return;
    if (reset) {
      _generation++;
      setState(() {
        _items = const [];
        _error = null;
        _hasMore = true;
      });
    }
    final generation = _generation;
    setState(() => _loading = true);
    try {
      final pageFilter = _withOffset(
        widget.filter,
        reset ? 0 : _items.length,
      );
      final page = await ref.read(articlePageProvider(pageFilter).future);
      if (!mounted || generation != _generation) return;
      final byId = <int, bridge.ArticleSummary>{
        for (final item in _items) item.id.toInt(): item,
        for (final item in page) item.id.toInt(): item,
      };
      setState(() {
        _items = byId.values.toList(growable: false);
        _hasMore = page.length == articlePageSize;
        _error = null;
      });
    } catch (error) {
      if (mounted && generation == _generation) {
        setState(() => _error = error);
      }
    } finally {
      if (mounted && generation == _generation) {
        setState(() => _loading = false);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_items.isEmpty && _loading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_items.isEmpty && _error != null) {
      return _MessageState(
        message: context.l10n.localizeError(_error!),
        actionLabel: context.l10n.retry,
        onAction: () => _load(reset: true),
      );
    }
    if (_items.isEmpty) return Center(child: Text(widget.emptyText));

    return RefreshIndicator(
      onRefresh: () => _load(reset: true),
      child: ListView.builder(
        controller: _scrollController,
        physics: const AlwaysScrollableScrollPhysics(),
        itemCount: _items.length + (_hasMore || _loading ? 1 : 0),
        itemBuilder: (context, index) {
          if (index == _items.length) {
            return const Padding(
              padding: EdgeInsets.all(20),
              child: Center(child: CircularProgressIndicator()),
            );
          }
          final article = _items[index];
          return _ArticleCard(
            article: article,
            onOpen: () => _open(index),
            onAction: (action) => _changeState(index, action),
          );
        },
      ),
    );
  }

  Future<void> _open(int index) async {
    final articleId = _items[index].id.toInt();
    if (!_items[index].isRead) await _changeState(index, _ArticleAction.read);
    if (!mounted) return;
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => ArticleDetailScreen(
          articleId: articleId,
        ),
      ),
    );
    if (mounted) await _load(reset: true);
  }

  Future<void> _changeState(int index, _ArticleAction action) async {
    final previous = _items[index];
    final next = switch (action) {
      _ArticleAction.read => _copyArticle(previous, isRead: true),
      _ArticleAction.unread => _copyArticle(previous, isRead: false),
      _ArticleAction.star => _copyArticle(previous, isStarred: true),
      _ArticleAction.unstar => _copyArticle(previous, isStarred: false),
      _ArticleAction.saveLater => _copyArticle(previous, readLater: true),
      _ArticleAction.removeLater => _copyArticle(previous, readLater: false),
    };
    setState(() => _items[index] = next);
    try {
      final repo = ref.read(articleRepositoryProvider);
      switch (action) {
        case _ArticleAction.read:
          await repo.setRead(previous.id.toInt(), true);
        case _ArticleAction.unread:
          await repo.setRead(previous.id.toInt(), false);
        case _ArticleAction.star:
          await repo.setStarred(previous.id.toInt(), true);
        case _ArticleAction.unstar:
          await repo.setStarred(previous.id.toInt(), false);
        case _ArticleAction.saveLater:
          await repo.setReadLater(previous.id.toInt(), true);
        case _ArticleAction.removeLater:
          await repo.setReadLater(previous.id.toInt(), false);
      }
      ref.invalidate(articleCountsProvider);
      ref.invalidate(articleCountProvider);
      widget.onArticleChanged?.call();
      if (_doesNotMatch(next, widget.filter)) {
        setState(() => _items.removeAt(index));
      }
    } catch (error) {
      if (!mounted) return;
      final current = _items.indexWhere((item) => item.id == previous.id);
      if (current >= 0) setState(() => _items[current] = previous);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(context.l10n.localizeError(error))),
      );
    }
  }
}

class _ArticleCard extends StatelessWidget {
  final bridge.ArticleSummary article;
  final VoidCallback onOpen;
  final ValueChanged<_ArticleAction> onAction;

  const _ArticleCard({
    required this.article,
    required this.onOpen,
    required this.onAction,
  });

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Card(
      margin: const EdgeInsets.fromLTRB(12, 6, 12, 6),
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onOpen,
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _ArticleThumbnail(url: article.imageUrl),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      article.title,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                      style: Theme.of(context).textTheme.titleMedium?.copyWith(
                            fontWeight: article.isRead
                                ? FontWeight.normal
                                : FontWeight.bold,
                          ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      [
                        article.feedTitle,
                        if (article.publishedAt != null)
                          _shortDate(article.publishedAt!),
                      ].join(' · '),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                    if (article.snippet?.trim().isNotEmpty ?? false) ...[
                      const SizedBox(height: 6),
                      Text(
                        article.snippet!,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                    const SizedBox(height: 6),
                    Row(
                      children: [
                        if (!article.isRead) const Icon(Icons.circle, size: 9),
                        if (article.isStarred) ...[
                          const SizedBox(width: 8),
                          const Icon(Icons.star, size: 18),
                        ],
                        if (article.readLater) ...[
                          const SizedBox(width: 8),
                          const Icon(Icons.bookmark, size: 18),
                        ],
                      ],
                    ),
                  ],
                ),
              ),
              PopupMenuButton<_ArticleAction>(
                tooltip: l10n.articleActions,
                onSelected: onAction,
                itemBuilder: (_) => [
                  PopupMenuItem(
                    value: article.isRead
                        ? _ArticleAction.unread
                        : _ArticleAction.read,
                    child:
                        Text(article.isRead ? l10n.markUnread : l10n.markRead),
                  ),
                  PopupMenuItem(
                    value: article.isStarred
                        ? _ArticleAction.unstar
                        : _ArticleAction.star,
                    child: Text(article.isStarred ? l10n.unstar : l10n.star),
                  ),
                  PopupMenuItem(
                    value: article.readLater
                        ? _ArticleAction.removeLater
                        : _ArticleAction.saveLater,
                    child: Text(
                      article.readLater ? l10n.removeReadLater : l10n.readLater,
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ArticleThumbnail extends StatelessWidget {
  final String? url;

  const _ArticleThumbnail({this.url});

  @override
  Widget build(BuildContext context) {
    final placeholder = Container(
      width: 72,
      height: 72,
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      alignment: Alignment.center,
      child: const Icon(Icons.image_not_supported_outlined),
    );
    final uri = url == null ? null : Uri.tryParse(url!);
    if (uri == null || (uri.scheme != 'http' && uri.scheme != 'https')) {
      return placeholder;
    }
    return ClipRRect(
      borderRadius: BorderRadius.circular(8),
      child: Image.network(
        url!,
        width: 72,
        height: 72,
        fit: BoxFit.cover,
        errorBuilder: (_, __, ___) => placeholder,
      ),
    );
  }
}

class _MessageState extends StatelessWidget {
  final String message;
  final String actionLabel;
  final VoidCallback onAction;

  const _MessageState({
    required this.message,
    required this.actionLabel,
    required this.onAction,
  });

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(message, textAlign: TextAlign.center),
          const SizedBox(height: 12),
          FilledButton(onPressed: onAction, child: Text(actionLabel)),
        ],
      ),
    );
  }
}

bridge.ArticleFilter _withOffset(bridge.ArticleFilter filter, int offset) {
  return bridge.ArticleFilter(
    kind: filter.kind,
    search: filter.search,
    unreadOnly: filter.unreadOnly,
    oldestFirst: filter.oldestFirst,
    limit: articlePageSize,
    offset: offset,
  );
}

bridge.ArticleSummary _copyArticle(
  bridge.ArticleSummary article, {
  bool? isRead,
  bool? isStarred,
  bool? readLater,
}) {
  return bridge.ArticleSummary(
    id: article.id,
    feedId: article.feedId,
    feedTitle: article.feedTitle,
    sourceType: article.sourceType,
    title: article.title,
    author: article.author,
    snippet: article.snippet,
    imageUrl: article.imageUrl,
    url: article.url,
    publishedAt: article.publishedAt,
    isRead: isRead ?? article.isRead,
    isStarred: isStarred ?? article.isStarred,
    readLater: readLater ?? article.readLater,
  );
}

bool _doesNotMatch(
  bridge.ArticleSummary article,
  bridge.ArticleFilter filter,
) {
  if (filter.unreadOnly && article.isRead) return true;
  return filter.kind.when(
    all: () => false,
    unread: () => article.isRead,
    starred: () => !article.isStarred,
    readLater: () => !article.readLater,
    feed: (_) => false,
    folder: (_) => false,
    tag: (_) => false,
  );
}

String _shortDate(String value) =>
    value.length >= 10 ? value.substring(0, 10) : value;

enum _ArticleAction { read, unread, star, unstar, saveLater, removeLater }
