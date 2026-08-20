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
      final kind = switch (err.category) {
        gen.ErrorCategory.db => AppErrorKind.database,
        gen.ErrorCategory.network => AppErrorKind.network,
        gen.ErrorCategory.parse => AppErrorKind.parse,
        gen.ErrorCategory.invalidInput => AppErrorKind.invalidInput,
        gen.ErrorCategory.notFound => AppErrorKind.notFound,
        gen.ErrorCategory.ai => AppErrorKind.ai,
        gen.ErrorCategory.platform => AppErrorKind.platform,
        gen.ErrorCategory.sync_ => AppErrorKind.sync,
        gen.ErrorCategory.unknown => AppErrorKind.unknown,
      };
      return AppException(kind, err.code, err.detail);
    }
    return AppException(AppErrorKind.unknown, 'unknown', err.toString());
  }
}
