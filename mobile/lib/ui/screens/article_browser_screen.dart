import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
import '../../repositories/article_repository.dart';
import 'article_list_screen.dart';
import 'feed_list_screen.dart';

class ArticleBrowserScreen extends ConsumerStatefulWidget {
  final bool savedOnly;

  const ArticleBrowserScreen({super.key, required this.savedOnly});

  @override
  ConsumerState<ArticleBrowserScreen> createState() =>
      _ArticleBrowserScreenState();
}

class _ArticleBrowserScreenState extends ConsumerState<ArticleBrowserScreen> {
  final _searchController = TextEditingController();
  late bridge.ArticleFilterKind _kind;
  String? _selectionLabel;
  String? _search;
  bool _searching = false;
  bool _unreadOnly = false;
  bool _oldestFirst = false;
  int _revision = 0;

  @override
  void initState() {
    super.initState();
    _kind = widget.savedOnly
        ? const bridge.ArticleFilterKind.starred()
        : const bridge.ArticleFilterKind.all();
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  bridge.ArticleFilter get _filter => articleFilter(
        kind: _kind,
        search: _search,
        unreadOnly: _unreadOnly,
        oldestFirst: _oldestFirst,
      );

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final currentCount = ref.watch(articleCountProvider(_filter));
    return Scaffold(
      drawer: _FilterDrawer(
        selected: _kind,
        savedOnly: widget.savedOnly,
        onSelected: (kind, label) {
          setState(() {
            _kind = kind;
            _selectionLabel = label;
            _revision++;
          });
          Navigator.of(context).pop();
        },
      ),
      appBar: AppBar(
        title: _searching
            ? TextField(
                controller: _searchController,
                autofocus: true,
                textInputAction: TextInputAction.search,
                decoration: InputDecoration(
                  hintText: l10n.searchArticles,
                  border: InputBorder.none,
                ),
                onSubmitted: (value) {
                  setState(() {
                    _search = value.trim().isEmpty ? null : value.trim();
                    _revision++;
                  });
                },
              )
            : Text(
                '${_selectionLabel ?? (widget.savedOnly ? l10n.savedTitle : l10n.articlesTitle)}'
                '${currentCount.asData == null ? '' : ' (${currentCount.asData!.value})'}',
              ),
        actions: [
          IconButton(
            tooltip: l10n.search,
            icon: Icon(_searching ? Icons.close : Icons.search),
            onPressed: () {
              setState(() {
                _searching = !_searching;
                if (!_searching) {
                  _searchController.clear();
                  _search = null;
                  _revision++;
                }
              });
            },
          ),
          PopupMenuButton<_ListOption>(
            tooltip: l10n.listOptions,
            onSelected: (option) {
              setState(() {
                switch (option) {
                  case _ListOption.hideRead:
                    _unreadOnly = !_unreadOnly;
                  case _ListOption.oldestFirst:
                    _oldestFirst = !_oldestFirst;
                }
                _revision++;
              });
            },
            itemBuilder: (_) => [
              CheckedPopupMenuItem(
                value: _ListOption.hideRead,
                checked: _unreadOnly,
                child: Text(l10n.hideRead),
              ),
              CheckedPopupMenuItem(
                value: _ListOption.oldestFirst,
                checked: _oldestFirst,
                child: Text(l10n.oldestFirst),
              ),
            ],
          ),
          IconButton(
            tooltip: l10n.markAllRead,
            icon: const Icon(Icons.done_all),
            onPressed: _markAllRead,
          ),
        ],
      ),
      body: ArticleCollectionView(
        key: ValueKey('articles-$_revision-${_filter.hashCode}'),
        filter: _filter,
        emptyText: widget.savedOnly ? l10n.noSavedArticles : l10n.noArticles,
        onArticleChanged: _refreshCounts,
      ),
    );
  }

  Future<void> _markAllRead() async {
    try {
      final count =
          await ref.read(articleRepositoryProvider).markAllRead(_filter);
      if (!mounted) return;
      _refreshCounts();
      setState(() => _revision++);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(context.l10n.markedAllRead(count))),
      );
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.localizeError(error))),
        );
      }
    }
  }

  void _refreshCounts() {
    ref.invalidate(articleCountsProvider);
    ref.invalidate(articleCountProvider);
  }
}

class _FilterDrawer extends ConsumerWidget {
  final bridge.ArticleFilterKind selected;
  final bool savedOnly;
  final void Function(bridge.ArticleFilterKind kind, String label) onSelected;

  const _FilterDrawer({
    required this.selected,
    required this.savedOnly,
    required this.onSelected,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = context.l10n;
    final counts = ref.watch(articleCountsProvider).asData?.value;
    final feeds =
        ref.watch(feedListProvider).asData?.value ?? const <bridge.Feed>[];
    final folders =
        ref.watch(folderListProvider).asData?.value ?? const <bridge.Folder>[];
    final tags = ref.watch(articleTagsProvider).asData?.value ??
        const <bridge.TagSummary>[];
    final items = <Widget>[
      DrawerHeader(
        margin: EdgeInsets.zero,
        child: Align(
          alignment: Alignment.bottomLeft,
          child: Text(l10n.smartViews,
              style: Theme.of(context).textTheme.headlineSmall),
        ),
      ),
      if (!savedOnly)
        _FilterTile(
          icon: Icons.article_outlined,
          label: l10n.allArticles,
          count: counts?.all.toInt(),
          selected: selected == const bridge.ArticleFilterKind.all(),
          onTap: () => onSelected(
              const bridge.ArticleFilterKind.all(), l10n.allArticles),
        ),
      if (!savedOnly)
        _FilterTile(
          icon: Icons.mark_email_unread_outlined,
          label: l10n.unreadArticles,
          count: counts?.unread.toInt(),
          selected: selected == const bridge.ArticleFilterKind.unread(),
          onTap: () => onSelected(
            const bridge.ArticleFilterKind.unread(),
            l10n.unreadArticles,
          ),
        ),
      _FilterTile(
        icon: Icons.star_outline,
        label: l10n.starredArticles,
        count: counts?.starred.toInt(),
        selected: selected == const bridge.ArticleFilterKind.starred(),
        onTap: () => onSelected(
          const bridge.ArticleFilterKind.starred(),
          l10n.starredArticles,
        ),
      ),
      _FilterTile(
        icon: Icons.bookmark_outline,
        label: l10n.readLaterArticles,
        count: counts?.readLater.toInt(),
        selected: selected == const bridge.ArticleFilterKind.readLater(),
        onTap: () => onSelected(
          const bridge.ArticleFilterKind.readLater(),
          l10n.readLaterArticles,
        ),
      ),
      if (!savedOnly && folders.isNotEmpty) ...[
        const Divider(),
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
          child: Text(l10n.folders),
        ),
        for (final folder in folders)
          _DynamicFilterTile(
            icon: Icons.folder_outlined,
            label: folder.name,
            kind: bridge.ArticleFilterKind.folder(folderId: folder.id),
            selected: selected,
            onSelected: onSelected,
          ),
      ],
      if (!savedOnly && feeds.isNotEmpty) ...[
        const Divider(),
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
          child: Text(l10n.navSubscriptions),
        ),
        for (final feed in feeds)
          _DynamicFilterTile(
            icon: Icons.rss_feed,
            label: feed.title,
            kind: bridge.ArticleFilterKind.feed(feedId: feed.id),
            selected: selected,
            onSelected: onSelected,
          ),
      ],
      if (!savedOnly && tags.isNotEmpty) ...[
        const Divider(),
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
          child: Text(l10n.tags),
        ),
        for (final tag in tags)
          _FilterTile(
            icon: Icons.label_outline,
            label: tag.name,
            count: tag.articleCount.toInt(),
            selected: selected == bridge.ArticleFilterKind.tag(tagId: tag.id),
            onTap: () => onSelected(
              bridge.ArticleFilterKind.tag(tagId: tag.id),
              tag.name,
            ),
          ),
      ],
    ];
    return Drawer(child: SafeArea(child: ListView(children: items)));
  }
}

class _DynamicFilterTile extends ConsumerWidget {
  final IconData icon;
  final String label;
  final bridge.ArticleFilterKind kind;
  final bridge.ArticleFilterKind selected;
  final void Function(bridge.ArticleFilterKind kind, String label) onSelected;

  const _DynamicFilterTile({
    required this.icon,
    required this.label,
    required this.kind,
    required this.selected,
    required this.onSelected,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final count = ref.watch(
      articleCountProvider(articleFilter(kind: kind, limit: 1)),
    );
    return _FilterTile(
      icon: icon,
      label: label,
      count: count.asData?.value,
      selected: selected == kind,
      onTap: () => onSelected(kind, label),
    );
  }
}

class _FilterTile extends StatelessWidget {
  final IconData icon;
  final String label;
  final int? count;
  final bool selected;
  final VoidCallback onTap;

  const _FilterTile({
    required this.icon,
    required this.label,
    required this.count,
    required this.selected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: Icon(icon),
      title: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis),
      trailing: count == null ? null : Text('$count'),
      selected: selected,
      onTap: onTap,
    );
  }
}

enum _ListOption { hideRead, oldestFirst }
