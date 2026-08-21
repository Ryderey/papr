import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../l10n/l10n.dart';
import '../../repositories/feed_repository.dart';
import '../../services/platform_service.dart';
import '../screens/article_browser_screen.dart';
import '../screens/feed_list_screen.dart';
import '../screens/settings_screen.dart';

class AppShell extends ConsumerStatefulWidget {
  const AppShell({super.key});

  @override
  ConsumerState<AppShell> createState() => _AppShellState();
}

class _AppShellState extends ConsumerState<AppShell> {
  int _selectedIndex = 0;
  StreamSubscription<String>? _deepLinkSubscription;

  @override
  void initState() {
    super.initState();
    _deepLinkSubscription = platformService.deepLinks.listen(_openDeepLink);
    platformService.getInitialDeepLink().then((link) {
      if (link != null) _openDeepLink(link);
    });
  }

  @override
  void dispose() {
    _deepLinkSubscription?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final destinations = [
      NavigationDestination(
        icon: const Icon(Icons.article_outlined),
        selectedIcon: const Icon(Icons.article),
        label: l10n.navArticles,
      ),
      NavigationDestination(
        icon: const Icon(Icons.rss_feed_outlined),
        selectedIcon: const Icon(Icons.rss_feed),
        label: l10n.navSubscriptions,
      ),
      NavigationDestination(
        icon: const Icon(Icons.bookmark_outline),
        selectedIcon: const Icon(Icons.bookmark),
        label: l10n.navSaved,
      ),
      NavigationDestination(
        icon: const Icon(Icons.settings_outlined),
        selectedIcon: const Icon(Icons.settings),
        label: l10n.navSettings,
      ),
    ];
    final pages = const [
      ArticleBrowserScreen(savedOnly: false),
      FeedListScreen(),
      ArticleBrowserScreen(savedOnly: true),
      SettingsScreen(),
    ];

    return LayoutBuilder(
      builder: (context, constraints) {
        final tablet = constraints.maxWidth >= 600;
        final body = IndexedStack(index: _selectedIndex, children: pages);
        if (tablet) {
          return Scaffold(
            body: SafeArea(
              child: Row(
                children: [
                  NavigationRail(
                    selectedIndex: _selectedIndex,
                    extended: constraints.maxWidth >= 900,
                    labelType: constraints.maxWidth >= 900
                        ? NavigationRailLabelType.none
                        : NavigationRailLabelType.all,
                    onDestinationSelected: _select,
                    destinations: [
                      for (final destination in destinations)
                        NavigationRailDestination(
                          icon: destination.icon,
                          selectedIcon: destination.selectedIcon,
                          label: Text(destination.label),
                        ),
                    ],
                  ),
                  const VerticalDivider(width: 1),
                  Expanded(child: body),
                ],
              ),
            ),
          );
        }
        return Scaffold(
          body: body,
          bottomNavigationBar: NavigationBar(
            selectedIndex: _selectedIndex,
            onDestinationSelected: _select,
            destinations: destinations,
          ),
        );
      },
    );
  }

  void _select(int index) => setState(() => _selectedIndex = index);

  Future<void> _openDeepLink(String link) async {
    try {
      final target = await ref.read(feedRepositoryProvider).parseDeepLink(link);
      if (!mounted || target == null) return;
      setState(() => _selectedIndex = 1);
      await Future<void>.delayed(Duration.zero);
      if (!mounted) return;
      final added = await showAddFeedDialog(context, initialUrl: target);
      if (added == true) ref.invalidate(feedListProvider);
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.localizeError(error))),
        );
      }
    }
  }
}
