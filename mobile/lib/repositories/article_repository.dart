import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../core/di.dart';
import '../services/papr_core_service.dart';

final articleRepositoryProvider = Provider<ArticleRepository>((ref) {
  return ArticleRepository(ref);
});

class ArticleRepository {
  final Ref _ref;

  ArticleRepository(this._ref);

  Future<List<bridge.ArticleSummary>> listArticles({bridge.ArticleFilter? filter}) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.getArticles(
        core: core,
        filter: filter ??
            const bridge.ArticleFilter(
              kind: bridge.ArticleFilterKind.all(),
              limit: null,
              offset: null,
            ),
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
}
