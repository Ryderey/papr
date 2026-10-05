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
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    if (Platform.isAndroid) {
      _start();
    }
  }

  void _start() {
    _timer?.cancel();
    _timer = Timer.periodic(const Duration(seconds: 10), (_) => _tick());
    _tick();
  }

  Future<void> _tick() async {
    if (_running || !mounted) return;
    _running = true;
    try {
      final repository = ref.read(githubSyncRepositoryProvider);
      final status = await repository.status();
      if (mounted && status.profile != null && status.automaticDue) {
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
