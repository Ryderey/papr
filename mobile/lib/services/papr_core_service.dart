import '../../bridge/generated/generated.dart' as gen;
import '../core/exceptions.dart';

/// Thin wrapper around the FRB-generated API.
///
/// This class keeps the generated bindings out of repositories and widgets.
class PaprCoreService {
  final gen.PaprCoreBridge bridge;

  PaprCoreService(this.bridge);

  static AppException mapError(Object err) {
    if (err is gen.PaprBridgeError) {
      final kind = switch (err) {
        gen.PaprBridgeError_Database() => AppErrorKind.database,
        gen.PaprBridgeError_Network() => AppErrorKind.network,
        gen.PaprBridgeError_Parse() => AppErrorKind.parse,
        gen.PaprBridgeError_InvalidInput() => AppErrorKind.invalidInput,
        gen.PaprBridgeError_NotFound() => AppErrorKind.notFound,
        gen.PaprBridgeError_Ai() => AppErrorKind.ai,
        gen.PaprBridgeError_Platform() => AppErrorKind.platform,
        gen.PaprBridgeError_Unknown() => AppErrorKind.unknown,
      };
      return AppException(kind, err.toString());
    }
    return AppException(AppErrorKind.unknown, err.toString());
  }
}
