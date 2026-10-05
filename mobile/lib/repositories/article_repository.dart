import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../core/di.dart';
import '../services/papr_core_service.dart';

final articleRepositoryProvider = Provider<ArticleRepository>((ref) {
  return ArticleRepository(ref);
});

final articleCountsProvider = FutureProvider<bridge.ArticleCounts>((ref) {
  return ref.watch(articleRepositoryProvider).getArticleCounts();
});

final articlePageProvider =
    FutureProvider.family<List<bridge.ArticleSummary>, bridge.ArticleFilter>(
  (ref, filter) {
    return ref.watch(articleRepositoryProvider).listArticles(filter: filter);
  },
);

final articleTagsProvider = FutureProvider<List<bridge.TagSummary>>((ref) {
  return ref.watch(articleRepositoryProvider).listTags();
});

final articleCountProvider =
    FutureProvider.family<int, bridge.ArticleFilter>((ref, filter) async {
  return ref.watch(articleRepositoryProvider).countArticles(filter);
});

class ArticleRepository {
  final Ref _ref;

  ArticleRepository(this._ref);

  Future<List<bridge.ArticleSummary>> listArticles(
      {bridge.ArticleFilter? filter}) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.getArticles(
        core: core,
        filter: filter ??
            const bridge.ArticleFilter(
              kind: bridge.ArticleFilterKind.all(),
              search: null,
              unreadOnly: false,
              oldestFirst: false,
              limit: null,
              offset: null,
            ),
      );
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<int> countArticles(bridge.ArticleFilter filter) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return (await bridge.countArticles(core: core, filter: filter)).toInt();
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<bridge.ArticleCounts> getArticleCounts() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.getArticleCounts(core: core);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<List<bridge.TagSummary>> listTags() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.listArticleTags(core: core);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<int> createTag(String name) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return (await bridge.createTag(core: core, name: name)).toInt();
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> renameTag(int id, String name) => _tagWrite(
        (core) => bridge.renameTag(core: core, id: id, name: name),
      );

  Future<void> setTagColor(int id, String color) => _tagWrite(
        (core) => bridge.setTagColor(core: core, id: id, color: color),
      );

  Future<void> reorderTags(List<int> ids) => _tagWrite(
        (core) => bridge.reorderTags(
          core: core,
          tagIds: Int64List.fromList(ids),
        ),
      );

  Future<void> deleteTag(int id) => _tagWrite(
        (core) => bridge.deleteTag(core: core, id: id),
      );

  Future<void> setArticleTag(int articleId, int tagId, bool attached) =>
      _tagWrite(
        (core) => bridge.setArticleTag(
          core: core,
          articleId: articleId,
          tagId: tagId,
          attached: attached,
        ),
      );

  Future<List<bridge.Rule>> listRules() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.listRules(core: core);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<int> createRule(bridge.RuleInput input) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return (await bridge.createRule(core: core, input: input)).toInt();
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> updateRule(int id, bridge.RuleInput input) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.updateRule(core: core, id: id, input: input);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> deleteRule(int id) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.deleteRule(core: core, id: id);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<bridge.RulePreview> previewRule(bridge.RuleInput input) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.previewRule(core: core, input: input);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<int> applyRuleToExisting(bridge.RuleInput input) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return (await bridge.applyRuleToExisting(core: core, input: input))
          .toInt();
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<List<bridge.Highlight>> listHighlights(int articleId) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.listHighlights(core: core, articleId: articleId);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<List<bridge.Highlight>> listAllHighlights() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.listAllHighlights(core: core);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<int> createHighlight(bridge.HighlightInput input) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return (await bridge.createHighlight(core: core, input: input)).toInt();
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> updateHighlightNote(int id, String note) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.updateHighlightNote(core: core, id: id, note: note);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> setHighlightColor(int id, String color) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.setHighlightColor(core: core, id: id, color: color);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> deleteHighlight(int id) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.deleteHighlight(core: core, id: id);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<List<bridge.ResolvedHighlight>> resolveHighlights(
    int articleId,
    String text,
  ) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.resolveHighlights(
        core: core,
        articleId: articleId,
        text: text,
      );
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<bridge.ArticleDetail> getArticleDetail(int articleId) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.getArticleDetail(core: core, articleId: articleId);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> setRead(int articleId, bool value) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.setArticleRead(
        core: core,
        articleId: articleId,
        value: value,
      );
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> setStarred(int articleId, bool value) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.setArticleStarred(
        core: core,
        articleId: articleId,
        value: value,
      );
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> setReadLater(int articleId, bool value) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.setArticleReadLater(
        core: core,
        articleId: articleId,
        value: value,
      );
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<int> markAllRead(bridge.ArticleFilter filter) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return (await bridge.markAllArticlesRead(core: core, filter: filter))
          .toInt();
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<String> extractFulltext(int articleId) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.extractArticleFulltext(
        core: core,
        articleId: articleId,
      );
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> _tagWrite(
    Future<void> Function(bridge.PaprCoreBridge core) write,
  ) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await write(core);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }
}
