import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
import '../../repositories/github_sync_repository.dart';

class GithubScheduleControls extends ConsumerStatefulWidget {
  const GithubScheduleControls({super.key});
  @override
  ConsumerState<GithubScheduleControls> createState() =>
      _GithubScheduleControlsState();
}

class _GithubScheduleControlsState
    extends ConsumerState<GithubScheduleControls> {
  bool _saving = false;
  String? _error;
  bridge.GithubSchedule? _draft;
  bridge.GithubSchedule? _lastSaved;
  Future<void> _save(bridge.GithubSchedule value) async {
    setState(() {
      _saving = true;
      _error = null;
    });
    try {
      await ref.read(githubSyncRepositoryProvider).setSchedule(value);
      if (!mounted) return;
      final saved = await ref.read(githubScheduleProvider.future);
      if (saved != value) {
        if (mounted) {
          setState(() => _error = context.l10n.githubScheduleSaveFailed);
        }
        return;
      }
      if (mounted) {
        setState(() => _draft = null);
        ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text(context.l10n.githubScheduleSaved)));
      }
    } catch (error) {
      if (mounted) setState(() => _error = context.l10n.localizeError(error));
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final l = context.l10n;
    final schedule = ref.watch(githubScheduleProvider);
    final saved = schedule.asData?.value ?? _lastSaved;
    void retryRead() {
      setState(() => _error = null);
      ref.invalidate(githubScheduleProvider);
    }

    if (saved == null) {
      return Column(children: [
        if (schedule.hasError) ...[
          Text(l.localizeError(schedule.error!)),
          TextButton(onPressed: retryRead, child: Text(l.retry)),
        ] else
          const LinearProgressIndicator(),
      ]);
    }
    _lastSaved = saved;
    final value = _draft ?? saved;
    final changed = value != saved;
    void update({bool? enabled, int? delay, int? interval, int? background}) {
      setState(() => _draft = bridge.GithubSchedule(
          enabled: enabled ?? value.enabled,
          uploadDelaySecs: delay ?? value.uploadDelaySecs,
          cloudIntervalMinutes: interval ?? value.cloudIntervalMinutes,
          backgroundIntervalMinutes:
              background ?? value.backgroundIntervalMinutes));
    }

    Widget choice(String label, int selected, List<int> choices, bool seconds,
            void Function(int) change) =>
        DropdownButtonFormField<int>(
            key: ValueKey('$label:$selected:$_saving'),
            initialValue: selected,
            decoration: InputDecoration(labelText: label),
            items: choices
                .map((n) => DropdownMenuItem(
                    value: n,
                    child: Text(seconds ? l.githubSeconds(n) : l.minutes(n))))
                .toList(),
            onChanged: _saving || !value.enabled
                ? null
                : (n) {
                    if (n != null) change(n);
                  });
    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(l.githubAutomatic),
          value: value.enabled,
          onChanged: _saving ? null : (enabled) => update(enabled: enabled)),
      choice(l.githubUploadDelay, value.uploadDelaySecs, [10, 30, 60, 120],
          true, (n) => update(delay: n)),
      const SizedBox(height: 12),
      choice(l.githubCloudInterval, value.cloudIntervalMinutes,
          [5, 10, 15, 30, 60], false, (n) => update(interval: n)),
      const SizedBox(height: 12),
      choice(l.githubBackgroundInterval, value.backgroundIntervalMinutes,
          [15, 30, 60, 120, 360], false, (n) => update(background: n)),
      const SizedBox(height: 8),
      Text(changed
          ? l.githubScheduleUnsaved
          : saved.enabled
              ? l.githubScheduleHint
              : l.githubManualOnly),
      if (saved.enabled) Text(l.githubBackgroundHint),
      const SizedBox(height: 8),
      FilledButton(
          onPressed: _saving || !changed ? null : () => _save(value),
          child: Text(l.save)),
      if (_error != null || schedule.hasError)
        Text(_error ?? l.localizeError(schedule.error!),
            style: TextStyle(color: Theme.of(context).colorScheme.error)),
      if (schedule.hasError)
        TextButton(onPressed: _saving ? null : retryRead, child: Text(l.retry)),
    ]);
  }
}
