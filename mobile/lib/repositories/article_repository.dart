import 'package:flutter_riverpod/flutter_riverpod.dart';

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
}
