import 'package:html/dom.dart' as dom;
import 'package:html/parser.dart' show parseFragment;

import '../bridge/generated/generated.dart' as bridge;

String highlightPlainText(String html) =>
    _textSegments(parseFragment(html)).map((segment) => segment.text).join();

String renderHighlightHtml(
  String html,
  Iterable<bridge.ResolvedHighlight> highlights,
) {
  final fragment = parseFragment(html);
  final ranges = highlights
      .where((resolved) =>
          resolved.start != null &&
          resolved.end != null &&
          resolved.end! > resolved.start!)
      .map(
        (resolved) => _HighlightRange(
          start: resolved.start!.toInt(),
          end: resolved.end!.toInt(),
          color: resolved.highlight.color,
        ),
      )
      .toList();
  if (ranges.isEmpty) return html;

  for (final segment in _textSegments(fragment)) {
    final matches = ranges
        .where(
            (range) => range.start < segment.end && range.end > segment.start)
        .toList();
    if (matches.isEmpty) continue;

    final boundaries = <int>{0, segment.text.length};
    for (final range in matches) {
      boundaries
          .add((range.start - segment.start).clamp(0, segment.text.length));
      boundaries.add((range.end - segment.start).clamp(0, segment.text.length));
    }
    final points = boundaries.toList()..sort();
    final replacement = dom.DocumentFragment();
    for (var index = 0; index < points.length - 1; index++) {
      final start = points[index];
      final end = points[index + 1];
      if (start == end) continue;
      final text = segment.text.substring(start, end);
      final range = matches.lastWhere(
        (candidate) =>
            candidate.start <= segment.start + start &&
            candidate.end >= segment.start + end,
        orElse: () => const _HighlightRange.empty(),
      );
      if (range.color == null) {
        replacement.append(dom.Text(text));
      } else {
        final mark = dom.Element.tag('mark')
          ..classes.add('papr-highlight')
          ..attributes['data-highlight-color'] = range.color!;
        mark.append(dom.Text(text));
        replacement.append(mark);
      }
    }
    segment.node.replaceWith(replacement);
  }
  return fragment.outerHtml;
}

List<_TextSegment> _textSegments(dom.Node root) {
  final segments = <_TextSegment>[];
  var offset = 0;

  void visit(dom.Node node) {
    if (node is dom.Element &&
        (node.localName == 'script' || node.localName == 'style')) {
      return;
    }
    if (node is dom.Text && node.data.isNotEmpty) {
      segments.add(_TextSegment(node, offset));
      offset += node.data.length;
      return;
    }
    for (final child in node.nodes.toList()) {
      visit(child);
    }
  }

  visit(root);
  return segments;
}

class _TextSegment {
  final dom.Text node;
  final int start;

  const _TextSegment(this.node, this.start);

  String get text => node.data;
  int get end => start + text.length;
}

class _HighlightRange {
  final int start;
  final int end;
  final String? color;

  const _HighlightRange({
    required this.start,
    required this.end,
    required this.color,
  });

  const _HighlightRange.empty()
      : start = 0,
        end = 0,
        color = null;
}
