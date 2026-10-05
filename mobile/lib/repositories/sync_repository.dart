import 'dart:math';

import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../core/di.dart';
import '../core/exceptions.dart';
import '../services/papr_core_service.dart';
import '../services/platform_service.dart';

final syncRepositoryProvider = Provider<SyncRepository>(
  (ref) => SyncRepository(ref, platformService),
);

final syncStatusProvider = FutureProvider<bridge.SyncStatus>(
  (ref) => ref.watch(syncRepositoryProvider).status(),
);

class SyncRepository {
  final Ref _ref;
  final PlatformService _platform;

  SyncRepository(this._ref, this._platform);

  Future<bridge.SyncStatus> status() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.getSyncStatus(core: core);
    } catch (error) {
      throw _coreError(error);
    }
  }

  Future<void> testConnection({
    required bridge.SyncProvider provider,
    required String serverUrl,
    required String username,
    required String credential,
  }) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.testSyncConnection(
        core: core,
        profile: _profile(provider, serverUrl, username),
        credential: credential,
      );
    } catch (error) {
      throw _coreError(error);
    }
  }

  Future<void> connect({
    required bridge.SyncProvider provider,
    required String serverUrl,
    required String username,
    required String credential,
  }) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    final previous = await status();
    final profile = _profile(provider, serverUrl, username);
    final stored = await _platformCall(
      () => _platform.setSyncCredential(profile.credentialRef, credential),
    );
    if (!stored) throw _credentialError('credentialWriteFailed');
    try {
      await bridge.connectSyncProfile(
        core: core,
        profile: profile,
        credential: credential,
      );
    } catch (error) {
      var removed = false;
      try {
        removed = await _platform.deleteSyncCredential(profile.credentialRef);
      } catch (_) {
        // The saved connection still points to the previous credential.
      }
      if (!removed) throw _credentialError('credentialDeleteFailed');
      throw _coreError(error);
    }
    final oldRef = previous.profile?.credentialRef;
    if (oldRef != null && oldRef != profile.credentialRef) {
      final removed = await _platformCall(
        () => _platform.deleteSyncCredential(oldRef),
      );
      if (!removed) throw _credentialError('credentialDeleteFailed');
    }
  }

  Future<int> syncNow() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    final profile = (await status()).profile;
    if (profile == null) {
      throw const AppException(AppErrorKind.sync, 'syncNotConnected', null);
    }
    final credential = await _platformCall(
      () => _platform.getSyncCredential(profile.credentialRef),
    );
    if (credential == null || credential.isEmpty) {
      throw _credentialError('syncCredentialMissing');
    }
    try {
      return (await bridge.syncNow(core: core, credential: credential)).toInt();
    } catch (error) {
      throw _coreError(error);
    }
  }

  Future<void> disconnect() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    String? credentialRef;
    try {
      credentialRef = await bridge.deleteSyncProfile(core: core);
    } catch (error) {
      throw _coreError(error);
    }
    if (credentialRef != null) {
      final deleted = await _platformCall(
        () => _platform.deleteSyncCredential(credentialRef!),
      );
      if (!deleted) throw _credentialError('credentialDeleteFailed');
    }
  }

  static bridge.SyncProfile _profile(
    bridge.SyncProvider provider,
    String serverUrl,
    String username,
  ) {
    final random = Random.secure();
    final suffix = List.generate(16, (_) => random.nextInt(256))
        .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
        .join();
    return bridge.SyncProfile(
      provider: provider,
      serverUrl: serverUrl.trim(),
      username: username.trim(),
      credentialRef: 'papr.sync.$suffix',
    );
  }

  static AppException _credentialError(String code) =>
      AppException(AppErrorKind.sync, code, null);

  static AppException _coreError(Object error) {
    final mapped = PaprCoreService.mapError(error);
    return AppException(mapped.kind, mapped.code, null);
  }

  static Future<T> _platformCall<T>(Future<T> Function() call) async {
    try {
      return await call();
    } on PlatformException catch (error) {
      throw _credentialError(error.code);
    }
  }
}
