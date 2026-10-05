import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
import '../../repositories/sync_repository.dart';

class SyncSettingsScreen extends ConsumerStatefulWidget {
  const SyncSettingsScreen({super.key});

  @override
  ConsumerState<SyncSettingsScreen> createState() => _SyncSettingsScreenState();
}

class _SyncSettingsScreenState extends ConsumerState<SyncSettingsScreen> {
  bool _busy = false;

  @override
  Widget build(BuildContext context) {
    final status = ref.watch(syncStatusProvider);
    final l10n = context.l10n;
    return Scaffold(
      appBar: AppBar(title: Text(l10n.syncTitle)),
      body: status.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, _) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(l10n.localizeError(error)),
              TextButton(
                onPressed: () => ref.invalidate(syncStatusProvider),
                child: Text(l10n.retry),
              ),
            ],
          ),
        ),
        data: (value) => ListView(
          padding: const EdgeInsets.all(16),
          children: [
            if (value.profile != null) ...[
              Text(l10n.syncConnected,
                  style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 8),
              Text('${value.profile!.username} · ${value.profile!.serverUrl}'),
              const SizedBox(height: 8),
              Text(value.lastSuccessAt == null
                  ? l10n.syncNeverCompleted
                  : l10n.syncLastSuccess(_localTime(value.lastSuccessAt!))),
              if (value.lastErrorCode != null)
                Text(l10n.localizeSyncCode(value.lastErrorCode!)),
              const SizedBox(height: 12),
              Wrap(
                spacing: 8,
                children: [
                  FilledButton(
                    onPressed: _busy ? null : _syncNow,
                    child: Text(value.lastErrorCode == null
                        ? l10n.syncNow
                        : l10n.retry),
                  ),
                  TextButton(
                    onPressed: _busy ? null : _disconnect,
                    child: Text(l10n.syncDisconnect),
                  ),
                ],
              ),
              const Divider(height: 36),
              Text(l10n.syncReplaceConnection,
                  style: Theme.of(context).textTheme.titleMedium),
            ] else
              Text(l10n.syncNotConnected,
                  style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 12),
            _ConnectionForm(
              key: ValueKey(value.profile?.credentialRef),
              existing: value.profile,
              enabled: !_busy,
              onBusyChanged: (busy) => setState(() => _busy = busy),
              onChanged: () => ref.invalidate(syncStatusProvider),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _syncNow() async {
    setState(() => _busy = true);
    try {
      await ref.read(syncRepositoryProvider).syncNow();
      ref.invalidate(syncStatusProvider);
      if (mounted) _show(context.l10n.syncCompleted);
    } catch (error) {
      ref.invalidate(syncStatusProvider);
      if (mounted) _show(context.l10n.localizeError(error));
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _disconnect() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(context.l10n.syncDisconnect),
        content: Text(context.l10n.syncDisconnectConfirm),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(context.l10n.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(context.l10n.syncDisconnect),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;
    setState(() => _busy = true);
    try {
      await ref.read(syncRepositoryProvider).disconnect();
      ref.invalidate(syncStatusProvider);
    } catch (error) {
      ref.invalidate(syncStatusProvider);
      if (mounted) _show(context.l10n.localizeError(error));
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _show(String message) => ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(message)),
      );

  String _localTime(String value) {
    final parsed = DateTime.tryParse(value);
    return parsed == null
        ? value
        : parsed.toLocal().toString().substring(0, 16);
  }
}

class _ConnectionForm extends ConsumerStatefulWidget {
  final bridge.SyncProfile? existing;
  final bool enabled;
  final ValueChanged<bool> onBusyChanged;
  final VoidCallback onChanged;

  const _ConnectionForm({
    super.key,
    required this.existing,
    required this.enabled,
    required this.onBusyChanged,
    required this.onChanged,
  });

  @override
  ConsumerState<_ConnectionForm> createState() => _ConnectionFormState();
}

class _ConnectionFormState extends ConsumerState<_ConnectionForm> {
  final _formKey = GlobalKey<FormState>();
  late bridge.SyncProvider _provider =
      widget.existing?.provider ?? bridge.SyncProvider.freshRss;
  late final _server = TextEditingController(text: widget.existing?.serverUrl);
  late final _username = TextEditingController(text: widget.existing?.username);
  final _credential = TextEditingController();
  bool _busy = false;

  @override
  void dispose() {
    _server.dispose();
    _username.dispose();
    _credential.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Form(
      key: _formKey,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          DropdownButtonFormField<bridge.SyncProvider>(
            initialValue: _provider,
            decoration: InputDecoration(labelText: l10n.syncProvider),
            items: const [
              DropdownMenuItem(
                  value: bridge.SyncProvider.freshRss, child: Text('FreshRSS')),
              DropdownMenuItem(
                  value: bridge.SyncProvider.miniflux, child: Text('Miniflux')),
            ],
            onChanged: _busy || !widget.enabled
                ? null
                : (value) => setState(() => _provider = value!),
          ),
          TextFormField(
            controller: _server,
            enabled: widget.enabled && !_busy,
            decoration: InputDecoration(labelText: l10n.syncServerUrl),
            keyboardType: TextInputType.url,
            autocorrect: false,
            validator: _required,
          ),
          TextFormField(
            controller: _username,
            enabled: widget.enabled && !_busy,
            decoration: InputDecoration(labelText: l10n.syncUsername),
            autocorrect: false,
            validator: _required,
          ),
          TextFormField(
            controller: _credential,
            enabled: widget.enabled && !_busy,
            decoration: InputDecoration(labelText: l10n.syncCredential),
            obscureText: true,
            enableSuggestions: false,
            autocorrect: false,
            validator: _required,
          ),
          const SizedBox(height: 16),
          if (_busy) const LinearProgressIndicator(),
          Wrap(
            spacing: 8,
            children: [
              OutlinedButton(
                onPressed: _busy || !widget.enabled
                    ? null
                    : () => _submit(connect: false),
                child: Text(l10n.testAiConnection),
              ),
              FilledButton(
                onPressed: _busy || !widget.enabled
                    ? null
                    : () => _submit(connect: true),
                child: Text(l10n.syncConnect),
              ),
            ],
          ),
        ],
      ),
    );
  }

  String? _required(String? value) => value == null || value.trim().isEmpty
      ? context.l10n.syncFieldRequired
      : null;

  Future<void> _submit({required bool connect}) async {
    if (!_formKey.currentState!.validate()) return;
    setState(() => _busy = true);
    widget.onBusyChanged(true);
    final repository = ref.read(syncRepositoryProvider);
    try {
      if (connect) {
        await repository.connect(
          provider: _provider,
          serverUrl: _server.text,
          username: _username.text,
          credential: _credential.text,
        );
        _credential.clear();
        widget.onChanged();
      } else {
        await repository.testConnection(
          provider: _provider,
          serverUrl: _server.text,
          username: _username.text,
          credential: _credential.text,
        );
      }
      if (mounted) {
        _show(connect
            ? context.l10n.syncConnected
            : context.l10n.syncTestSucceeded);
      }
    } catch (error) {
      if (connect) {
        widget.onChanged();
      }
      if (mounted) {
        _show(context.l10n.localizeError(error));
      }
    } finally {
      widget.onBusyChanged(false);
      if (mounted) setState(() => _busy = false);
    }
  }

  void _show(String message) => ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(message)),
      );
}
