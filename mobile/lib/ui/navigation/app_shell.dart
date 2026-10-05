import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../l10n/l10n.dart';
import '../../repositories/feed_repository.dart';
import '../../services/platform_service.dart';
import '../screens/article_browser_screen.dart';
import '../screens/feed_list_screen.dart';
import '../screens/playback_screen.dart';
import '../screens/settings_screen.dart';

class AppShell extends ConsumerStatefulWidget {
  const AppShell({super.key});

  @override
  ConsumerState<AppShell> createState() => _AppShellState();
}

class _AppShellState extends ConsumerState<AppShell> {
  int _selectedIndex = 0;
  StreamSubscription<String>? _deepLinkSubscription;
  StreamSubscription<PlaybackState>? _playbackSubscription;
  PlaybackState _playbackState = PlaybackState.empty;

  @override
  void initState() {
    super.initState();
    _deepLinkSubscription = platformService.deepLinks.listen(_openDeepLink);
    _playbackSubscription = platformService.playbackStates.listen((state) {
      if (mounted) setState(() => _playbackState = state);
    });
    platformService.getPlaybackState().then((state) {
      if (mounted) setState(() => _playbackState = state);
    });
    platformService.getInitialDeepLink().then((link) {
      if (link != null) _openDeepLink(link);
    });
  }

  @override
  void dispose() {
    _deepLinkSubscription?.cancel();
    _playbackSubscription?.cancel();
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
                  Expanded(
                    child: Column(
                      children: [
                        Expanded(child: body),
                        if (_playbackState.mediaId.isNotEmpty)
                          _MiniPlayer(
                            state: _playbackState,
                            onOpen: _openPlayback,
                          ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          );
        }
        return Scaffold(
          body: body,
          bottomNavigationBar: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (_playbackState.mediaId.isNotEmpty)
                _MiniPlayer(state: _playbackState, onOpen: _openPlayback),
              NavigationBar(
                selectedIndex: _selectedIndex,
                onDestinationSelected: _select,
                destinations: destinations,
              ),
            ],
          ),
        );
      },
    );
  }

  void _select(int index) => setState(() => _selectedIndex = index);

  Future<void> _openPlayback() => Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => PlaybackScreen(initialState: _playbackState),
        ),
      );

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

class _MiniPlayer extends StatelessWidget {
  final PlaybackState state;
  final VoidCallback onOpen;

  const _MiniPlayer({required this.state, required this.onOpen});

  @override
  Widget build(BuildContext context) => Material(
        color: Theme.of(context).colorScheme.surfaceContainerHigh,
        child: ListTile(
          leading: const Icon(Icons.graphic_eq),
          title: Text(
            state.title,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          subtitle: Text(
            state.error == null
                ? state.source
                : _miniPlayerError(context, state.error!),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          onTap: onOpen,
          trailing: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              IconButton(
                tooltip: state.playing
                    ? context.l10n.pauseAudio
                    : context.l10n.playAudio,
                onPressed: () => platformService.playbackCommand(
                  state.playing ? 'pause' : 'play',
                ),
                icon: Icon(state.playing ? Icons.pause : Icons.play_arrow),
              ),
              IconButton(
                tooltip: context.l10n.stopPlayback,
                onPressed: () => platformService.playbackCommand('stop'),
                icon: const Icon(Icons.close),
              ),
            ],
          ),
        ),
      );
}

String _miniPlayerError(BuildContext context, String error) => switch (error) {
      'invalidPlaybackUrl' => context.l10n.errorInvalidPlaybackUrl,
      'playbackNetwork' => context.l10n.errorPlaybackNetwork,
      'playbackUnavailable' => context.l10n.errorPlaybackUnavailable,
      _ => context.l10n.errorPlaybackFailed,
    };
