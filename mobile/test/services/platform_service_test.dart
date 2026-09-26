import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/services/platform_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('com.papr.papr_mobile/platform');
  const syncChannel = MethodChannel('com.papr.papr_mobile/sync_credentials');
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  tearDown(() async {
    messenger.setMockMethodCallHandler(channel, null);
    messenger.setMockMethodCallHandler(syncChannel, null);
  });

  test('credential references are restricted to their Papr namespaces', () {
    expect(PlatformService.isValidAiCredentialRef('papr.ai.profile_1'), isTrue);
    expect(PlatformService.isValidAiCredentialRef('profile_1'), isFalse);
    expect(PlatformService.isValidAiCredentialRef('papr.ai.bad/ref'), isFalse);
    expect(
        PlatformService.isValidAiCredentialRef('papr.sync.profile_1'), isFalse);
    expect(PlatformService.isValidSyncCredentialRef('papr.sync.profile_1'),
        isTrue);
    expect(
        PlatformService.isValidSyncCredentialRef('papr.ai.profile_1'), isFalse);
  });

  test('credential calls use the secure platform channel contract', () async {
    final calls = <MethodCall>[];
    messenger.setMockMethodCallHandler(channel, (call) async {
      calls.add(call);
      return switch (call.method) {
        'setAiCredential' => true,
        'getAiCredential' => 'transient-secret',
        'deleteAiCredential' => true,
        _ => null,
      };
    });
    messenger.setMockMethodCallHandler(syncChannel, (call) async {
      calls.add(call);
      return switch (call.method) {
        'setSyncCredential' => true,
        'getSyncCredential' => 'sync-secret',
        'deleteSyncCredential' => true,
        _ => null,
      };
    });
    final service = PlatformService();
    const credentialRef = 'papr.ai.profile_1';
    const syncCredentialRef = 'papr.sync.primary';

    expect(await service.setAiCredential(credentialRef, 'transient-secret'),
        isTrue);
    expect(await service.getAiCredential(credentialRef), 'transient-secret');
    expect(await service.deleteAiCredential(credentialRef), isTrue);
    expect(await service.setSyncCredential(syncCredentialRef, 'sync-secret'),
        isTrue);
    expect(await service.getSyncCredential(syncCredentialRef), 'sync-secret');
    expect(await service.deleteSyncCredential(syncCredentialRef), isTrue);

    expect(calls.map((call) => call.method), [
      'setAiCredential',
      'getAiCredential',
      'deleteAiCredential',
      'setSyncCredential',
      'getSyncCredential',
      'deleteSyncCredential',
    ]);
    expect(calls.first.arguments, {
      'credentialRef': credentialRef,
      'secret': 'transient-secret',
    });
    expect(calls[1].arguments, {'credentialRef': credentialRef});
    expect(calls[2].arguments, {'credentialRef': credentialRef});
    expect(calls[3].arguments, {
      'credentialRef': syncCredentialRef,
      'secret': 'sync-secret',
    });
    expect(calls[4].arguments, {'credentialRef': syncCredentialRef});
    expect(calls[5].arguments, {'credentialRef': syncCredentialRef});
  });

  test('invalid credential input never crosses the platform channel', () async {
    var callCount = 0;
    messenger.setMockMethodCallHandler(channel, (call) async {
      callCount += 1;
      return true;
    });
    messenger.setMockMethodCallHandler(syncChannel, (call) async {
      callCount += 1;
      return true;
    });
    final service = PlatformService();

    expect(
        await service.setAiCredential('outside.namespace', 'secret'), isFalse);
    expect(await service.setAiCredential('papr.ai.profile_1', '   '), isFalse);
    expect(await service.getAiCredential('outside.namespace'), isNull);
    expect(await service.deleteAiCredential('outside.namespace'), isFalse);
    expect(await service.setSyncCredential('papr.ai.profile_1', 'secret'),
        isFalse);
    expect(await service.getSyncCredential('papr.ai.profile_1'), isNull);
    expect(await service.deleteSyncCredential('papr.ai.profile_1'), isFalse);
    expect(callCount, 0);
  });

  test('playback commands validate inputs and use the platform contract',
      () async {
    final calls = <MethodCall>[];
    messenger.setMockMethodCallHandler(channel, (call) async {
      calls.add(call);
      return switch (call.method) {
        'startPlayback' || 'playbackCommand' => true,
        'getPlaybackState' => {
            'mediaId': 'article-1',
            'title': 'Episode',
            'source': 'Feed',
            'durationMs': 60000,
            'positionMs': 15000,
            'speed': 1.25,
            'playing': true,
            'buffering': false,
            'error': null,
          },
        _ => null,
      };
    });
    final service = PlatformService();

    expect(
      await service.startPlayback(
        mediaId: 'article-1',
        url: 'https://cdn.example.com/episode.mp3',
        title: 'Episode',
        source: 'Feed',
      ),
      isTrue,
    );
    expect(await service.playbackCommand('setSpeed', speed: 1.25), isTrue);
    expect(await service.playbackCommand('seekTo', positionMs: -1), isFalse);
    expect(
        await service.startPlayback(
          mediaId: 'article-1',
          url: 'file:///episode.mp3',
          title: 'Episode',
          source: 'Feed',
        ),
        isFalse);

    final state = await service.getPlaybackState();
    expect(state.mediaId, 'article-1');
    expect(state.positionMs, 15000);
    expect(state.speed, 1.25);
    expect(calls.map((call) => call.method), [
      'startPlayback',
      'playbackCommand',
      'getPlaybackState',
    ]);
  });
}
