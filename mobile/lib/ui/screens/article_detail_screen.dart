import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_widget_from_html_core/flutter_widget_from_html_core.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../../l10n/l10n.dart';
import '../../repositories/article_repository.dart';
import '../../repositories/settings_repository.dart';
import '../../services/platform_service.dart';
import '../highlight_html.dart';
import '../highlight_style.dart';
import 'ai_summary_screen.dart';

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
  List<bridge.Highlight> _highlights = const [];
  List<bridge.ResolvedHighlight> _resolvedHighlights = const [];
  String? _highlightedHtml;
  String _selectedText = '';

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
      await _loadHighlights();
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
                  tooltip: context.l10n.aiSummary,
                  icon: const Icon(Icons.auto_awesome_outlined),
                  onPressed: () => Navigator.of(context).push(
                    MaterialPageRoute<void>(
                      builder: (_) =>
                          AiSummaryScreen(articleId: widget.articleId),
                    ),
                  ),
                ),
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
    final renderedContent = _highlightedHtml ?? content;
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
                    SelectionArea(
                      onSelectionChanged: (selection) {
                        _selectedText = selection?.plainText.trim() ?? '';
                      },
                      contextMenuBuilder: (context, selectableRegionState) {
                        final actions = [
                          ...selectableRegionState.contextMenuButtonItems,
                        ];
                        if (_selectedText.isNotEmpty) {
                          actions.add(
                            ContextMenuButtonItem(
                              label: context.l10n.addHighlight,
                              onPressed: () {
                                selectableRegionState.hideToolbar();
                                _createHighlight(content!);
                              },
                            ),
                          );
                        }
                        return AdaptiveTextSelectionToolbar.buttonItems(
                          anchors: selectableRegionState.contextMenuAnchors,
                          buttonItems: actions,
                        );
                      },
                      child: HtmlWidget(
                        renderedContent!,
                        baseUrl: detail.url == null
                            ? null
                            : Uri.tryParse(detail.url!),
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
                        customStylesBuilder: (element) {
                          if (element.localName == 'img') {
                            return {'max-width': '100%', 'height': 'auto'};
                          }
                          if (element.localName == 'mark' &&
                              element.classes.contains('papr-highlight')) {
                            return {
                              'background-color': highlightCssColor(
                                element.attributes['data-highlight-color'] ??
                                    'yellow',
                              ),
                            };
                          }
                          return null;
                        },
                        textStyle: textStyle,
                      ),
                    ),
                  const Divider(height: 28),
                  Text(
                    context.l10n.highlights,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  if (_resolvedHighlights.any(
                    (highlight) =>
                        highlight.start == null || highlight.end == null,
                  ))
                    Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Text(context.l10n.highlightUnableToLocate),
                    ),
                  if (_highlights.isEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Text(context.l10n.noHighlights),
                    )
                  else
                    for (final highlight in _highlights)
                      ListTile(
                        contentPadding: EdgeInsets.zero,
                        leading: _HighlightColor(color: highlight.color),
                        title: Text(highlight.quote),
                        subtitle: highlight.note.isEmpty
                            ? null
                            : Text(highlight.note),
                        trailing: IconButton(
                          tooltip: context.l10n.delete,
                          icon: const Icon(Icons.delete_outline),
                          onPressed: () => _editHighlight(highlight),
                        ),
                        onTap: () => _editHighlight(highlight),
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
      ref.invalidate(articleDetailProvider(widget.articleId));
      ref.invalidate(articlePageProvider);
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
        setState(() {
          _detail = detail;
          _highlightedHtml = null;
          _resolvedHighlights = const [];
        });
        await _loadHighlights();
        if (!mounted) return;
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

  Future<void> _loadHighlights() async {
    try {
      final repository = ref.read(articleRepositoryProvider);
      final highlights = await repository.listHighlights(widget.articleId);
      final content = _detail?.extractedHtml ?? _detail?.contentHtml;
      final resolved = content == null
          ? const <bridge.ResolvedHighlight>[]
          : await repository.resolveHighlights(
              widget.articleId,
              highlightPlainText(content),
            );
      if (mounted) {
        setState(() {
          _highlights = highlights;
          _resolvedHighlights = resolved;
          _highlightedHtml =
              content == null ? null : renderHighlightHtml(content, resolved);
        });
      }
    } catch (_) {
      // Highlight loading must not block the existing reader content.
    }
  }

  Future<void> _createHighlight(String html) async {
    final selected = _selectedText;
    if (selected.isEmpty) return;
    final text = highlightPlainText(html);
    final start = text.indexOf(selected);
    if (start < 0) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.highlightSelectionUnavailable)),
        );
      }
      return;
    }
    final draft = await showModalBottomSheet<_HighlightDraft>(
      context: context,
      isScrollControlled: true,
      builder: (_) => _HighlightEditorSheet(quote: selected),
    );
    if (draft == null) return;
    final end = start + selected.length;
    final prefix = text.substring(math.max(0, start - 32), start);
    final suffix = text.substring(end, math.min(text.length, end + 32));
    try {
      await ref.read(articleRepositoryProvider).createHighlight(
            bridge.HighlightInput(
              articleId: widget.articleId,
              quote: selected,
              prefix: prefix,
              suffix: suffix,
              textOffset: start,
              color: draft.color,
              note: draft.note,
            ),
          );
      _selectedText = '';
      await _loadHighlights();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.highlightCreated)),
        );
      }
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.localizeError(error))),
        );
      }
    }
  }

  Future<void> _editHighlight(bridge.Highlight highlight) async {
    final draft = await showModalBottomSheet<_HighlightDraft>(
      context: context,
      isScrollControlled: true,
      builder: (_) => _HighlightEditorSheet(
        quote: highlight.quote,
        initialColor: highlight.color,
        initialNote: highlight.note,
        allowDelete: true,
      ),
    );
    if (draft == null) return;
    try {
      final repository = ref.read(articleRepositoryProvider);
      if (draft.delete) {
        await repository.deleteHighlight(highlight.id.toInt());
      } else {
        await repository.setHighlightColor(highlight.id.toInt(), draft.color);
        await repository.updateHighlightNote(highlight.id.toInt(), draft.note);
      }
      await _loadHighlights();
    } catch (error) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(context.l10n.localizeError(error))),
        );
      }
    }
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

class _HighlightColor extends StatelessWidget {
  final String color;

  const _HighlightColor({required this.color});

  @override
  Widget build(BuildContext context) => CircleAvatar(
        radius: 10,
        backgroundColor: highlightColor(color),
      );
}

class _HighlightEditorSheet extends StatefulWidget {
  final String quote;
  final String initialColor;
  final String initialNote;
  final bool allowDelete;

  const _HighlightEditorSheet({
    required this.quote,
    this.initialColor = 'yellow',
    this.initialNote = '',
    this.allowDelete = false,
  });

  @override
  State<_HighlightEditorSheet> createState() => _HighlightEditorSheetState();
}

class _HighlightEditorSheetState extends State<_HighlightEditorSheet> {
  static const _colors = ['yellow', 'green', 'blue', 'pink', 'purple'];
  late String _color = widget.initialColor;
  late final TextEditingController _note =
      TextEditingController(text: widget.initialNote);

  @override
  void dispose() {
    _note.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => SafeArea(
        child: Padding(
          padding: EdgeInsets.fromLTRB(
            20,
            20,
            20,
            20 + MediaQuery.viewInsetsOf(context).bottom,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(widget.quote, maxLines: 3, overflow: TextOverflow.ellipsis),
              const SizedBox(height: 16),
              Text(context.l10n.highlightColor),
              Wrap(
                spacing: 8,
                children: [
                  for (final color in _colors)
                    ChoiceChip(
                      label: const SizedBox.shrink(),
                      avatar: _HighlightColor(color: color),
                      selected: _color == color,
                      onSelected: (_) => setState(() => _color = color),
                    ),
                ],
              ),
              const SizedBox(height: 12),
              TextField(
                controller: _note,
                autofocus: !widget.allowDelete,
                maxLines: 3,
                decoration:
                    InputDecoration(labelText: context.l10n.highlightNote),
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  if (widget.allowDelete)
                    TextButton.icon(
                      onPressed: () => Navigator.pop(
                        context,
                        const _HighlightDraft(delete: true),
                      ),
                      icon: const Icon(Icons.delete_outline),
                      label: Text(context.l10n.delete),
                    ),
                  const Spacer(),
                  TextButton(
                    onPressed: () => Navigator.pop(context),
                    child: Text(context.l10n.cancel),
                  ),
                  FilledButton(
                    onPressed: () => Navigator.pop(
                      context,
                      _HighlightDraft(color: _color, note: _note.text),
                    ),
                    child: Text(context.l10n.save),
                  ),
                ],
              ),
            ],
          ),
        ),
      );
}

class _HighlightDraft {
  final String color;
  final String note;
  final bool delete;

  const _HighlightDraft({
    this.color = 'yellow',
    this.note = '',
    this.delete = false,
  });
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
