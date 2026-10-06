import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../bridge/generated/generated.dart' as bridge;
import '../core/di.dart';
import '../core/exceptions.dart';
import '../services/papr_core_service.dart';
import '../services/platform_service.dart';
import '../services/github_sync_credentials.dart';
import '../services/background_refresh_service.dart';
import 'article_repository.dart';
import 'feed_repository.dart';

final githubSyncRepositoryProvider =
    Provider((ref) => GithubSyncRepository(ref));
final githubSyncStatusProvider = FutureProvider<bridge.GithubStatus>(
    (ref) => ref.read(githubSyncRepositoryProvider).status());
final githubScheduleProvider = FutureProvider<bridge.GithubSchedule>(
    (ref) => ref.read(githubSyncRepositoryProvider).schedule());

class GithubSyncRepository {
  final Ref _ref;
  GithubSyncRepository(this._ref);
  Future<T> _guard<T>(Future<T> Function() action) async {
    try {
      return await action();
    } catch (error) {
      final platformError = githubPlatformError(error);
      if (platformError != null) {
        try {
          await bridge.githubReportPlatformFailure(
              core: await _ref.read(paprCoreBridgeProvider.future),
              code: platformError.code);
        } catch (_) {/* Preserve the original platform error. */}
        throw platformError;
      }
      final mapped = PaprCoreService.mapError(error);
      throw AppException(AppErrorKind.sync, mapped.code, null);
    }
  }

  Future<bridge.GithubStatus> status() => _guard(() async => bridge
      .githubStatus(core: await _ref.read(paprCoreBridgeProvider.future)));
  Future<bool> automaticDue() => _guard(() async => bridge.githubAutomaticDue(
      core: await _ref.read(paprCoreBridgeProvider.future)));
  Future<bridge.GithubSchedule> schedule() => _guard(() async => bridge
      .githubSchedule(core: await _ref.read(paprCoreBridgeProvider.future)));
  Future<void> setSchedule(bridge.GithubSchedule schedule) => _guard(() async {
        final core = await _ref.read(paprCoreBridgeProvider.future);
        final previous = await bridge.githubSchedule(core: core);
        await bridge.githubSetSchedule(core: core, schedule: schedule);
        try {
          await const BackgroundRefreshService().reconcileCore(core);
        } catch (_) {
          await bridge.githubSetSchedule(core: core, schedule: previous);
          try {
            await const BackgroundRefreshService().reconcileCore(core);
          } catch (_) {/* Keep the original platform error. */}
          rethrow;
        } finally {
          _ref.invalidate(githubScheduleProvider);
          _ref.invalidate(githubSyncStatusProvider);
        }
      });
  Future<bridge.GithubPreview> preview(
          String owner, String repo, String branch, String token) =>
      _guard(() async => bridge.githubPreview(
          core: await _ref.read(paprCoreBridgeProvider.future),
          owner: owner.trim(),
          repo: repo.trim(),
          branch: branch.trim().isEmpty ? null : branch.trim(),
          credentialRef: 'papr.sync.${githubRandomId()}',
          token: token));
  Future<void> connect(bridge.GithubPreview preview, String token) =>
      _guard(() async {
        final core = await _ref.read(paprCoreBridgeProvider.future);
        if ((await bridge.githubStatus(core: core)).profile != null) {
          throw const AppException(
              AppErrorKind.sync, 'githubAlreadyConnected', null);
        }
        final installation = await githubInstallation(create: true);
        final reference = preview.profile.credentialRef;
        if (!await platformService.setSyncCredential(reference, token)) {
          throw const AppException(
              AppErrorKind.sync, 'credentialWriteFailed', null);
        }
        try {
          await bridge.githubConnect(
              core: core,
              preview: preview,
              token: token,
              installation: installation);
        } catch (_) {
          // A concurrent confirmation may now own this credential reference.
          try {
            final active = (await bridge.githubStatus(core: core)).profile;
            if (active?.credentialRef != reference) {
              await platformService.deleteSyncCredential(reference);
            }
          } catch (_) {
            /* Preserve the original failure and uncertain ownership. */
          }
          rethrow;
        }
        _ref.invalidate(githubSyncStatusProvider);
        await _schedule();
      });
  Future<void> updateToken(String token) => _guard(() async {
        final core = await _ref.read(paprCoreBridgeProvider.future);
        final profile = (await status()).profile;
        if (profile == null) {
          throw const AppException(
              AppErrorKind.sync, 'githubNotConnected', null);
        }
        await bridge.githubVerifyCredential(core: core, token: token);
        if (!await platformService.setSyncCredential(
            profile.credentialRef, token)) {
          throw const AppException(
              AppErrorKind.sync, 'credentialWriteFailed', null);
        }
        await bridge.githubCredentialUpdated(
            core: core, credentialRef: profile.credentialRef);
        _ref.invalidate(githubSyncStatusProvider);
      });
  Future<bridge.GithubSyncReport> syncNow() => _guard(() async {
        final core = await _ref.read(paprCoreBridgeProvider.future);
        final profile = (await status()).profile;
        if (profile == null) {
          throw const AppException(
              AppErrorKind.sync, 'githubNotConnected', null);
        }
        final token =
            await platformService.getSyncCredential(profile.credentialRef);
        if (token == null || token.isEmpty) {
          throw const AppException(
              AppErrorKind.sync, 'syncCredentialMissing', null);
        }
        await githubCheckpoint(core, profile.credentialRef);
        try {
          final report = await bridge.githubSyncNow(
              core: core,
              token: token,
              installation: await githubInstallation());
          await githubCheckpoint(core, profile.credentialRef);
          if (!report.unchanged) {
            _ref.invalidate(articleRepositoryProvider);
            _ref.invalidate(feedRepositoryProvider);
          }
          return report;
        } finally {
          _ref.invalidate(githubSyncStatusProvider);
        }
      });
  Future<void> cancel() => _guard(() async => bridge.githubCancelSync(
      core: await _ref.read(paprCoreBridgeProvider.future)));
  Future<void> _schedule() async {
    try {
      final core = await _ref.read(paprCoreBridgeProvider.future);
      await const BackgroundRefreshService().reconcileCore(core);
    } catch (_) {/* Startup reconciliation retries platform scheduling. */}
  }

  Future<void> disconnect() => _guard(() async {
        final reference = await bridge.githubDisconnect(
            core: await _ref.read(paprCoreBridgeProvider.future));
        if (reference != null &&
            !await platformService.deleteSyncCredential(reference)) {
          throw const AppException(
              AppErrorKind.sync, 'credentialDeleteFailed', null);
        }
        _ref.invalidate(githubSyncStatusProvider);
        await _schedule();
      });
}
