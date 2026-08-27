import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../core/exceptions.dart';
import '../../l10n/l10n.dart';
import '../../repositories/ai_repository.dart';
import '../../repositories/settings_repository.dart';
import 'ai_profiles_screen.dart';

class AiSummaryScreen extends ConsumerStatefulWidget {
  final int articleId;

  const AiSummaryScreen({super.key, required this.articleId});

  @override
  ConsumerState<AiSummaryScreen> createState() => _AiSummaryScreenState();
}

class _AiSummaryScreenState extends ConsumerState<AiSummaryScreen> {
  final _question = TextEditingController();
  StreamSubscription<bridge.AiStreamEvent>? _subscription;
  bridge.AiProfile? _profile;
  bridge.AiSummaryCache? _cache;
  bridge.SummaryTemplate _template = bridge.SummaryTemplate.classic;
  List<bridge.AiFollowUpTurn> _history = const [];
  String _streamed = '';
  String _followUpDraft = '';
  String? _activeRequestId;
  Object? _error;
  bool _loading = true;
  bool _generating = false;
  bool _asking = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    final requestId = _activeRequestId;
    if (requestId != null) {
      unawaited(_cancelWhenDisposing(requestId));
    }
    unawaited(_subscription?.cancel());
    _question.dispose();
    super.dispose();
  }

  Future<void> _cancelWhenDisposing(String requestId) async {
    try {
      await ref.read(aiRepositoryProvider).cancel(requestId);
    } catch (_) {
      // The screen is already closing, so cancellation remains best-effort.
    }
  }

  Future<void> _load() async {
    if (mounted) {
      setState(() {
        _loading = true;
        _error = null;
      });
    }
    try {
      final repository = ref.read(aiRepositoryProvider);
      final results = await Future.wait<Object?>([
        repository.listProfiles(),
        repository.getSummaryCache(widget.articleId),
      ]);
      if (!mounted) return;
      final profiles = results[0]! as List<bridge.AiProfile>;
      final cache = results[1] as bridge.AiSummaryCache?;
      setState(() {
        _profile = repository.summaryProfile(profiles);
        _cache = cache;
        _template = _templateFromCache(cache?.template);
        _loading = false;
      });
      if (cache == null && _profile != null) {
        await _generate();
      }
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error;
          _loading = false;
        });
      }
    }
  }

  Future<void> _generate() async {
    final profile = _profile;
    if (profile == null || _generating || _asking) return;
    await _cancelActive();
    final requestId = _newRequestId('summary');
    setState(() {
      _history = const [];
      _followUpDraft = '';
      _streamed = '';
      _error = null;
      _generating = true;
      _activeRequestId = requestId;
    });
    final language =
        ref.read(appearanceProvider).asData?.value.language ?? 'en';
    _subscription = ref
        .read(aiRepositoryProvider)
        .summarize(
          articleId: widget.articleId,
          profile: profile,
          template: _template,
          language: language,
          requestId: requestId,
        )
        .listen(
          _handleSummaryEvent,
          onError: (Object error) => _finishWithError(requestId, error),
        );
  }

  void _handleSummaryEvent(bridge.AiStreamEvent event) {
    event.when<void>(
      delta: (requestId, text) {
        if (!_isActive(requestId)) return;
        setState(() => _streamed += text);
      },
      progress: (_, __, ___) {},
      completed: (requestId) {
        if (!_isActive(requestId)) return;
        final language =
            ref.read(appearanceProvider).asData?.value.language ?? 'en';
        setState(() {
          _cache = bridge.AiSummaryCache(
            summary: _streamed,
            template: _templateName(_template),
            language: _responseLanguage(language),
          );
          _streamed = '';
          _generating = false;
          _activeRequestId = null;
        });
      },
      error: (requestId, code) {
        if (!_isActive(requestId)) return;
        _finishWithError(
          requestId,
          AppException(AppErrorKind.ai, code, null),
        );
      },
    );
  }

  Future<void> _ask() async {
    final profile = _profile;
    final summary = _cache?.summary.trim();
    final question = _question.text.trim();
    if (profile == null ||
        summary == null ||
        summary.isEmpty ||
        question.isEmpty ||
        _generating ||
        _asking) {
      return;
    }
    await _cancelActive();
    final requestId = _newRequestId('follow');
    setState(() {
      _asking = true;
      _followUpDraft = '';
      _error = null;
      _activeRequestId = requestId;
    });
    final language =
        ref.read(appearanceProvider).asData?.value.language ?? 'en';
    final previousHistory = List<bridge.AiFollowUpTurn>.of(_history);
    _subscription = ref
        .read(aiRepositoryProvider)
        .followUp(
          profile: profile,
          summary: summary,
          history: previousHistory,
          question: question,
          language: language,
          requestId: requestId,
        )
        .listen(
          (event) => event.when<void>(
            delta: (eventRequestId, text) {
              if (!_isActive(eventRequestId)) return;
              setState(() => _followUpDraft += text);
            },
            progress: (_, __, ___) {},
            completed: (eventRequestId) {
              if (!_isActive(eventRequestId)) return;
              setState(() {
                _history = [
                  ...previousHistory,
                  bridge.AiFollowUpTurn(
                    question: question,
                    answer: _followUpDraft,
                  ),
                ];
                _question.clear();
                _followUpDraft = '';
                _asking = false;
                _activeRequestId = null;
              });
            },
            error: (eventRequestId, code) {
              if (!_isActive(eventRequestId)) return;
              _finishWithError(
                eventRequestId,
                AppException(AppErrorKind.ai, code, null),
              );
            },
          ),
          onError: (Object error) => _finishWithError(requestId, error),
        );
  }

  Future<void> _cancelActive() async {
    final requestId = _activeRequestId;
    _activeRequestId = null;
    if (requestId != null) {
      try {
        await ref.read(aiRepositoryProvider).cancel(requestId);
      } catch (_) {
        // Cancellation is best-effort during navigation and request replacement.
      }
    }
    await _subscription?.cancel();
    _subscription = null;
    if (mounted) {
      setState(() {
        _generating = false;
        _asking = false;
        _streamed = '';
        _followUpDraft = '';
      });
    }
  }

  void _finishWithError(String requestId, Object error) {
    if (!_isActive(requestId) || !mounted) return;
    setState(() {
      _error = error;
      _generating = false;
      _asking = false;
      _streamed = '';
      _followUpDraft = '';
      _activeRequestId = null;
    });
  }

  bool _isActive(String requestId) => mounted && requestId == _activeRequestId;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(context.l10n.aiSummary)),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    if (_loading) return const Center(child: CircularProgressIndicator());
    if (_profile == null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.auto_awesome_outlined, size: 42),
              const SizedBox(height: 12),
              Text(context.l10n.summaryNoProfile),
              const SizedBox(height: 16),
              FilledButton(
                onPressed: () async {
                  await Navigator.of(context).push(
                    MaterialPageRoute<void>(
                      builder: (_) => const AiProfilesScreen(),
                    ),
                  );
                  if (mounted) await _load();
                },
                child: Text(context.l10n.configureAiProfile),
              ),
            ],
          ),
        ),
      );
    }

    final visibleSummary = _generating ? _streamed : _cache?.summary ?? '';
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Row(
          children: [
            Expanded(
              child: DropdownButtonFormField<bridge.SummaryTemplate>(
                initialValue: _template,
                decoration:
                    InputDecoration(labelText: context.l10n.summaryTemplate),
                items: [
                  for (final template in bridge.SummaryTemplate.values)
                    DropdownMenuItem(
                      value: template,
                      child: Text(_templateLabel(template)),
                    ),
                ],
                onChanged: _generating || _asking
                    ? null
                    : (value) => setState(() => _template = value!),
              ),
            ),
            const SizedBox(width: 12),
            if (_generating)
              OutlinedButton.icon(
                onPressed: _cancelActive,
                icon: const Icon(Icons.stop),
                label: Text(context.l10n.stop),
              )
            else
              FilledButton.icon(
                onPressed: _generate,
                icon: const Icon(Icons.refresh),
                label: Text(
                  _cache == null ? context.l10n.retry : context.l10n.regenerate,
                ),
              ),
          ],
        ),
        if (_cache != null && !_generating)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(
              context.l10n.summaryCacheInfo(
                _cache!.template ?? context.l10n.summaryTemplateLegacy,
                _cache!.language ?? '-',
              ),
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ),
        if (_generating) ...[
          const SizedBox(height: 12),
          const LinearProgressIndicator(),
        ],
        const SizedBox(height: 20),
        if (visibleSummary.isEmpty && !_generating)
          Text(context.l10n.noSummaryYet)
        else
          SelectableText(
            visibleSummary,
            style: Theme.of(context).textTheme.bodyLarge?.copyWith(height: 1.6),
          ),
        if (_error != null) ...[
          const SizedBox(height: 12),
          Text(
            context.l10n.localizeError(_error!),
            style: TextStyle(color: Theme.of(context).colorScheme.error),
          ),
        ],
        if (_cache != null && !_generating) ...[
          const Divider(height: 36),
          Text(
            context.l10n.askAboutSummary,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          for (final turn in _history) ...[
            const SizedBox(height: 12),
            Text(turn.question, style: Theme.of(context).textTheme.titleSmall),
            const SizedBox(height: 4),
            SelectableText(turn.answer),
          ],
          if (_asking && _followUpDraft.isNotEmpty) ...[
            const SizedBox(height: 12),
            SelectableText(_followUpDraft),
          ],
          const SizedBox(height: 16),
          Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Expanded(
                child: TextField(
                  controller: _question,
                  minLines: 1,
                  maxLines: 4,
                  enabled: !_asking,
                  decoration: InputDecoration(
                      hintText: context.l10n.summaryQuestionHint),
                  onSubmitted: (_) => _ask(),
                ),
              ),
              IconButton.filled(
                tooltip: context.l10n.send,
                onPressed: _asking ? null : _ask,
                icon: _asking
                    ? const SizedBox.square(
                        dimension: 20,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.send),
              ),
            ],
          ),
        ],
      ],
    );
  }

  String _templateLabel(bridge.SummaryTemplate template) => switch (template) {
        bridge.SummaryTemplate.classic => context.l10n.summaryTemplateClassic,
        bridge.SummaryTemplate.news5W1H => context.l10n.summaryTemplateNews,
        bridge.SummaryTemplate.decision => context.l10n.summaryTemplateDecision,
        bridge.SummaryTemplate.funnel => context.l10n.summaryTemplateFunnel,
        bridge.SummaryTemplate.argument => context.l10n.summaryTemplateArgument,
        bridge.SummaryTemplate.minimal => context.l10n.summaryTemplateMinimal,
      };
}

bridge.SummaryTemplate _templateFromCache(String? value) => switch (value) {
      'news5w1h' => bridge.SummaryTemplate.news5W1H,
      'decision' => bridge.SummaryTemplate.decision,
      'funnel' => bridge.SummaryTemplate.funnel,
      'argument' => bridge.SummaryTemplate.argument,
      'minimal' => bridge.SummaryTemplate.minimal,
      _ => bridge.SummaryTemplate.classic,
    };

String _templateName(bridge.SummaryTemplate template) => switch (template) {
      bridge.SummaryTemplate.classic => 'classic',
      bridge.SummaryTemplate.news5W1H => 'news5w1h',
      bridge.SummaryTemplate.decision => 'decision',
      bridge.SummaryTemplate.funnel => 'funnel',
      bridge.SummaryTemplate.argument => 'argument',
      bridge.SummaryTemplate.minimal => 'minimal',
    };

String _responseLanguage(String language) {
  final code = language.split(RegExp('[-_]')).first.toLowerCase();
  return code == 'zh' || code == 'ja' ? code : 'en';
}

String _newRequestId(String prefix) =>
    '$prefix-${DateTime.now().microsecondsSinceEpoch.toRadixString(36)}';
