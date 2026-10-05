import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_widget_from_html_core/flutter_widget_from_html_core.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../core/exceptions.dart';
import '../../l10n/l10n.dart';
import '../../repositories/ai_repository.dart';
import '../../repositories/article_repository.dart';
import '../../repositories/settings_repository.dart';
import '../../services/platform_service.dart';
import 'ai_profiles_screen.dart';

class AiTranslationScreen extends ConsumerStatefulWidget {
  final int articleId;

  const AiTranslationScreen({super.key, required this.articleId});

  @override
  ConsumerState<AiTranslationScreen> createState() =>
      _AiTranslationScreenState();
}

class _AiTranslationScreenState extends ConsumerState<AiTranslationScreen> {
  StreamSubscription<bridge.AiStreamEvent>? _subscription;
  bridge.AiProfile? _profile;
  bridge.ArticleDetail? _detail;
  String _language = 'en';
  String? _cachedHtml;
  String? _cachedLanguage;
  String? _activeRequestId;
  Object? _error;
  int _completedBatches = 0;
  int _totalBatches = 0;
  bool _loading = true;
  bool _translating = false;

  @override
  void initState() {
    super.initState();
    _language = _responseLanguage(
      ref.read(appearanceProvider).asData?.value.language ?? 'en',
    );
    _load();
  }

  @override
  void dispose() {
    final requestId = _activeRequestId;
    if (requestId != null) {
      unawaited(_cancelWhenDisposing(requestId));
    }
    unawaited(_subscription?.cancel());
    super.dispose();
  }

  Future<void> _cancelWhenDisposing(String requestId) async {
    try {
      await ref.read(aiRepositoryProvider).cancel(requestId);
    } catch (_) {
      // The route is already closing, so cancellation remains best-effort.
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
      final results = await Future.wait<Object?>([
        ref.read(aiRepositoryProvider).listProfiles(),
        ref.read(articleRepositoryProvider).getArticleDetail(widget.articleId),
      ]);
      if (!mounted) return;
      final profiles = results[0]! as List<bridge.AiProfile>;
      final detail = results[1]! as bridge.ArticleDetail;
      setState(() {
        _profile = ref.read(aiRepositoryProvider).activeProfile(profiles);
        _detail = detail;
        _cachedHtml = detail.translatedHtml;
        _cachedLanguage = detail.translatedLang;
        _language = detail.translatedLang ?? _language;
        _loading = false;
      });
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error;
          _loading = false;
        });
      }
    }
  }

  Future<void> _translate() async {
    final profile = _profile;
    if (profile == null || _translating) return;
    await _cancelActive();
    final requestId = _newRequestId('translate');
    setState(() {
      _error = null;
      _completedBatches = 0;
      _totalBatches = 0;
      _translating = true;
      _activeRequestId = requestId;
    });
    _subscription = ref
        .read(aiRepositoryProvider)
        .translate(
          articleId: widget.articleId,
          profile: profile,
          language: _language,
          requestId: requestId,
        )
        .listen(
          _handleEvent,
          onError: (Object error) => _finishWithError(requestId, error),
        );
  }

  void _handleEvent(bridge.AiStreamEvent event) {
    event.when<void>(
      delta: (_, __) {},
      progress: (requestId, completed, total) {
        if (!_isActive(requestId)) return;
        setState(() {
          _completedBatches = completed.toInt();
          _totalBatches = total.toInt();
        });
      },
      completed: (requestId) {
        if (!_isActive(requestId)) return;
        _reloadCompletedTranslation(requestId);
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

  Future<void> _reloadCompletedTranslation(String requestId) async {
    try {
      final detail = await ref
          .read(articleRepositoryProvider)
          .getArticleDetail(widget.articleId);
      if (!_isActive(requestId) || !mounted) return;
      setState(() {
        _detail = detail;
        _cachedHtml = detail.translatedHtml;
        _cachedLanguage = detail.translatedLang;
        _translating = false;
        _activeRequestId = null;
      });
    } catch (error) {
      _finishWithError(requestId, error);
    }
  }

  Future<void> _cancelActive() async {
    final requestId = _activeRequestId;
    _activeRequestId = null;
    if (requestId != null) {
      try {
        await ref.read(aiRepositoryProvider).cancel(requestId);
      } catch (_) {
        // Cancellation is best-effort during navigation and replacement.
      }
    }
    await _subscription?.cancel();
    _subscription = null;
    if (mounted) {
      setState(() => _translating = false);
    }
  }

  void _finishWithError(String requestId, Object error) {
    if (!_isActive(requestId) || !mounted) return;
    setState(() {
      _error = error;
      _translating = false;
      _activeRequestId = null;
    });
  }

  bool _isActive(String requestId) => mounted && requestId == _activeRequestId;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(context.l10n.aiTranslation)),
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
              const Icon(Icons.translate, size: 42),
              const SizedBox(height: 12),
              Text(context.l10n.translationNoProfile),
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

    final detail = _detail;
    final progress =
        _totalBatches == 0 ? null : _completedBatches / _totalBatches;
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Row(
          children: [
            Expanded(
              child: DropdownButtonFormField<String>(
                initialValue: _language,
                decoration:
                    InputDecoration(labelText: context.l10n.targetLanguage),
                items: [
                  DropdownMenuItem(
                    value: 'zh',
                    child: Text(context.l10n.languageChinese),
                  ),
                  DropdownMenuItem(
                    value: 'en',
                    child: Text(context.l10n.languageEnglish),
                  ),
                  DropdownMenuItem(
                    value: 'ja',
                    child: Text(context.l10n.languageJapanese),
                  ),
                ],
                onChanged: _translating
                    ? null
                    : (value) => setState(() => _language = value!),
              ),
            ),
            const SizedBox(width: 12),
            if (_translating)
              OutlinedButton.icon(
                onPressed: _cancelActive,
                icon: const Icon(Icons.stop),
                label: Text(context.l10n.stop),
              )
            else
              FilledButton.icon(
                onPressed: _translate,
                icon: const Icon(Icons.translate),
                label: Text(
                  _cachedHtml == null
                      ? context.l10n.translateArticle
                      : context.l10n.regenerate,
                ),
              ),
          ],
        ),
        if (_cachedHtml != null && !_translating)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(
              context.l10n.translationCacheInfo(_cachedLanguage ?? '-'),
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ),
        if (_translating) ...[
          const SizedBox(height: 12),
          LinearProgressIndicator(value: progress),
          const SizedBox(height: 8),
          Text(
            _totalBatches == 0
                ? context.l10n.translationPreparing
                : context.l10n.translationProgress(
                    _completedBatches,
                    _totalBatches,
                  ),
            style: Theme.of(context).textTheme.bodySmall,
          ),
        ],
        if (_error != null) ...[
          const SizedBox(height: 12),
          Text(
            context.l10n.localizeError(_error!),
            style: TextStyle(color: Theme.of(context).colorScheme.error),
          ),
        ],
        const SizedBox(height: 20),
        if (_cachedHtml == null && !_translating)
          Text(context.l10n.noTranslationYet)
        else if (_cachedHtml != null)
          HtmlWidget(
            _cachedHtml!,
            baseUrl: detail?.url == null ? null : Uri.tryParse(detail!.url!),
            onTapUrl: platformService.openUrl,
            textStyle: Theme.of(context).textTheme.bodyLarge?.copyWith(
                  height: 1.6,
                ),
          ),
      ],
    );
  }
}

String _responseLanguage(String language) {
  final code = language.split(RegExp('[-_]')).first.toLowerCase();
  return code == 'zh' || code == 'ja' ? code : 'en';
}

String _newRequestId(String prefix) =>
    '$prefix-${DateTime.now().microsecondsSinceEpoch.toRadixString(36)}';
