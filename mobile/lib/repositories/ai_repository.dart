import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/generated/generated.dart' as bridge;
import '../core/di.dart';
import '../core/exceptions.dart';
import '../services/papr_core_service.dart';
import '../services/platform_service.dart';

final aiRepositoryProvider = Provider<AiRepository>((ref) {
  return AiRepository(ref, platformService);
});

final aiProfilesProvider = FutureProvider<List<bridge.AiProfile>>((ref) {
  return ref.watch(aiRepositoryProvider).listProfiles();
});

final aiProfileConnectionProbeProvider = StateProvider<String?>((ref) => null);
final aiProfileEnabledToggleProvider = StateProvider<String?>((ref) => null);

class AiRepository {
  final Ref _ref;
  final PlatformService _platform;

  AiRepository(this._ref, this._platform);

  Future<List<bridge.AiProfile>> listProfiles() async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.listAiProfiles(core: core);
    } catch (error) {
      throw PaprCoreService.mapError(error);
    }
  }

  Future<void> saveProfile(
    bridge.AiProfile profile, {
    String? newSecret,
  }) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    bridge.AiProfile? existing;
    try {
      final profiles = await bridge.listAiProfiles(core: core);
      for (final item in profiles) {
        if (item.id == profile.id) {
          existing = item;
          break;
        }
      }
    } catch (error) {
      throw PaprCoreService.mapError(error);
    }
    final credentialRef = profile.credentialRef;
    String? previousSecret;
    var credentialWritten = false;

    if (profile.auth != bridge.AiAuthMode.none) {
      if (credentialRef == null ||
          !PlatformService.isValidAiCredentialRef(credentialRef)) {
        throw const AppException(
          AppErrorKind.invalidInput,
          'invalidAiProfile',
          null,
        );
      }
      previousSecret = await _readCredential(credentialRef);
      if (newSecret?.trim().isNotEmpty == true) {
        final stored = await _writeCredential(credentialRef, newSecret!);
        if (!stored) {
          throw const AppException(
            AppErrorKind.platform,
            'credentialWriteFailed',
            null,
          );
        }
        credentialWritten = true;
      } else if (previousSecret == null) {
        throw const AppException(AppErrorKind.ai, 'noAiCredential', null);
      }
    }

    try {
      await bridge.saveAiProfile(core: core, profile: profile);
    } catch (error) {
      if (credentialWritten && credentialRef != null) {
        if (previousSecret == null) {
          await _platform.deleteAiCredential(credentialRef);
        } else {
          await _writeCredential(credentialRef, previousSecret);
        }
      }
      throw PaprCoreService.mapError(error);
    }

    final oldCredentialRef = existing?.credentialRef;
    if (oldCredentialRef != null && oldCredentialRef != credentialRef) {
      try {
        await _platform.deleteAiCredential(oldCredentialRef);
      } catch (_) {
        // Metadata is already correct; stale secure-storage cleanup is best-effort.
      }
    }
  }

  Future<void> deleteProfile(String profileId) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    String? credentialRef;
    try {
      credentialRef = await bridge.deleteAiProfile(
        core: core,
        profileId: profileId,
      );
    } catch (error) {
      throw PaprCoreService.mapError(error);
    }
    if (credentialRef != null) {
      try {
        await _platform.deleteAiCredential(credentialRef);
      } catch (_) {
        // Profile metadata was deleted first; key cleanup must not restore it.
      }
    }
  }

  Future<void> setProfileEnabled(String profileId, bool enabled) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      await bridge.setAiProfileEnabled(
        core: core,
        profileId: profileId,
        enabled: enabled,
      );
    } catch (error) {
      throw PaprCoreService.mapError(error);
    }
  }

  bridge.AiProfile? summaryProfile(List<bridge.AiProfile> profiles) {
    for (final profile in profiles) {
      if (profile.enabled &&
          profile.defaultFor.contains(bridge.AiPurpose.summary)) {
        return profile;
      }
    }
    for (final profile in profiles) {
      if (profile.enabled) return profile;
    }
    return null;
  }

  Future<void> testConnection(bridge.AiProfile profile) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    final credential = await _credentialFor(profile);
    try {
      await bridge.testAiConnection(
        core: core,
        profile: profile,
        credential: credential,
      );
    } catch (error) {
      throw PaprCoreService.mapError(error);
    }
  }

  Future<bridge.AiSummaryCache?> getSummaryCache(int articleId) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.getAiSummaryCache(
        core: core,
        articleId: articleId,
      );
    } catch (error) {
      throw PaprCoreService.mapError(error);
    }
  }

  Stream<bridge.AiStreamEvent> summarize({
    required int articleId,
    required bridge.AiProfile profile,
    required bridge.SummaryTemplate template,
    required String language,
    required String requestId,
  }) async* {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    final credential = await _credentialFor(profile);
    try {
      yield* bridge.streamAiSummary(
        core: core,
        articleId: articleId,
        profile: profile,
        credential: credential,
        template: template,
        language: language,
        requestId: requestId,
      );
    } catch (error) {
      throw PaprCoreService.mapError(error);
    }
  }

  Stream<bridge.AiStreamEvent> followUp({
    required bridge.AiProfile profile,
    required String summary,
    required List<bridge.AiFollowUpTurn> history,
    required String question,
    required String language,
    required String requestId,
  }) async* {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    final credential = await _credentialFor(profile);
    try {
      yield* bridge.streamAiFollowUp(
        core: core,
        profile: profile,
        credential: credential,
        summary: summary,
        history: history,
        question: question,
        language: language,
        requestId: requestId,
      );
    } catch (error) {
      throw PaprCoreService.mapError(error);
    }
  }

  Future<bool> cancel(String requestId) async {
    final core = await _ref.read(paprCoreBridgeProvider.future);
    try {
      return await bridge.cancelAiRequest(core: core, requestId: requestId);
    } catch (error) {
      throw PaprCoreService.mapError(error);
    }
  }

  Future<String?> _credentialFor(bridge.AiProfile profile) async {
    if (profile.auth == bridge.AiAuthMode.none) return null;
    final credentialRef = profile.credentialRef;
    if (credentialRef == null) {
      throw const AppException(AppErrorKind.ai, 'noAiCredential', null);
    }
    final credential = await _readCredential(credentialRef);
    if (credential == null || credential.trim().isEmpty) {
      throw const AppException(AppErrorKind.ai, 'noAiCredential', null);
    }
    return credential;
  }

  Future<String?> _readCredential(String credentialRef) async {
    try {
      return await _platform.getAiCredential(credentialRef);
    } on PlatformException catch (error) {
      throw AppException(
        AppErrorKind.platform,
        error.code,
        null,
      );
    }
  }

  Future<bool> _writeCredential(String credentialRef, String secret) async {
    try {
      return await _platform.setAiCredential(credentialRef, secret);
    } on PlatformException catch (error) {
      throw AppException(AppErrorKind.platform, error.code, null);
    }
  }
}
