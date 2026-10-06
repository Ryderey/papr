import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
import '../../repositories/github_sync_repository.dart';
import 'github_schedule_controls.dart';

class GithubSyncPanel extends ConsumerStatefulWidget {
  final bool otherConnected;
  const GithubSyncPanel({super.key, required this.otherConnected});
  @override
  ConsumerState<GithubSyncPanel> createState() => _GithubSyncPanelState();
}

class _GithubSyncPanelState extends ConsumerState<GithubSyncPanel> {
  final _owner = TextEditingController(),
      _repo = TextEditingController(),
      _branch = TextEditingController(),
      _token = TextEditingController();
  bridge.GithubPreview? _preview;
  bool _busy = false;
  String? _error;
  @override
  void dispose() {
    _owner.dispose();
    _repo.dispose();
    _branch.dispose();
    _token.dispose();
    super.dispose();
  }

  Future<void> _run(Future<void> Function() action) async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      await action();
    } catch (error) {
      if (mounted) setState(() => _error = context.l10n.localizeError(error));
    } finally {
      if (mounted) {
        setState(() => _busy = false);
        ref.invalidate(githubSyncStatusProvider);
      }
    }
  }

  Widget _input(TextEditingController controller, String label,
          {bool secret = false}) =>
      TextField(
          controller: controller,
          enabled: !_busy && _preview == null,
          decoration: InputDecoration(labelText: label),
          obscureText: secret,
          enableSuggestions: !secret,
          autocorrect: false);
  @override
  Widget build(BuildContext context) {
    final l = context.l10n,
        status = ref.watch(githubSyncStatusProvider),
        repository = ref.read(githubSyncRepositoryProvider);
    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      const Divider(height: 36),
      Text('GitHub', style: Theme.of(context).textTheme.titleMedium),
      const SizedBox(height: 8),
      Text(l.githubScope),
      if (_busy) const LinearProgressIndicator(),
      if (_error != null)
        Text(_error!,
            style: TextStyle(color: Theme.of(context).colorScheme.error)),
      status.when(
          loading: () => Text(l.githubLoading),
          error: (error, _) => Text(l.localizeError(error)),
          data: (value) {
            final profile = value.profile;
            if (profile != null) {
              return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const SizedBox(height: 12),
                    Text(
                        '${profile.owner}/${profile.repo} 路 ${profile.branch}'),
                    const GithubScheduleControls(),
                    Text(l.githubPending(value.pending.toInt())),
                    Text(l.githubLastSuccess(
                        value.lastSuccessAt ?? l.githubNever)),
                    if (value.rejected.toInt() > 0)
                      Text(l.githubRejected(value.rejected.toInt())),
                    if (value.uncertainPublication) Text(l.githubUncertain),
                    if (value.retryAt != null)
                      Text(l.githubRetryAt(value.retryAt!)),
                    if (value.lastErrorCode != null)
                      Text(l.localizeGithubCode(value.lastErrorCode!)),
                    Wrap(spacing: 8, children: [
                      FilledButton(
                          onPressed: _busy || value.busy
                              ? null
                              : () => _run(() async {
                                    await repository.syncNow();
                                  }),
                          child: Text(l.syncNow)),
                      if (_busy || value.busy)
                        TextButton(
                            onPressed: () => repository.cancel(),
                            child: Text(l.githubCancel)),
                      TextButton(
                          onPressed: _busy || value.busy
                              ? null
                              : () => _run(repository.disconnect),
                          child: Text(l.syncDisconnect))
                    ]),
                    _input(_token, l.githubToken, secret: true),
                    TextButton(
                        onPressed: _busy
                            ? null
                            : () => _run(() async {
                                  await repository.updateToken(_token.text);
                                  _token.clear();
                                }),
                        child: Text(l.githubUpdateToken)),
                  ]);
            }
            if (widget.otherConnected) {
              return Padding(
                  padding: const EdgeInsets.only(top: 12),
                  child: Text(l.githubOtherBackend));
            }
            return Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const SizedBox(height: 12),
                  Text(l.githubSetup),
                  _input(_owner, l.githubOwner),
                  _input(_repo, l.githubRepo),
                  _input(_branch, l.githubBranch),
                  _input(_token, l.githubToken, secret: true),
                  const SizedBox(height: 12),
                  if (_preview case final preview?) ...[
                    Text(l.githubPreviewCounts(
                        preview.localFeeds.toInt(),
                        preview.remoteFeeds.toInt(),
                        preview.localArticles.toInt(),
                        preview.remoteArticles.toInt())),
                    Text(l.githubPreviewExcluded(
                        preview.excludedFeeds.toInt(),
                        preview.excludedArticles.toInt(),
                        preview.warningCount.toInt())),
                    Text(l.githubPreviewHint),
                    Wrap(spacing: 8, children: [
                      FilledButton(
                          onPressed: _busy
                              ? null
                              : () => _run(() async {
                                    await repository.connect(
                                        preview, _token.text);
                                    _token.clear();
                                    if (mounted) {
                                      setState(() => _preview = null);
                                    }
                                  }),
                          child: Text(l.githubConfirm)),
                      TextButton(
                          onPressed: _busy
                              ? null
                              : () => setState(() => _preview = null),
                          child: Text(l.githubCancel))
                    ])
                  ] else
                    FilledButton(
                        onPressed: _busy
                            ? null
                            : () => _run(() async {
                                  final preview = await repository.preview(
                                      _owner.text,
                                      _repo.text,
                                      _branch.text,
                                      _token.text);
                                  if (mounted) {
                                    setState(() => _preview = preview);
                                  }
                                }),
                        child: Text(l.githubPreview)),
                ]);
          }),
    ]);
  }
}
