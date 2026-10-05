import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
import '../../repositories/article_repository.dart';
import '../../repositories/feed_repository.dart';

final rulesProvider = FutureProvider<List<bridge.Rule>>((ref) {
  return ref.watch(articleRepositoryProvider).listRules();
});

class TagManagerScreen extends ConsumerWidget {
  const TagManagerScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final tags = ref.watch(articleTagsProvider);
    return Scaffold(
      appBar: AppBar(title: Text(context.l10n.manageTags)),
      floatingActionButton: FloatingActionButton(
        tooltip: context.l10n.createTag,
        onPressed: () => _editTag(context, ref),
        child: const Icon(Icons.add),
      ),
      body: tags.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, _) =>
            Center(child: Text(context.l10n.localizeError(error))),
        data: (items) => ReorderableListView.builder(
          padding: const EdgeInsets.only(bottom: 88),
          itemCount: items.length,
          onReorderItem: (oldIndex, newIndex) async {
            final reordered = [...items];
            final tag = reordered.removeAt(oldIndex);
            reordered.insert(newIndex, tag);
            try {
              await ref
                  .read(articleRepositoryProvider)
                  .reorderTags(reordered.map((tag) => tag.id.toInt()).toList());
              ref.invalidate(articleTagsProvider);
            } catch (error) {
              if (context.mounted) _showError(context, error);
            }
          },
          itemBuilder: (context, index) {
            final tag = items[index];
            return ListTile(
              key: ValueKey(tag.id),
              leading: CircleAvatar(
                  backgroundColor: _tagColor(tag.color), radius: 10),
              title: Text(tag.name),
              subtitle: Text('${tag.articleCount}'),
              trailing: const Icon(Icons.drag_handle),
              onTap: () => _editTag(context, ref, tag: tag),
            );
          },
        ),
      ),
    );
  }
}

Future<void> _editTag(
  BuildContext context,
  WidgetRef ref, {
  bridge.TagSummary? tag,
}) async {
  final controller = TextEditingController(text: tag?.name);
  var color = tag?.color ?? 'clay';
  final result = await showDialog<_TagDraft>(
    context: context,
    builder: (context) => StatefulBuilder(
      builder: (context, setState) => AlertDialog(
        title: Text(tag == null ? context.l10n.createTag : context.l10n.rename),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: controller,
              autofocus: true,
              decoration: InputDecoration(labelText: context.l10n.tagName),
            ),
            DropdownButtonFormField<String>(
              initialValue: color,
              items: _tagColors
                  .map((value) =>
                      DropdownMenuItem(value: value, child: Text(value)))
                  .toList(),
              onChanged: (value) => setState(() => color = value!),
            ),
          ],
        ),
        actions: [
          if (tag != null)
            TextButton(
              onPressed: () =>
                  Navigator.pop(context, const _TagDraft(delete: true)),
              child: Text(context.l10n.delete),
            ),
          TextButton(
              onPressed: () => Navigator.pop(context),
              child: Text(context.l10n.cancel)),
          FilledButton(
            onPressed: () => Navigator.pop(
                context, _TagDraft(name: controller.text, color: color)),
            child: Text(context.l10n.save),
          ),
        ],
      ),
    ),
  );
  controller.dispose();
  if (result == null) return;
  try {
    final repository = ref.read(articleRepositoryProvider);
    if (result.delete) {
      await repository.deleteTag(tag!.id.toInt());
    } else if (tag == null) {
      final id = await repository.createTag(result.name);
      await repository.setTagColor(id, result.color);
    } else {
      await repository.renameTag(tag.id.toInt(), result.name);
      await repository.setTagColor(tag.id.toInt(), result.color);
    }
    ref.invalidate(articleTagsProvider);
  } catch (error) {
    if (context.mounted) _showError(context, error);
  }
}

class RuleManagerScreen extends ConsumerStatefulWidget {
  const RuleManagerScreen({super.key});

  @override
  ConsumerState<RuleManagerScreen> createState() =>
      _RuleManagerScreenState();
}

class _RuleManagerScreenState extends ConsumerState<RuleManagerScreen> {
  final Map<int, bool> _enabledOverrides = {};

  @override
  Widget build(BuildContext context) {
    final rules = ref.watch(rulesProvider);
    return Scaffold(
      appBar: AppBar(title: Text(context.l10n.manageRules)),
      floatingActionButton: FloatingActionButton(
        tooltip: context.l10n.newRule,
        onPressed: () => _editRule(context, ref),
        child: const Icon(Icons.add),
      ),
      body: rules.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, _) =>
            Center(child: Text(context.l10n.localizeError(error))),
        data: (items) => ListView.builder(
          padding: const EdgeInsets.only(bottom: 88),
          itemCount: items.length,
          itemBuilder: (context, index) {
            final rule = items[index];
            final ruleId = rule.id.toInt();
            final enabled = _enabledOverrides[ruleId] ?? rule.enabled;
            return ListTile(
              title: Text(rule.name),
              subtitle: Text('${rule.field}: ${rule.query} → ${rule.action}'),
              trailing: Switch(
                value: enabled,
                onChanged: _enabledOverrides.containsKey(ruleId)
                    ? null
                    : (value) => _setRuleEnabled(rule, value),
              ),
              onTap: () => _editRule(context, ref, rule: rule),
            );
          },
        ),
      ),
    );
  }

  Future<void> _setRuleEnabled(bridge.Rule rule, bool enabled) async {
    final ruleId = rule.id.toInt();
    setState(() => _enabledOverrides[ruleId] = enabled);
    try {
      await ref
          .read(articleRepositoryProvider)
          .updateRule(ruleId, _ruleInput(rule, enabled: enabled));
      ref.invalidate(rulesProvider);
      await ref.read(rulesProvider.future);
    } catch (error) {
      if (mounted) _showError(context, error);
    } finally {
      if (mounted) setState(() => _enabledOverrides.remove(ruleId));
    }
  }
}

Future<void> _editRule(
  BuildContext context,
  WidgetRef ref, {
  bridge.Rule? rule,
}) async {
  List<bridge.Feed> feeds = const [];
  try {
    feeds = await ref.read(feedRepositoryProvider).listFeeds();
  } catch (_) {
    // A global rule remains useful while subscriptions are temporarily unavailable.
  }
  if (!context.mounted) return;
  final draft = await showModalBottomSheet<_RuleDraft>(
    context: context,
    isScrollControlled: true,
    builder: (_) => _RuleEditorSheet(
      repository: ref.read(articleRepositoryProvider),
      rule: rule,
      feeds: feeds,
      onArticleStateChanged: () {
        ref.invalidate(articlePageProvider);
        ref.invalidate(articleCountsProvider);
        ref.invalidate(articleCountProvider);
      },
    ),
  );
  if (draft == null) return;
  try {
    final repository = ref.read(articleRepositoryProvider);
    if (draft.delete) {
      await repository.deleteRule(rule!.id.toInt());
    } else if (rule == null) {
      await repository.createRule(draft.input!);
    } else {
      await repository.updateRule(rule.id.toInt(), draft.input!);
    }
    ref.invalidate(rulesProvider);
  } catch (error) {
    if (context.mounted) _showError(context, error);
  }
}

class _RuleEditorSheet extends StatefulWidget {
  final ArticleRepository repository;
  final bridge.Rule? rule;
  final List<bridge.Feed> feeds;
  final VoidCallback onArticleStateChanged;
  const _RuleEditorSheet({
    required this.repository,
    required this.feeds,
    required this.onArticleStateChanged,
    this.rule,
  });

  @override
  State<_RuleEditorSheet> createState() => _RuleEditorSheetState();
}

class _RuleEditorSheetState extends State<_RuleEditorSheet> {
  late final TextEditingController _name =
      TextEditingController(text: widget.rule?.name);
  late final TextEditingController _query =
      TextEditingController(text: widget.rule?.query);
  late String _field = widget.rule?.field ?? 'title';
  late String _action = widget.rule?.action ?? 'skip';
  late bool _enabled = widget.rule?.enabled ?? true;
  late int? _feedId = widget.rule?.feedId?.toInt();
  bridge.RulePreview? _preview;

  bridge.RuleInput get _input => bridge.RuleInput(
        name: _name.text,
        enabled: _enabled,
        feedId: _feedId,
        field: _field,
        query: _query.text,
        action: _action,
      );

  @override
  void dispose() {
    _name.dispose();
    _query.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => SafeArea(
        child: Padding(
          padding: EdgeInsets.fromLTRB(
              20, 20, 20, 20 + MediaQuery.viewInsetsOf(context).bottom),
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                    controller: _name,
                    decoration:
                        InputDecoration(labelText: context.l10n.ruleName)),
                TextField(
                    controller: _query,
                    decoration:
                        InputDecoration(labelText: context.l10n.ruleKeywords)),
                DropdownButtonFormField<String>(
                  initialValue: _field,
                  decoration:
                      InputDecoration(labelText: context.l10n.ruleField),
                  items: [
                    ('title', context.l10n.titleField),
                    ('author', context.l10n.authorField),
                    ('content', context.l10n.contentField),
                    ('any', context.l10n.anyField),
                  ]
                      .map((item) => DropdownMenuItem(
                          value: item.$1, child: Text(item.$2)))
                      .toList(),
                  onChanged: (value) => setState(() => _field = value!),
                ),
                DropdownButtonFormField<int?>(
                  initialValue: _feedId,
                  decoration: InputDecoration(
                      labelText: context.l10n.subscriptionsTitle),
                  items: [
                    DropdownMenuItem<int?>(
                        value: null, child: Text(context.l10n.allFeeds)),
                    for (final feed in widget.feeds)
                      DropdownMenuItem<int?>(
                        value: feed.id.toInt(),
                        child: Text(feed.title),
                      ),
                  ],
                  onChanged: (value) => setState(() => _feedId = value),
                ),
                DropdownButtonFormField<String>(
                  initialValue: _action,
                  decoration:
                      InputDecoration(labelText: context.l10n.ruleAction),
                  items: [
                    ('skip', context.l10n.skipAction),
                    ('read', context.l10n.readAction),
                    ('star', context.l10n.starAction),
                  ]
                      .map((item) => DropdownMenuItem(
                          value: item.$1, child: Text(item.$2)))
                      .toList(),
                  onChanged: (value) => setState(() => _action = value!),
                ),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(context.l10n.ruleEnabled),
                  value: _enabled,
                  onChanged: (value) => setState(() => _enabled = value),
                ),
                if (_preview != null)
                  Text(context.l10n.rulePreviewResult(_preview!.count.toInt())),
                if (_preview?.samples.isNotEmpty == true)
                  for (final sample in _preview!.samples)
                    Align(alignment: Alignment.centerLeft, child: Text(sample)),
                const SizedBox(height: 12),
                Wrap(
                  alignment: WrapAlignment.end,
                  spacing: 8,
                  runSpacing: 8,
                  children: [
                    OutlinedButton(
                      onPressed: _previewRule,
                      child: Text(context.l10n.rulePreview),
                    ),
                    OutlinedButton(
                      onPressed: _applyRule,
                      child: Text(context.l10n.applyRule),
                    ),
                    if (widget.rule != null)
                      TextButton(
                        onPressed: () => Navigator.pop(
                            context, const _RuleDraft(delete: true)),
                        child: Text(context.l10n.delete),
                      ),
                    FilledButton(
                      onPressed: () =>
                          Navigator.pop(context, _RuleDraft(input: _input)),
                      child: Text(context.l10n.save),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      );

  Future<void> _previewRule() async {
    try {
      final preview = await widget.repository.previewRule(_input);
      if (mounted) setState(() => _preview = preview);
    } catch (error) {
      if (mounted) _showError(context, error);
    }
  }

  Future<void> _applyRule() async {
    if (_action == 'skip') {
      final accepted = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text(context.l10n.applySkipRuleTitle),
          content: Text(context.l10n.applySkipRuleMessage),
          actions: [
            TextButton(
                onPressed: () => Navigator.pop(context, false),
                child: Text(context.l10n.cancel)),
            FilledButton(
                onPressed: () => Navigator.pop(context, true),
                child: Text(context.l10n.applyRule)),
          ],
        ),
      );
      if (accepted != true) return;
    }
    try {
      final count = await widget.repository.applyRuleToExisting(_input);
      if (mounted) {
        widget.onArticleStateChanged();
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.applyRuleComplete(count))),
        );
        await _previewRule();
      }
    } catch (error) {
      if (mounted) _showError(context, error);
    }
  }
}

bridge.RuleInput _ruleInput(bridge.Rule rule, {bool? enabled}) =>
    bridge.RuleInput(
      name: rule.name,
      enabled: enabled ?? rule.enabled,
      feedId: rule.feedId,
      field: rule.field,
      query: rule.query,
      action: rule.action,
    );

class _TagDraft {
  final String name;
  final String color;
  final bool delete;
  const _TagDraft({this.name = '', this.color = 'clay', this.delete = false});
}

class _RuleDraft {
  final bridge.RuleInput? input;
  final bool delete;
  const _RuleDraft({this.input, this.delete = false});
}

const _tagColors = [
  'clay',
  'amber',
  'pine',
  'teal',
  'indigo',
  'violet',
  'rose',
  'slate'
];
Color _tagColor(String color) => switch (color) {
      'amber' => Colors.amber,
      'pine' => Colors.green,
      'teal' => Colors.teal,
      'indigo' => Colors.indigo,
      'violet' => Colors.deepPurple,
      'rose' => Colors.pink,
      'slate' => Colors.blueGrey,
      _ => Colors.brown,
    };

void _showError(BuildContext context, Object error) =>
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(context.l10n.localizeError(error))),
    );
