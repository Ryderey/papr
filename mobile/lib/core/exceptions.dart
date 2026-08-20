enum AppErrorKind {
  database,
  network,
  parse,
  invalidInput,
  notFound,
  ai,
  platform,
  sync,
  unknown,
}

/// Domain-level error surfaced to the UI.
///
/// [kind] is the coarse category (drives icons/general handling); [code] is a
/// stable machine-readable identifier suitable for localisation; [detail] is an
/// optional safe context (never contains secrets).
class AppException implements Exception {
  final AppErrorKind kind;
  final String code;
  final String? detail;

  const AppException(this.kind, this.code, this.detail);

  @override
  String toString() {
    final base = 'AppException($kind, $code)';
    return detail == null ? base : '$base: $detail';
  }
}
