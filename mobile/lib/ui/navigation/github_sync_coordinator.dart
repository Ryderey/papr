import 'dart:async';
import 'dart:io';
import 'package:flutter/widgets.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../repositories/github_sync_repository.dart';

/// The shared Core decides debounce and polling deadlines; Flutter only wakes
/// it while the Android application is visible.
class GithubSyncCoordinator extends ConsumerStatefulWidget {
  final Widget child;
  const GithubSyncCoordinator({super.key, required this.child});
  @override
  ConsumerState<GithubSyncCoordinator> createState() =>
      _GithubSyncCoordinatorState();
}

class _GithubSyncCoordinatorState extends ConsumerState<GithubSyncCoordinator>
    with WidgetsBindingObserver {
  Timer? _timer;
  bool _running = false;
  bool _visible = true;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    final lifecycle = WidgetsBinding.instance.lifecycleState;
    _visible = lifecycle == null || lifecycle == AppLifecycleState.resumed;
    if (Platform.isAndroid && _visible) {
      _start();
    }
  }

  void _start() {
    _timer?.cancel();
    _timer = Timer.periodic(const Duration(seconds: 10), (_) => _tick());
    _tick();
  }

  Future<void> _tick() async {
    if (_running || !mounted || !_visible) return;
    _running = true;
    try {
      final repository = ref.read(githubSyncRepositoryProvider);
      final due = await repository.automaticDue();
      if (mounted && _visible && due) {
        await repository.syncNow();
      }
    } catch (_) {
      /* Local reads stay available; settings show the stable sync error. */
    } finally {
      _running = false;
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (!Platform.isAndroid) return;
    _visible = state == AppLifecycleState.resumed;
    if (state == AppLifecycleState.resumed) {
      _start();
    } else {
      _timer?.cancel();
    }
  }

  @override
  void dispose() {
    _timer?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}
