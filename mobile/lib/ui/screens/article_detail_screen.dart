import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_widget_from_html_core/flutter_widget_from_html_core.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
import '../../repositories/article_repository.dart';
import '../../repositories/settings_repository.dart';
import '../../services/platform_service.dart';

final articleDetailProvider =
    FutureProvider.family<bridge.ArticleDetail, int>((ref, articleId) {
  return ref.watch(articleRepositoryProvider).getArticleDetail(articleId);
});

class ArticleDetailScreen extends ConsumerStatefulWidget {
  final int articleId;

  const ArticleDetailScreen({super.key, required this.articleId});

  @override
  ConsumerState<ArticleDetailScreen> createState() =>
      _ArticleDetailScreenState();
}

class _ArticleDetailScreenState extends ConsumerState<ArticleDetailScreen> {
  bridge.ArticleDetail? _detail;
  Object? _error;
  bool _loading = true;
  bool _extracting = false;
  bool _autoExtractAttempted = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      var detail =
          await ref.read(articleDetailProvider(widget.articleId).future);
      if (!mounted) return;
      setState(() {
        _detail = detail;
        _loading = false;
      });
      if (!detail.isRead) {
        await _setState(_ArticleState.read, true);
        detail = _detail ?? detail;
      }
      final reading = ref.read(appearanceProvider).asData?.value.reading ??
          const bridge.ReadingSettings(
            font: 'system',
            fontSize: 17,
            lineHeight: 1.65,
            contentWidth: 680,
            showReadingTime: true,
            autoExtract: false,
          );
      if (reading.autoExtract &&
          !_autoExtractAttempted &&
          detail.url != null &&
          (detail.extractedHtml?.trim().isEmpty ?? true)) {
        _autoExtractAttempted = true;
        await _extract();
      }
    } catch (error) {
      if (mounted) {
        setState(() {
          _loading = false;
          _error = error;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    ref.listen(appearanceProvider, (_, next) {
      final reading = next.asData?.value.reading;
      if (reading?.autoExtract == true &&
          !_autoExtractAttempted &&
          !_extracting &&
          _detail?.url != null &&
          (_detail?.extractedHtml?.trim().isEmpty ?? true)) {
        _autoExtractAttempted = true;
        Future<void>.microtask(_extract);
      }
    });
    final detail = _detail;
    final reading = ref.watch(appearanceProvider).asData?.value.reading ??
        const bridge.ReadingSettings(
          font: 'system',
          fontSize: 17,
          lineHeight: 1.65,
          contentWidth: 680,
          showReadingTime: true,
          autoExtract: false,
        );
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.articleTitle),
        actions: detail == null
            ? null
            : [
                IconButton(
                  tooltip: detail.isRead
                      ? context.l10n.markUnread
                      : context.l10n.markRead,
                  icon: Icon(
                    detail.isRead
                        ? Icons.mark_email_read
                        : Icons.mark_email_unread,
                  ),
                  onPressed: () =>
                      _setState(_ArticleState.read, !detail.isRead),
                ),
                IconButton(
                  tooltip: detail.isStarred
                      ? context.l10n.unstar
                      : context.l10n.star,
                  icon:
                      Icon(detail.isStarred ? Icons.star : Icons.star_outline),
                  onPressed: () =>
                      _setState(_ArticleState.starred, !detail.isStarred),
                ),
                IconButton(
                  tooltip: detail.readLater
                      ? context.l10n.removeReadLater
                      : context.l10n.readLater,
                  icon: Icon(
                    detail.readLater ? Icons.bookmark : Icons.bookmark_outline,
                  ),
                  onPressed: () =>
                      _setState(_ArticleState.readLater, !detail.readLater),
                ),
                PopupMenuButton<_ReaderAction>(
                  tooltip: context.l10n.readerActions,
                  onSelected: _readerAction,
                  itemBuilder: (_) => [
                    PopupMenuItem(
                      value: _ReaderAction.extract,
                      enabled: detail.url != null && !_extracting,
                      child: Text(
                        detail.extractedHtml == null
                            ? context.l10n.extractFulltext
                            : context.l10n.reextractFulltext,
                      ),
                    ),
                    PopupMenuItem(
                      value: _ReaderAction.browser,
                      enabled: detail.url != null,
                      child: Text(context.l10n.openInBrowser),
                    ),
                    PopupMenuItem(
                      value: _ReaderAction.share,
                      enabled: detail.url != null,
                      child: Text(context.l10n.shareArticle),
                    ),
                    PopupMenuItem(
                      value: _ReaderAction.settings,
                      child: Text(context.l10n.readingSettings),
                    ),
                  ],
                ),
              ],
      ),
      body: _buildBody(detail, reading),
    );
  }

  Widget _buildBody(
    bridge.ArticleDetail? detail,
    bridge.ReadingSettings reading,
  ) {
    if (_loading && detail == null) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_error != null && detail == null) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(context.l10n.localizeError(_error!)),
            const SizedBox(height: 12),
            FilledButton(onPressed: _load, child: Text(context.l10n.retry)),
          ],
        ),
      );
    }
    if (detail == null) return const SizedBox.shrink();
    final content = detail.extractedHtml ?? detail.contentHtml;
    final hasContent = content?.trim().isNotEmpty ?? false;
    final textStyle = TextStyle(
      fontSize: reading.fontSize,
      height: reading.lineHeight,
      fontFamily: switch (reading.font) {
        'serif' => 'serif',
        'sans' => 'sans-serif',
        _ => null,
      },
    );

    return Stack(
      children: [
        SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Center(
            child: ConstrainedBox(
              constraints: BoxConstraints(maxWidth: reading.contentWidth),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    detail.title,
                    style: Theme.of(context).textTheme.headlineSmall,
                  ),
                  const SizedBox(height: 8),
                  Wrap(
                    spacing: 12,
                    runSpacing: 4,
                    children: [
                      Text(detail.feedTitle),
                      if (detail.author != null)
                        Text(context.l10n.byAuthor(detail.author!)),
                      if (detail.publishedAt != null)
                        Text(_shortDate(detail.publishedAt!)),
                      if (reading.showReadingTime && hasContent)
                        Text(context.l10n
                            .readMinutes(_readingMinutes(content!))),
                    ],
                  ),
                  const Divider(height: 28),
                  if (!hasContent)
                    Text(context.l10n.noContent)
                  else
                    HtmlWidget(
                      content!,
                      baseUrl:
                          detail.url == null ? null : Uri.tryParse(detail.url!),
                      onTapUrl: platformService.openUrl,
                      customWidgetBuilder: (element) {
                        if (element.localName != 'img') return null;
                        final src = element.attributes['src'];
                        final uri = src == null ? null : Uri.tryParse(src);
                        if (uri == null ||
                            (uri.scheme != 'http' && uri.scheme != 'https')) {
                          return const _BrokenBodyImage();
                        }
                        return Image.network(
                          src!,
                          fit: BoxFit.contain,
                          errorBuilder: (_, __, ___) =>
                              const _BrokenBodyImage(),
                        );
                      },
                      customStylesBuilder: (element) =>
                          element.localName == 'img'
                              ? {'max-width': '100%', 'height': 'auto'}
                              : null,
                      textStyle: textStyle,
                    ),
                  if (detail.enclosures.isNotEmpty) ...[
                    const Divider(height: 28),
                    Text(
                      context.l10n.attachments,
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    for (final enclosure in detail.enclosures)
                      ListTile(
                        contentPadding: EdgeInsets.zero,
                        leading: const Icon(Icons.attach_file),
                        title:
                            Text(enclosure.mimeType ?? context.l10n.attachment),
                        subtitle: Text(
                          enclosure.url,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                        onTap: () => platformService.openUrl(enclosure.url),
                      ),
                  ],
                ],
              ),
            ),
          ),
        ),
        if (_extracting) const LinearProgressIndicator(),
      ],
    );
  }

  Future<void> _setState(_ArticleState field, bool value) async {
    final previous = _detail;
    if (previous == null) return;
    setState(() => _detail = _copyDetail(previous, field, value));
    try {
      final repo = ref.read(articleRepositoryProvider);
      switch (field) {
        case _ArticleState.read:
          await repo.setRead(widget.articleId, value);
        case _ArticleState.starred:
          await repo.setStarred(widget.articleId, value);
        case _ArticleState.readLater:
          await repo.setReadLater(widget.articleId, value);
      }
      ref.invalidate(articleCountsProvider);
      ref.invalidate(articleCountProvider);
    } catch (error) {
      if (!mounted) return;
      setState(() => _detail = previous);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(context.l10n.localizeError(error))),
      );
    }
  }

  Future<void> _readerAction(_ReaderAction action) async {
    switch (action) {
      case _ReaderAction.extract:
        await _extract();
      case _ReaderAction.browser:
        final url = _detail?.url;
        if (url != null && !await platformService.openUrl(url) && mounted) {
          _showPlatformFailure();
        }
      case _ReaderAction.share:
        final detail = _detail;
        if (detail != null &&
            detail.url != null &&
            !await platformService.shareArticle(detail.title, detail.url!) &&
            mounted) {
          _showPlatformFailure();
        }
      case _ReaderAction.settings:
        await showModalBottomSheet<void>(
          context: context,
          isScrollControlled: true,
          builder: (_) => const _ReadingSettingsSheet(),
        );
    }
  }

  Future<void> _extract() async {
    if (_extracting || _detail?.url == null) return;
    setState(() => _extracting = true);
    try {
      await ref
          .read(articleRepositoryProvider)
          .extractFulltext(widget.articleId);
      final detail = await ref
          .read(articleRepositoryProvider)
          .getArticleDetail(widget.articleId);
      if (mounted) {
        setState(() => _detail = detail);
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.fulltextExtracted)),
        );
      }
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.localizeError(error))),
        );
      }
    } finally {
      if (mounted) setState(() => _extracting = false);
    }
  }

  void _showPlatformFailure() {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(context.l10n.platformActionFailed)),
    );
  }
}

class _ReadingSettingsSheet extends ConsumerWidget {
  const _ReadingSettingsSheet();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final current = ref.watch(appearanceProvider).asData?.value.reading;
    if (current == null) {
      return const SafeArea(child: LinearProgressIndicator());
    }
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              context.l10n.readingSettings,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 12),
            SegmentedButton<String>(
              segments: [
                ButtonSegment(
                    value: 'system', label: Text(context.l10n.fontSystem)),
                ButtonSegment(
                    value: 'serif', label: Text(context.l10n.fontSerif)),
                ButtonSegment(
                    value: 'sans', label: Text(context.l10n.fontSans)),
              ],
              selected: {current.font},
              onSelectionChanged: (value) => _save(
                context,
                ref,
                copyReadingSettings(current, font: value.first),
              ),
            ),
            _SettingSlider(
              label: context.l10n.fontSize,
              value: current.fontSize,
              min: 14,
              max: 24,
              onChanged: (value) => _save(
                context,
                ref,
                copyReadingSettings(current, fontSize: value),
              ),
            ),
            _SettingSlider(
              label: context.l10n.lineHeight,
              value: current.lineHeight,
              min: 1.3,
              max: 2,
              divisions: 14,
              onChanged: (value) => _save(
                context,
                ref,
                copyReadingSettings(current, lineHeight: value),
              ),
            ),
            _SettingSlider(
              label: context.l10n.readingWidth,
              value: current.contentWidth,
              min: 320,
              max: 840,
              divisions: 13,
              onChanged: (value) => _save(
                context,
                ref,
                copyReadingSettings(current, contentWidth: value),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _save(
    BuildContext context,
    WidgetRef ref,
    bridge.ReadingSettings settings,
  ) async {
    try {
      await ref.read(appearanceProvider.notifier).setReading(settings);
    } catch (error) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.localizeError(error))),
        );
      }
    }
  }
}

class _SettingSlider extends StatefulWidget {
  final String label;
  final double value;
  final double min;
  final double max;
  final int? divisions;
  final ValueChanged<double> onChanged;

  const _SettingSlider({
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    this.divisions,
    required this.onChanged,
  });

  @override
  State<_SettingSlider> createState() => _SettingSliderState();
}

class _SettingSliderState extends State<_SettingSlider> {
  late double _value = widget.value;

  @override
  void didUpdateWidget(covariant _SettingSlider oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.value != widget.value) _value = widget.value;
  }

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        SizedBox(width: 112, child: Text(widget.label)),
        Expanded(
          child: Slider(
            value: _value.clamp(widget.min, widget.max),
            min: widget.min,
            max: widget.max,
            divisions: widget.divisions,
            label: _value.toStringAsFixed(_value < 10 ? 1 : 0),
            onChanged: (value) => setState(() => _value = value),
            onChangeEnd: widget.onChanged,
          ),
        ),
      ],
    );
  }
}

class _BrokenBodyImage extends StatelessWidget {
  const _BrokenBodyImage();

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: const BoxConstraints(minHeight: 96),
      alignment: Alignment.center,
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.broken_image_outlined),
          Text(context.l10n.imageUnavailable),
        ],
      ),
    );
  }
}

bridge.ArticleDetail _copyDetail(
  bridge.ArticleDetail detail,
  _ArticleState field,
  bool value,
) {
  return bridge.ArticleDetail(
    id: detail.id,
    feedId: detail.feedId,
    feedTitle: detail.feedTitle,
    sourceType: detail.sourceType,
    title: detail.title,
    author: detail.author,
    url: detail.url,
    contentHtml: detail.contentHtml,
    extractedHtml: detail.extractedHtml,
    imageUrl: detail.imageUrl,
    publishedAt: detail.publishedAt,
    isRead: field == _ArticleState.read ? value : detail.isRead,
    isStarred: field == _ArticleState.starred ? value : detail.isStarred,
    readLater: field == _ArticleState.readLater ? value : detail.readLater,
    aiSummary: detail.aiSummary,
    translatedHtml: detail.translatedHtml,
    translatedLang: detail.translatedLang,
    enclosures: detail.enclosures,
    tags: detail.tags,
  );
}

int _readingMinutes(String html) {
  final text = html
      .replaceAll(RegExp(r'<[^>]+>'), ' ')
      .replaceAll(RegExp(r'\s+'), ' ')
      .trim();
  if (text.isEmpty) return 1;
  final wordCount = text.split(' ').where((word) => word.isNotEmpty).length;
  final estimate = wordCount > 20 ? wordCount / 220 : text.runes.length / 500;
  return math.max(1, estimate.ceil());
}

String _shortDate(String value) =>
    value.length >= 10 ? value.substring(0, 10) : value;

enum _ArticleState { read, starred, readLater }

enum _ReaderAction { extract, browser, share, settings }
