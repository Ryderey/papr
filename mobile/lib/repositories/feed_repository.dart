import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart'
    as frb;

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
      return await bridge.addFeed(
        core: core,
        input: bridge.AddFeedInput(input: feedUrl.trim()),
      );
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<List<bridge.Folder>> listFolders() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.listFolders(core: core);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> createFolder(String name) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.createFolder(core: core, name: name.trim());
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> renameFolder(bridge.Folder folder, String name) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.renameFolder(core: core, id: folder.id, name: name.trim());
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> deleteFolder(bridge.Folder folder) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.deleteFolder(core: core, id: folder.id);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> reorderFolders(List<bridge.Folder> folders) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.reorderFolders(
        core: core,
        folderIds: frb.Int64List.fromList(
          folders.map((folder) => folder.id.toInt()).toList(),
        ),
      );
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> deleteFeed(bridge.Feed feed) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.deleteFeed(core: core, id: feed.id);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> renameFeed(bridge.Feed feed, String title) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.renameFeed(core: core, id: feed.id, title: title.trim());
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> moveFeed(bridge.Feed feed, bridge.Folder? folder) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.moveFeed(core: core, id: feed.id, folderId: folder?.id);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<void> setFeedRefreshInterval(
    bridge.Feed feed,
    int? minutes,
  ) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.setFeedRefreshInterval(
        core: core,
        id: feed.id,
        minutes: minutes,
      );
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<bridge.RefreshReport> refreshFeed(bridge.Feed feed) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.refreshFeed(core: core, id: feed.id);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<List<bridge.DiscoveryResult>> searchDirectory(
    String query,
    String language,
  ) async {
    try {
      return await bridge.searchDirectory(
        query: query.trim(),
        lang: language,
      );
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<String?> parseDeepLink(String url) async {
    try {
      return await bridge.parseDeepLink(url: url);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<bridge.OpmlImportReport> importOpml(String text) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.importOpml(core: core, opmlText: text);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<String> exportOpml() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.exportOpml(core: core);
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }

  Future<bridge.RefreshReport> refreshFeeds({
    bridge.RefreshOptions? options,
  }) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.refreshFeeds(
        core: core,
        options:
            options ?? const bridge.RefreshOptions(feedIds: null, force: false),
      );
    } catch (e) {
      throw PaprCoreService.mapError(e);
    }
  }
}
