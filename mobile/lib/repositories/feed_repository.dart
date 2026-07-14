import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../core/di.dart';
import '../services/papr_core_service.dart';

final feedRepositoryProvider = Provider<FeedRepository>((ref) {
  return FeedRepository(ref);
});

class FeedRepository {
  final Ref _ref;

  FeedRepository(this._ref);

  Future<List<bridge.Feed>> listFeeds() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.getFeeds(core: core);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<bridge.Feed> addFeed(String feedUrl) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.addFeed(core: core, feedUrl: feedUrl.trim());
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }
}
