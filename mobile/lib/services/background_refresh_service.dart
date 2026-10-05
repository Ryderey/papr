import 'dart:io';
import 'dart:ui';

import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:workmanager/workmanager.dart';

import '../bridge/generated/generated.dart' as bridge;
import '../bridge/generated/frb_generated.dart';
import '../core/config.dart';
import '../l10n/app_localizations.dart';
import 'platform_service.dart';
import 'github_sync_credentials.dart';

const refreshOffMinutes = 525600;
const backgroundRefreshUniqueName = 'papr.background.refresh';
const backgroundRefreshTaskName = 'papr.refresh.due';
const androidNotificationIconName = 'ic_notification';

final backgroundRefreshServiceProvider = Provider<BackgroundRefreshService>(
  (_) => const BackgroundRefreshService(),
);

@pragma('vm:entry-point')
void backgroundRefreshDispatcher() {
  Workmanager().executeTask((taskName, _) async {
    if (taskName != backgroundRefreshTaskName) return true;
    DartPluginRegistrant.ensureInitialized();
    try {
      await RustLib.init();
      final core = await bridge.initPaprCore(config: await buildCoreConfig());
      final report = await bridge.refreshFeeds(
        core: core,
        options: const bridge.RefreshOptions(feedIds: null, force: false),
      );
      final settings = await bridge.getSettings(core: core);
      if (shouldShowNewArticleNotification(
        count: report.newArticles.toInt(),
        enabled: settings.notificationsEnabled,
        quietHours: settings.notificationQuietHours,
        now: DateTime.now(),
      )) {
        await _showNewArticleNotification(
          language: settings.language,
          count: report.newArticles.toInt(),
        );
      }
      try {
        final github = await bridge.githubStatus(core: core);
        if (github.profile != null && github.backgroundDue) {
          final token = await platformService
              .getSyncCredential(github.profile!.credentialRef);
          if (token == null || token.isEmpty) {
            await bridge.githubReportPlatformFailure(
                core: core, code: 'syncCredentialMissing');
          } else {
            await githubCheckpoint(core, github.profile!.credentialRef);
            await bridge.githubSyncNow(
                core: core,
                token: token,
                installation: await githubInstallation());
            await githubCheckpoint(core, github.profile!.credentialRef);
          }
        }
      } catch (error) {
        final platformError = githubPlatformError(error);
        if (platformError != null) {
          try {
            await bridge.githubReportPlatformFailure(
                core: core, code: platformError.code);
          } catch (_) {/* Preserve the successful feed refresh. */}
        }
        // Keep the successful feed refresh; sync status permits manual retry.
      }
      try {
        final status = await bridge.getSyncStatus(core: core);
        if (status.backgroundDue && status.profile != null) {
          String? credential;
          try {
            credential = await platformService
                .getSyncCredential(status.profile!.credentialRef);
          } catch (_) {
            // Core records a stable missing-credential error below.
          }
          await bridge.syncNow(core: core, credential: credential ?? '');
        }
      } catch (_) {
        // Feed refresh succeeded; sync shows its stable error for manual retry.
      }
      return true;
    } catch (_) {
      return false;
    }
  });
}

class BackgroundRefreshService {
  const BackgroundRefreshService();

  static Future<void> initialize() async {
    if (!Platform.isAndroid) return;
    await Workmanager().initialize(backgroundRefreshDispatcher);
  }

  Future<void> reconcile(int refreshIntervalMin,
      {bool githubEnabled = false}) async {
    if (!Platform.isAndroid) return;
    if (!isBackgroundRefreshEnabled(refreshIntervalMin) && !githubEnabled) {
      await Workmanager().cancelByUniqueName(backgroundRefreshUniqueName);
      return;
    }
    await Workmanager().registerPeriodicTask(
      backgroundRefreshUniqueName,
      backgroundRefreshTaskName,
      frequency: Duration(
        minutes:
            !isBackgroundRefreshEnabled(refreshIntervalMin) && githubEnabled
                ? 360
                : backgroundRefreshFrequencyMinutes(refreshIntervalMin),
      ),
      constraints: Constraints(networkType: NetworkType.connected),
      existingWorkPolicy: ExistingPeriodicWorkPolicy.update,
      backoffPolicy: BackoffPolicy.exponential,
      backoffPolicyDelay: const Duration(minutes: 15),
    );
  }

  Future<bool> requestNotificationPermission() async {
    if (!Platform.isAndroid) return true;
    final plugin = await _initializedNotifications();
    final android = plugin.resolvePlatformSpecificImplementation<
        AndroidFlutterLocalNotificationsPlugin>();
    return await android?.requestNotificationsPermission() ?? true;
  }
}

bool isBackgroundRefreshEnabled(int refreshIntervalMin) =>
    refreshIntervalMin < refreshOffMinutes;

int backgroundRefreshFrequencyMinutes(int refreshIntervalMin) =>
    refreshIntervalMin.clamp(15, 120);

bool shouldShowNewArticleNotification({
  required int count,
  required bool enabled,
  required bool quietHours,
  required DateTime now,
}) {
  if (count <= 0 || !enabled) return false;
  return !quietHours || (now.hour >= 8 && now.hour < 22);
}

String newArticleNotificationBody(String language, int count) =>
    lookupAppLocalizations(Locale(language)).newArticleNotificationBody(count);

Future<FlutterLocalNotificationsPlugin> _initializedNotifications() async {
  final plugin = FlutterLocalNotificationsPlugin();
  await plugin.initialize(
    settings: const InitializationSettings(
      android: AndroidInitializationSettings(androidNotificationIconName),
    ),
  );
  return plugin;
}

Future<void> _showNewArticleNotification({
  required String language,
  required int count,
}) async {
  final plugin = await _initializedNotifications();
  final android = plugin.resolvePlatformSpecificImplementation<
      AndroidFlutterLocalNotificationsPlugin>();
  if (await android?.areNotificationsEnabled() == false) return;
  final localizations = lookupAppLocalizations(Locale(language));
  await plugin.show(
    id: 6101,
    title: localizations.appTitle,
    body: localizations.newArticleNotificationBody(count),
    notificationDetails: const NotificationDetails(
      android: AndroidNotificationDetails(
        'papr_new_articles',
        'New articles',
        channelDescription: 'New article summaries after background refresh',
      ),
    ),
  );
}
