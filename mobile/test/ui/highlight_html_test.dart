import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/bridge/generated/generated.dart' as bridge;
import 'package:papr_mobile/ui/highlight_html.dart';

void main() {
  test('renders resolved highlights without changing surrounding HTML', () {
    const html = '<p>Before <strong>selected text</strong> after.</p>';
    expect(highlightPlainText(html), 'Before selected text after.');

    final rendered = renderHighlightHtml(html, [
      bridge.ResolvedHighlight(
        highlight: _highlight(),
        start: 7,
        end: 20,
      ),
    ]);

    expect(
      rendered,
      '<p>Before <strong><mark class="papr-highlight" data-highlight-color="green">selected text</mark></strong> after.</p>',
    );
  });

  test('leaves unresolved highlights out of the rendered article', () {
    const html = '<p>Current text</p>';
    expect(
        renderHighlightHtml(
            html, [bridge.ResolvedHighlight(highlight: _highlight())]),
        html);
  });

  test('renders a highlight that crosses inline HTML nodes', () {
    const html = '<p>Before <strong>selected</strong> text after.</p>';
    expect(highlightPlainText(html), 'Before selected text after.');

    final rendered = renderHighlightHtml(html, [
      bridge.ResolvedHighlight(
        highlight: _highlight(),
        start: 7,
        end: 20,
      ),
    ]);

    expect(
      rendered,
      '<p>Before <strong><mark class="papr-highlight" data-highlight-color="green">selected</mark></strong><mark class="papr-highlight" data-highlight-color="green"> text</mark> after.</p>',
    );
  });
}

bridge.Highlight _highlight() => const bridge.Highlight(
      id: 1,
      articleId: 1,
      quote: 'selected text',
      prefix: 'Before ',
      suffix: ' after.',
      textOffset: 7,
      color: 'green',
      note: '',
      createdAt: '2026-08-25T00:00:00Z',
    );
