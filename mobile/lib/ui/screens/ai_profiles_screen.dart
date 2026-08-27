import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
import '../../repositories/ai_repository.dart';

class AiProfilesScreen extends ConsumerWidget {
  const AiProfilesScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final profiles = ref.watch(aiProfilesProvider);
    final probingId = ref.watch(aiProfileConnectionProbeProvider);
    final togglingId = ref.watch(aiProfileEnabledToggleProvider);
    return Scaffold(
      appBar: AppBar(title: Text(context.l10n.aiProfiles)),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _edit(context, ref, null),
        tooltip: context.l10n.addAiProfile,
        child: const Icon(Icons.add),
      ),
      floatingActionButtonLocation: FloatingActionButtonLocation.endFloat,
      body: profiles.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, _) => Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(context.l10n.localizeError(error)),
                const SizedBox(height: 12),
                FilledButton(
                  onPressed: () => ref.invalidate(aiProfilesProvider),
                  child: Text(context.l10n.retry),
                ),
              ],
            ),
          ),
        ),
        data: (items) => items.isEmpty
            ? Center(child: Text(context.l10n.noAiProfiles))
            : ListView.separated(
                itemCount: items.length,
                separatorBuilder: (_, __) => const Divider(height: 1),
                itemBuilder: (context, index) {
                  final profile = items[index];
                  return ListTile(
                    leading: Icon(
                      profile.enabled
                          ? Icons.auto_awesome
                          : Icons.auto_awesome_outlined,
                    ),
                    title: Text(profile.name),
                    subtitle: Text(
                      '${profile.model}\n${profile.baseUrl}',
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                    isThreeLine: true,
                    trailing: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Switch(
                          value: profile.enabled,
                          onChanged: togglingId == null
                              ? (enabled) =>
                                  _setEnabled(context, ref, profile, enabled)
                              : null,
                        ),
                        IconButton(
                          tooltip: context.l10n.testAiConnection,
                          onPressed: probingId == null
                              ? () => _testConnection(context, ref, profile)
                              : null,
                          icon: probingId == profile.id
                              ? const SizedBox.square(
                                  dimension: 18,
                                  child:
                                      CircularProgressIndicator(strokeWidth: 2),
                                )
                              : const Icon(Icons.network_check_outlined),
                        ),
                        IconButton(
                          onPressed: () => _delete(context, ref, profile),
                          icon: const Icon(Icons.delete_outline),
                        ),
                      ],
                    ),
                    onTap: () => _edit(context, ref, profile),
                  );
                },
              ),
      ),
    );
  }

  Future<void> _setEnabled(
    BuildContext context,
    WidgetRef ref,
    bridge.AiProfile profile,
    bool enabled,
  ) async {
    ref.read(aiProfileEnabledToggleProvider.notifier).state = profile.id;
    try {
      await ref
          .read(aiRepositoryProvider)
          .setProfileEnabled(profile.id, enabled);
      ref.invalidate(aiProfilesProvider);
    } catch (error) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.localizeError(error))),
        );
      }
    } finally {
      if (ref.read(aiProfileEnabledToggleProvider) == profile.id) {
        ref.read(aiProfileEnabledToggleProvider.notifier).state = null;
      }
    }
  }

  Future<void> _testConnection(
    BuildContext context,
    WidgetRef ref,
    bridge.AiProfile profile,
  ) async {
    ref.read(aiProfileConnectionProbeProvider.notifier).state = profile.id;
    try {
      await ref.read(aiRepositoryProvider).testConnection(profile);
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.aiConnectionSucceeded)),
        );
      }
    } catch (error) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.localizeError(error))),
        );
      }
    } finally {
      if (ref.read(aiProfileConnectionProbeProvider) == profile.id) {
        ref.read(aiProfileConnectionProbeProvider.notifier).state = null;
      }
    }
  }

  Future<void> _edit(
    BuildContext context,
    WidgetRef ref,
    bridge.AiProfile? profile,
  ) async {
    final draft = await showDialog<_AiProfileDraft>(
      context: context,
      builder: (_) => _AiProfileDialog(profile: profile),
    );
    if (draft == null || !context.mounted) return;
    try {
      await ref.read(aiRepositoryProvider).saveProfile(
            draft.profile,
            newSecret: draft.secret,
          );
      ref.invalidate(aiProfilesProvider);
    } catch (error) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.localizeError(error))),
        );
      }
    }
  }

  Future<void> _delete(
    BuildContext context,
    WidgetRef ref,
    bridge.AiProfile profile,
  ) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(context.l10n.deleteAiProfileTitle),
        content: Text(profile.name),
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
    );
    if (confirmed != true || !context.mounted) return;
    try {
      await ref.read(aiRepositoryProvider).deleteProfile(profile.id);
      ref.invalidate(aiProfilesProvider);
    } catch (error) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.localizeError(error))),
        );
      }
    }
  }
}

class _AiProfileDialog extends StatefulWidget {
  final bridge.AiProfile? profile;

  const _AiProfileDialog({this.profile});

  @override
  State<_AiProfileDialog> createState() => _AiProfileDialogState();
}

class _AiProfileDialogState extends State<_AiProfileDialog> {
  final _formKey = GlobalKey<FormState>();
  late final TextEditingController _name;
  late final TextEditingController _model;
  late final TextEditingController _baseUrl;
  late final TextEditingController _secret;
  late bridge.AiProtocol _protocol;
  late bridge.AiAuthMode _auth;

  @override
  void initState() {
    super.initState();
    final profile = widget.profile;
    _name = TextEditingController(text: profile?.name ?? '');
    _model = TextEditingController(text: profile?.model ?? '');
    _baseUrl = TextEditingController(
      text: profile?.baseUrl ?? 'https://api.openai.com/v1',
    );
    _secret = TextEditingController();
    _protocol = profile?.protocol ?? bridge.AiProtocol.openaiChatCompletions;
    _auth = profile?.auth ?? bridge.AiAuthMode.bearer;
  }

  @override
  void dispose() {
    _name.dispose();
    _model.dispose();
    _baseUrl.dispose();
    _secret.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
        title: Text(
          widget.profile == null
              ? context.l10n.addAiProfile
              : context.l10n.editAiProfile,
        ),
        content: SizedBox(
          width: 480,
          child: Form(
            key: _formKey,
            child: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  TextFormField(
                    controller: _name,
                    decoration:
                        InputDecoration(labelText: context.l10n.profileName),
                    validator: _required,
                  ),
                  DropdownButtonFormField<bridge.AiProtocol>(
                    initialValue: _protocol,
                    decoration:
                        InputDecoration(labelText: context.l10n.aiProtocol),
                    items: [
                      DropdownMenuItem(
                        value: bridge.AiProtocol.openaiChatCompletions,
                        child: Text(context.l10n.openaiCompatible),
                      ),
                      DropdownMenuItem(
                        value: bridge.AiProtocol.anthropicMessages,
                        child: Text(context.l10n.anthropic),
                      ),
                    ],
                    onChanged: (value) => setState(() => _protocol = value!),
                  ),
                  TextFormField(
                    controller: _model,
                    decoration:
                        InputDecoration(labelText: context.l10n.aiModel),
                    validator: _required,
                  ),
                  TextFormField(
                    controller: _baseUrl,
                    keyboardType: TextInputType.url,
                    decoration:
                        InputDecoration(labelText: context.l10n.aiBaseUrl),
                    validator: (value) {
                      final uri = Uri.tryParse(value?.trim() ?? '');
                      return uri != null &&
                              (uri.scheme == 'http' || uri.scheme == 'https') &&
                              uri.host.isNotEmpty
                          ? null
                          : context.l10n.errorInvalidAiProfile;
                    },
                  ),
                  DropdownButtonFormField<bridge.AiAuthMode>(
                    initialValue: _auth,
                    decoration: InputDecoration(labelText: context.l10n.aiAuth),
                    items: [
                      DropdownMenuItem(
                        value: bridge.AiAuthMode.bearer,
                        child: Text(context.l10n.bearerAuth),
                      ),
                      DropdownMenuItem(
                        value: bridge.AiAuthMode.xApiKey,
                        child: Text(context.l10n.xApiKeyAuth),
                      ),
                      DropdownMenuItem(
                        value: bridge.AiAuthMode.none,
                        child: Text(context.l10n.noAuth),
                      ),
                    ],
                    onChanged: (value) => setState(() => _auth = value!),
                  ),
                  if (_auth != bridge.AiAuthMode.none)
                    TextFormField(
                      controller: _secret,
                      obscureText: true,
                      enableSuggestions: false,
                      autocorrect: false,
                      decoration: InputDecoration(
                        labelText: context.l10n.aiApiKey,
                        helperText: widget.profile == null
                            ? null
                            : context.l10n.aiApiKeyKeep,
                      ),
                      validator: (value) {
                        if (widget.profile == null &&
                            (value == null || value.trim().isEmpty)) {
                          return context.l10n.errorNoAiCredential;
                        }
                        return null;
                      },
                    ),
                ],
              ),
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(context.l10n.cancel),
          ),
          FilledButton(onPressed: _submit, child: Text(context.l10n.save)),
        ],
      );

  String? _required(String? value) => value?.trim().isNotEmpty == true
      ? null
      : context.l10n.errorInvalidAiProfile;

  void _submit() {
    if (!_formKey.currentState!.validate()) return;
    final existing = widget.profile;
    final id =
        existing?.id ?? DateTime.now().microsecondsSinceEpoch.toRadixString(36);
    final credentialRef = _auth == bridge.AiAuthMode.none
        ? null
        : existing?.credentialRef ?? 'papr.ai.$id';
    Navigator.pop(
      context,
      _AiProfileDraft(
        bridge.AiProfile(
          id: id,
          name: _name.text.trim(),
          protocol: _protocol,
          model: _model.text.trim(),
          baseUrl: _baseUrl.text.trim(),
          auth: _auth,
          headers: existing?.headers ?? const [],
          credentialRef: credentialRef,
          enabled: existing?.enabled ?? true,
          defaultFor: existing?.defaultFor ?? const [bridge.AiPurpose.summary],
        ),
        _secret.text.trim().isEmpty ? null : _secret.text,
      ),
    );
  }
}

class _AiProfileDraft {
  final bridge.AiProfile profile;
  final String? secret;

  const _AiProfileDraft(this.profile, this.secret);
}
