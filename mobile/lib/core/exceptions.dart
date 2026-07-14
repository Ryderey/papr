enum AppErrorKind {
  database,
  network,
  parse,
  invalidInput,
  notFound,
  ai,
  platform,
  unknown,
}

class AppException implements Exception {
  final AppErrorKind kind;
  final String message;

  const AppException(this.kind, this.message);

  @override
  String toString() => 'AppException($kind): $message';
}
