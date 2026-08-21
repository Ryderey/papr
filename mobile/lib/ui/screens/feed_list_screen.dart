import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
import '../../repositories/feed_repository.dart';
import '../../services/platform_service.dart';
import 'article_list_screen.dart';

final feedListProvider = FutureProvider<List<bridge.Feed>>((ref) async {
  final repo = ref.watch(feedRepositoryProvider);
  return repo.listFeeds();
});

final folderListProvider = FutureProvider<List<bridge.Folder>>((ref) async {
  final repo = ref.watch(feedRepositoryProvider);
  return repo.listFolders();
});

class FeedListScreen extends ConsumerStatefulWidget {
  const FeedListScreen({super.key});

  @override
  ConsumerState<FeedListScreen> createState() => _FeedListScreenState();
}

class _FeedListScreenState extends ConsumerState<FeedListScreen> {
  bridge.Feed? _selectedFeed;

  @override
  Widget build(BuildContext context) {
    final feeds = ref.watch(feedListProvider);
    final folders = ref.watch(folderListProvider).asData?.value ?? const [];
    final l10n = context.l10n;

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.subscriptionsTitle),
        actions: [
          IconButton(
            icon: const Icon(Icons.folder_outlined),
            tooltip: l10n.manageFolders,
            onPressed: () => showDialog<void>(
              context: context,
              builder: (_) => const _FolderManagerDialog(),
            ),
          ),
          PopupMenuButton<_OpmlAction>(
            tooltip: l10n.opml,
            icon: const Icon(Icons.import_export),
            onSelected: (action) => _handleOpml(context, ref, action),
            itemBuilder: (context) => [
              PopupMenuItem(
                value: _OpmlAction.importDocument,
                child: Text(l10n.importOpml),
              ),
              PopupMenuItem(
                value: _OpmlAction.exportDocument,
                child: Text(l10n.exportOpml),
              ),
            ],
          ),
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: l10n.refreshTooltip,
            onPressed: () => _refresh(context, ref),
          ),
        ],
      ),
      body: LayoutBuilder(
        builder: (context, constraints) => feeds.when(
          data: (items) => _buildContent(
            context,
            ref,
            items,
            folders: folders,
            tablet: constraints.maxWidth >= 600,
          ),
          loading: () => const Center(child: CircularProgressIndicator()),
          error: (err, stack) => Center(child: Text(l10n.localizeError(err))),
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _openAddFeedDialog(context, ref),
        tooltip: l10n.addFeed,
        child: const Icon(Icons.add),
      ),
    );
  }

  Widget _buildContent(
    BuildContext context,
    WidgetRef ref,
    List<bridge.Feed> items, {
    required List<bridge.Folder> folders,
    required bool tablet,
  }) {
    final selected = _selectedFeed == null
        ? null
        : items.where((feed) => feed.id == _selectedFeed!.id).firstOrNull;
    final list = _buildFeedList(
      context,
      ref,
      items,
      folders: folders,
      tablet: tablet,
    );
    if (!tablet) return list;
    return Row(
      children: [
        SizedBox(width: 320, child: list),
        const VerticalDivider(width: 1),
        Expanded(
          child: selected == null
              ? Center(child: Text(context.l10n.selectSubscription))
              : ArticleListScreen(feed: selected),
        ),
      ],
    );
  }

  Widget _buildFeedList(
    BuildContext context,
    WidgetRef ref,
    List<bridge.Feed> items, {
    required List<bridge.Folder> folders,
    required bool tablet,
  }) {
    if (items.isEmpty) return Center(child: Text(context.l10n.noFeeds));
    return RefreshIndicator(
      onRefresh: () => _refresh(context, ref),
      child: ListView.builder(
        itemCount: items.length,
        itemBuilder: (context, index) {
          final feed = items[index];
          final folder = feed.folderId == null
              ? null
              : folders.where((item) => item.id == feed.folderId).firstOrNull;
          return ListTile(
            selected: tablet && _selectedFeed?.id == feed.id,
            title: Text(feed.title),
            subtitle: Text(
              '${folder?.name ?? context.l10n.uncategorized}\n${feed.feedUrl}',
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
            isThreeLine: true,
            trailing: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (feed.unreadCount > 0)
                  Badge(
                    label: Text('${feed.unreadCount}'),
                    child: const Icon(Icons.rss_feed),
                  )
                else
                  const Icon(Icons.rss_feed),
                PopupMenuButton<_FeedAction>(
                  tooltip: context.l10n.feedActions,
                  onSelected: (action) =>
                      _handleFeedAction(context, ref, feed, folders, action),
                  itemBuilder: (context) => [
                    PopupMenuItem(
                      value: _FeedAction.refresh,
                      child: Text(context.l10n.refreshTooltip),
                    ),
                    PopupMenuItem(
                      value: _FeedAction.rename,
                      child: Text(context.l10n.rename),
                    ),
                    PopupMenuItem(
                      value: _FeedAction.move,
                      child: Text(context.l10n.moveToFolder),
                    ),
                    PopupMenuItem(
                      value: _FeedAction.interval,
                      child: Text(context.l10n.refreshIntervalTitle),
                    ),
                    PopupMenuItem(
                      value: _FeedAction.delete,
                      child: Text(context.l10n.delete),
                    ),
                  ],
                ),
              ],
            ),
            onTap: () {
              if (tablet) {
                setState(() => _selectedFeed = feed);
              } else {
                Navigator.of(context).push(
                  MaterialPageRoute(
                    builder: (_) => ArticleListScreen(feed: feed),
                  ),
                );
              }
            },
          );
        },
      ),
    );
  }

  Future<void> _refresh(BuildContext context, WidgetRef ref) async {
    final messenger = ScaffoldMessenger.of(context);
    final repo = ref.read(feedRepositoryProvider);
    try {
      final report = await repo.refreshFeeds();
      if (context.mounted) {
        final errorCount = report.errors.length;
        final msg = errorCount == 0
            ? context.l10n.refreshComplete(report.newArticles.toInt())
            : context.l10n.refreshPartial(
                report.newArticles.toInt(),
                errorCount,
              );
        messenger.showSnackBar(SnackBar(content: Text(msg)));
      }
    } catch (e) {
      if (context.mounted) {
        messenger.showSnackBar(
          SnackBar(content: Text(context.l10n.refreshFailed(e.toString()))),
        );
      }
    } finally {
      ref.invalidate(feedListProvider);
      ref.invalidate(articleListProvider);
    }
  }

  Future<void> _openAddFeedDialog(BuildContext context, WidgetRef ref) async {
    final added = await showAddFeedDialog(context);
    if (added == true) {
      ref.invalidate(feedListProvider);
    }
  }

  Future<void> _handleFeedAction(
    BuildContext context,
    WidgetRef ref,
    bridge.Feed feed,
    List<bridge.Folder> folders,
    _FeedAction action,
  ) async {
    final repo = ref.read(feedRepositoryProvider);
    try {
      switch (action) {
        case _FeedAction.refresh:
          final report = await repo.refreshFeed(feed);
          if (context.mounted) {
            _showMessage(
              context,
              context.l10n.refreshOneComplete(
                feed.title,
                report.newArticles.toInt(),
              ),
            );
          }
          break;
        case _FeedAction.rename:
          final title = await _showTextDialog(
            context,
            title: context.l10n.renameFeed,
            label: context.l10n.feedTitle,
            initialValue: feed.title,
          );
          if (title != null) await repo.renameFeed(feed, title);
          break;
        case _FeedAction.move:
          final result = await showDialog<_FolderSelection>(
            context: context,
            builder: (context) => SimpleDialog(
              title: Text(context.l10n.moveToFolder),
              children: [
                SimpleDialogOption(
                  onPressed: () => Navigator.pop(
                    context,
                    const _FolderSelection(null),
                  ),
                  child: Text(context.l10n.uncategorized),
                ),
                for (final folder in folders)
                  SimpleDialogOption(
                    onPressed: () => Navigator.pop(
                      context,
                      _FolderSelection(folder),
                    ),
                    child: Text(folder.name),
                  ),
              ],
            ),
          );
          if (result != null) await repo.moveFeed(feed, result.folder);
          break;
        case _FeedAction.interval:
          final choice = await showDialog<_IntervalChoice>(
            context: context,
            builder: (context) => SimpleDialog(
              title: Text(context.l10n.refreshIntervalTitle),
              children: [
                SimpleDialogOption(
                  onPressed: () => Navigator.pop(
                    context,
                    _IntervalChoice.global,
                  ),
                  child: Text(context.l10n.useGlobalInterval),
                ),
                for (final choice in _IntervalChoice.values.skip(1))
                  SimpleDialogOption(
                    onPressed: () => Navigator.pop(context, choice),
                    child: Text(
                      choice == _IntervalChoice.off
                          ? context.l10n.refreshOff
                          : context.l10n.minutes(choice.minutes!),
                    ),
                  ),
              ],
            ),
          );
          if (choice != null) {
            await repo.setFeedRefreshInterval(feed, choice.minutes);
          }
          break;
        case _FeedAction.delete:
          final confirmed = await _confirm(
            context,
            title: context.l10n.deleteFeedTitle,
            message: context.l10n.deleteFeedMessage(feed.title),
          );
          if (confirmed) await repo.deleteFeed(feed);
          break;
      }
    } catch (error) {
      if (context.mounted) {
        _showMessage(context, context.l10n.localizeError(error));
      }
    } finally {
      ref.invalidate(feedListProvider);
      ref.invalidate(articleListProvider);
    }
  }

  Future<void> _handleOpml(
    BuildContext context,
    WidgetRef ref,
    _OpmlAction action,
  ) async {
    final repo = ref.read(feedRepositoryProvider);
    try {
      switch (action) {
        case _OpmlAction.importDocument:
          final text = await platformService.openOpmlDocument();
          if (text == null) return;
          if (text.trim().isEmpty) {
            if (context.mounted) {
              _showMessage(context, context.l10n.opmlRequired);
            }
            return;
          }
          final report = await repo.importOpml(text);
          if (context.mounted) {
            _showMessage(
              context,
              context.l10n.opmlImported(
                report.importedFeeds.toInt(),
                report.failedFeeds.toInt(),
              ),
            );
          }
          break;
        case _OpmlAction.exportDocument:
          final text = await repo.exportOpml();
          final saved = await platformService.saveOpmlDocument(text);
          if (saved && context.mounted) {
            _showMessage(context, context.l10n.opmlExported);
          }
          break;
      }
    } catch (error) {
      if (context.mounted) {
        _showMessage(context, context.l10n.localizeError(error));
      }
    } finally {
      ref.invalidate(feedListProvider);
      ref.invalidate(folderListProvider);
    }
  }
}

Future<bool?> showAddFeedDialog(
  BuildContext context, {
  String? initialUrl,
}) {
  return showDialog<bool>(
    context: context,
    builder: (_) => AddFeedDialog(initialUrl: initialUrl),
  );
}

class AddFeedDialog extends ConsumerStatefulWidget {
  final String? initialUrl;

  const AddFeedDialog({super.key, this.initialUrl});

  @override
  ConsumerState<AddFeedDialog> createState() => _AddFeedDialogState();
}

class _AddFeedDialogState extends ConsumerState<AddFeedDialog> {
  late final TextEditingController _controller;
  final _searchController = TextEditingController();
  List<bridge.DiscoveryResult>? _results;
  bool _isAdding = false;
  bool _isSearching = false;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.initialUrl);
  }

  @override
  void dispose() {
    _controller.dispose();
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(context.l10n.addFeed),
      content: SizedBox(
        width: 520,
        height: (MediaQuery.sizeOf(context).height * 0.48)
            .clamp(180.0, 420.0)
            .toDouble(),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              controller: _controller,
              decoration: InputDecoration(
                labelText: context.l10n.feedUrl,
                hintText: context.l10n.feedUrlHint,
              ),
              keyboardType: TextInputType.url,
              autofocus: true,
              enabled: !_isAdding,
              onSubmitted: (_) => _add(),
            ),
            const SizedBox(height: 20),
            Text(
              context.l10n.directory,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            TextField(
              controller: _searchController,
              decoration: InputDecoration(
                hintText: context.l10n.directoryHint,
                suffixIcon: _isSearching
                    ? const Padding(
                        padding: EdgeInsets.all(12),
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : IconButton(
                        tooltip: context.l10n.search,
                        icon: const Icon(Icons.search),
                        onPressed: _search,
                      ),
              ),
              textInputAction: TextInputAction.search,
              onSubmitted: (_) => _search(),
            ),
            const SizedBox(height: 8),
            Expanded(
              child: _results == null
                  ? const SizedBox.shrink()
                  : _results!.isEmpty
                      ? Center(child: Text(context.l10n.noResults))
                      : ListView.builder(
                          itemCount: _results!.length,
                          itemBuilder: (context, index) {
                            final result = _results![index];
                            return ListTile(
                              title: Text(result.title),
                              subtitle: result.description == null
                                  ? Text(result.feedUrl)
                                  : Text(result.description!),
                              onTap:
                                  _isAdding ? null : () => _add(result.feedUrl),
                            );
                          },
                        ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: _isAdding ? null : () => Navigator.of(context).pop(false),
          child: Text(context.l10n.cancel),
        ),
        TextButton(
          onPressed: _isAdding ? null : _add,
          child: _isAdding
              ? const SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : Text(context.l10n.add),
        ),
      ],
    );
  }

  Future<void> _add([String? selectedUrl]) async {
    final url = (selectedUrl ?? _controller.text).trim();
    if (url.isEmpty) {
      _showError(context.l10n.feedUrlRequired);
      return;
    }

    setState(() => _isAdding = true);

    try {
      final repo = ref.read(feedRepositoryProvider);
      await repo.addFeed(url);
      if (mounted) {
        Navigator.of(context).pop(true);
      }
    } catch (e) {
      if (mounted) {
        setState(() => _isAdding = false);
        _showError(context.l10n.localizeError(e));
      }
    }
  }

  Future<void> _search() async {
    setState(() => _isSearching = true);
    try {
      final language = Localizations.localeOf(context).languageCode;
      final results = await ref
          .read(feedRepositoryProvider)
          .searchDirectory(_searchController.text, language);
      if (mounted) setState(() => _results = results);
    } catch (error) {
      if (mounted) _showError(context.l10n.localizeError(error));
    } finally {
      if (mounted) setState(() => _isSearching = false);
    }
  }

  void _showError(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
  }
}

class _FolderManagerDialog extends ConsumerWidget {
  const _FolderManagerDialog();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final folders = ref.watch(folderListProvider);
    return AlertDialog(
      title: Text(context.l10n.manageFolders),
      content: SizedBox(
        width: 420,
        height: (MediaQuery.sizeOf(context).height * 0.45)
            .clamp(160.0, 360.0)
            .toDouble(),
        child: folders.when(
          data: (items) => items.isEmpty
              ? Center(child: Text(context.l10n.uncategorized))
              : ReorderableListView.builder(
                  itemCount: items.length,
                  onReorderItem: (oldIndex, newIndex) =>
                      _reorder(context, ref, items, oldIndex, newIndex),
                  itemBuilder: (context, index) {
                    final folder = items[index];
                    return ListTile(
                      key: ValueKey(folder.id),
                      leading: const Icon(Icons.folder_outlined),
                      title: Text(folder.name),
                      trailing: PopupMenuButton<_FolderAction>(
                        onSelected: (action) =>
                            _act(context, ref, folder, action),
                        itemBuilder: (context) => [
                          PopupMenuItem(
                            value: _FolderAction.rename,
                            child: Text(context.l10n.rename),
                          ),
                          PopupMenuItem(
                            value: _FolderAction.delete,
                            child: Text(context.l10n.delete),
                          ),
                        ],
                      ),
                    );
                  },
                ),
          loading: () => const Center(child: CircularProgressIndicator()),
          error: (error, _) => Center(
            child: Text(context.l10n.localizeError(error)),
          ),
        ),
      ),
      actions: [
        TextButton.icon(
          icon: const Icon(Icons.create_new_folder_outlined),
          label: Text(context.l10n.createFolder),
          onPressed: () => _create(context, ref),
        ),
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(context.l10n.close),
        ),
      ],
    );
  }

  Future<void> _create(BuildContext context, WidgetRef ref) async {
    final name = await _showTextDialog(
      context,
      title: context.l10n.createFolder,
      label: context.l10n.folderName,
      confirmLabel: context.l10n.createFolder,
    );
    if (name == null) return;
    if (!context.mounted) return;
    await _run(context, ref, () {
      return ref.read(feedRepositoryProvider).createFolder(name);
    });
  }

  Future<void> _reorder(
    BuildContext context,
    WidgetRef ref,
    List<bridge.Folder> folders,
    int oldIndex,
    int newIndex,
  ) async {
    final reordered = [...folders];
    final moved = reordered.removeAt(oldIndex);
    reordered.insert(newIndex, moved);
    await _run(context, ref, () {
      return ref.read(feedRepositoryProvider).reorderFolders(reordered);
    });
  }

  Future<void> _act(
    BuildContext context,
    WidgetRef ref,
    bridge.Folder folder,
    _FolderAction action,
  ) async {
    switch (action) {
      case _FolderAction.rename:
        final name = await _showTextDialog(
          context,
          title: context.l10n.rename,
          label: context.l10n.folderName,
          initialValue: folder.name,
        );
        if (name != null && context.mounted) {
          await _run(context, ref, () {
            return ref.read(feedRepositoryProvider).renameFolder(folder, name);
          });
        }
        break;
      case _FolderAction.delete:
        final confirmed = await _confirm(
          context,
          title: context.l10n.deleteFolderTitle,
          message: context.l10n.deleteFolderMessage,
        );
        if (confirmed && context.mounted) {
          await _run(context, ref, () {
            return ref.read(feedRepositoryProvider).deleteFolder(folder);
          });
        }
        break;
    }
  }

  Future<void> _run(
    BuildContext context,
    WidgetRef ref,
    Future<void> Function() operation,
  ) async {
    try {
      await operation();
    } catch (error) {
      if (context.mounted) {
        _showMessage(context, context.l10n.localizeError(error));
      }
    } finally {
      ref.invalidate(folderListProvider);
      ref.invalidate(feedListProvider);
    }
  }
}

enum _FeedAction { refresh, rename, move, interval, delete }

enum _FolderAction { rename, delete }

enum _OpmlAction { importDocument, exportDocument }

enum _IntervalChoice {
  global(null),
  minutes15(15),
  minutes30(30),
  minutes60(60),
  minutes240(240),
  off(525600);

  final int? minutes;

  const _IntervalChoice(this.minutes);
}

class _FolderSelection {
  final bridge.Folder? folder;

  const _FolderSelection(this.folder);
}

Future<String?> _showTextDialog(
  BuildContext context, {
  required String title,
  required String label,
  String? initialValue,
  String? confirmLabel,
}) async {
  final controller = TextEditingController(text: initialValue);
  final value = await showDialog<String>(
    context: context,
    builder: (context) => AlertDialog(
      title: Text(title),
      content: TextField(
        controller: controller,
        autofocus: true,
        decoration: InputDecoration(labelText: label),
        onSubmitted: (value) => Navigator.pop(context, value),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(context.l10n.cancel),
        ),
        TextButton(
          onPressed: () => Navigator.pop(context, controller.text),
          child: Text(confirmLabel ?? context.l10n.rename),
        ),
      ],
    ),
  );
  controller.dispose();
  return value;
}

Future<bool> _confirm(
  BuildContext context, {
  required String title,
  required String message,
}) async {
  return await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text(title),
          content: Text(message),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: Text(context.l10n.cancel),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, true),
              child: Text(context.l10n.delete),
            ),
          ],
        ),
      ) ??
      false;
}

void _showMessage(BuildContext context, String message) {
  ScaffoldMessenger.of(context).showSnackBar(
    SnackBar(content: Text(message)),
  );
}
