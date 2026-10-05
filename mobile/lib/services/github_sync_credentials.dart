import 'dart:math';
import '../bridge/generated/generated.dart' as bridge;
import '../core/exceptions.dart';
import 'platform_service.dart';

String githubRandomId() {
  final random = Random.secure();
  return List.generate(16, (_) => random.nextInt(256))
      .map((b) => b.toRadixString(16).padLeft(2, '0'))
      .join();
}

Future<String> githubInstallation({bool create = false}) async {
  var marker =
      await platformService.getSyncCredential('papr.sync.installation');
  if (marker == null && create) {
    marker = githubRandomId();
    if (!await platformService.setSyncCredential(
        'papr.sync.installation', marker)) {
      throw const AppException(
          AppErrorKind.sync, 'credentialWriteFailed', null);
    }
  }
  if (marker == null || marker.isEmpty) {
    throw const AppException(
        AppErrorKind.sync, 'githubInstallationMissing', null);
  }
  return marker;
}

Future<void> githubCheckpoint(
    bridge.PaprCoreBridge core, String credentialRef) async {
  final reference =
      'papr.sync.checkpoint_${credentialRef.substring('papr.sync.'.length)}';
  final previous = await platformService.getSyncCredential(reference);
  final checkpoint =
      await bridge.githubCheckpoint(core: core, previous: previous);
  if (!await platformService.setSyncCredential(reference, checkpoint)) {
    throw const AppException(AppErrorKind.sync, 'credentialWriteFailed', null);
  }
}
