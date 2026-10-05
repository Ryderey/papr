import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/core/exceptions.dart';
import 'package:papr_mobile/services/github_sync_credentials.dart';
import 'package:papr_mobile/services/platform_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel('com.papr.papr_mobile/sync_credentials');
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  tearDown(() => messenger.setMockMethodCallHandler(channel, null));

  test('native credential failures retain safe codes used to pause sync',
      () async {
    for (final method in ['getSyncCredential', 'setSyncCredential']) {
      final code = method == 'getSyncCredential'
          ? 'credentialReadFailed'
          : 'credentialWriteFailed';
      messenger.setMockMethodCallHandler(
          channel,
          (_) async => throw PlatformException(
              code: code, message: 'private native detail'));
      AppException? mapped;
      try {
        if (method == 'getSyncCredential') {
          await platformService.getSyncCredential('papr.sync.test');
        } else {
          await platformService.setSyncCredential(
              'papr.sync.test', 'test marker');
        }
        fail('expected a native storage failure');
      } catch (error) {
        mapped = githubPlatformError(error);
      }
      expect(mapped?.code, code);
      expect(mapped?.kind, AppErrorKind.sync);
      expect(mapped?.detail, isNull);
      expect(mapped.toString(), isNot(contains('private native detail')));
    }
  });
}
