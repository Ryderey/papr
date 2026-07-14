import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../repositories/feed_repository.dart';
import 'article_list_screen.dart';

final feedListProvider = FutureProvider<List<bridge.Feed>>((ref) async {
  final repo = ref.watch(feedRepositoryProvider);
  return repo.listFeeds();
});

class FeedListScreen extends ConsumerWidget {
  const FeedListScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final feeds = ref.watch(feedListProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Feeds'),
      ),
      body: feeds.when(
        data: (items) => items.isEmpty
            ? const Center(child: Text('No feeds yet. Tap + to add one.'))
            : ListView.builder(
                itemCount: items.length,
                itemBuilder: (context, index) {
                  final feed = items[index];
                  return ListTile(
                    title: Text(feed.title),
                    subtitle: Text(feed.feedUrl),
                    trailing: feed.unreadCount > 0
                        ? Badge(
                            label: Text('${feed.unreadCount}'),
                            child: const Icon(Icons.rss_feed),
                          )
                        : const Icon(Icons.rss_feed),
                    onTap: () {
                      Navigator.of(context).push(
                        MaterialPageRoute(
                          builder: (_) => ArticleListScreen(feed: feed),
                        ),
                      );
                    },
                  );
                },
              ),
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, stack) => Center(child: Text('Error: $err')),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showAddFeedDialog(context, ref),
        child: const Icon(Icons.add),
      ),
    );
  }

  Future<void> _showAddFeedDialog(BuildContext context, WidgetRef ref) async {
    final added = await showDialog<bool>(
      context: context,
      builder: (_) => const _AddFeedDialog(),
    );
    if (added == true) {
      ref.invalidate(feedListProvider);
    }
  }
}

class _AddFeedDialog extends ConsumerStatefulWidget {
  const _AddFeedDialog();

  @override
  ConsumerState<_AddFeedDialog> createState() => _AddFeedDialogState();
}

class _AddFeedDialogState extends ConsumerState<_AddFeedDialog> {
  final _controller = TextEditingController();
  bool _isAdding = false;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Add Feed'),
      content: TextField(
        controller: _controller,
        decoration: const InputDecoration(
          labelText: 'Feed URL',
          hintText: 'https://example.com/feed.xml',
        ),
        keyboardType: TextInputType.url,
        autofocus: true,
        enabled: !_isAdding,
        onSubmitted: (_) => _add(),
      ),
      actions: [
        TextButton(
          onPressed: _isAdding ? null : () => Navigator.of(context).pop(false),
          child: const Text('Cancel'),
        ),
        TextButton(
          onPressed: _isAdding ? null : _add,
          child: _isAdding
              ? const SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Text('Add'),
        ),
      ],
    );
  }

  Future<void> _add() async {
    final url = _controller.text.trim();
    if (url.isEmpty) {
      _showError('Please enter a feed URL');
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
        _showError(e.toString());
      }
    }
  }

  void _showError(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
  }
}
