import 'dart:async';

import 'package:flutter/services.dart';

class PlatformService {
  static const _channel = MethodChannel('com.papr.papr_mobile/platform');
  static const _playbackChannel = EventChannel(
    'com.papr.papr_mobile/platform/playback',
  );
  static final _aiCredentialRefPattern = RegExp(
    r'^papr\.ai\.[A-Za-z0-9_-]{1,80}$',
  );

  final _deepLinks = StreamController<String>.broadcast();
  late final Stream<PlaybackState> _playbackStates = _playbackChannel
      .receiveBroadcastStream()
      .where((event) => event is Map)
      .map((event) =>
          PlaybackState.fromMap(Map<Object?, Object?>.from(event as Map)));

  PlatformService() {
    _channel.setMethodCallHandler((call) async {
      if (call.method == 'deepLink' && call.arguments is String) {
        _deepLinks.add(call.arguments as String);
      }
    });
  }

  Stream<String> get deepLinks => _deepLinks.stream;

  Stream<PlaybackState> get playbackStates => _playbackStates;

  Future<PlaybackState> getPlaybackState() async {
    try {
      final value = await _channel.invokeMapMethod<String, dynamic>(
        'getPlaybackState',
      );
      return PlaybackState.fromMap(value ?? const {});
    } on MissingPluginException {
      return PlaybackState.empty;
    }
  }

  Future<bool> startPlayback({
    required String mediaId,
    required String url,
    required String title,
    required String source,
  }) async {
    if (!_isHttpUrl(url)) return false;
    try {
      return await _channel.invokeMethod<bool>('startPlayback', {
            'mediaId': mediaId,
            'url': url,
            'title': title,
            'source': source,
          }) ??
          false;
    } on MissingPluginException {
      return false;
    }
  }

  Future<bool> playbackCommand(
    String command, {
    int? positionMs,
    double? speed,
  }) async {
    const validCommands = {
      'play',
      'pause',
      'skipBack',
      'skipForward',
      'stop',
      'seekTo',
      'setSpeed',
    };
    if (!validCommands.contains(command) ||
        (command == 'seekTo' && (positionMs == null || positionMs < 0)) ||
        (command == 'setSpeed' &&
            (speed == null || speed < 0.75 || speed > 2))) {
      return false;
    }
    try {
      return await _channel.invokeMethod<bool>('playbackCommand', {
            'command': command,
            if (positionMs != null) 'positionMs': positionMs,
            if (speed != null) 'speed': speed,
          }) ??
          false;
    } on MissingPluginException {
      return false;
    }
  }

  Future<String?> getInitialDeepLink() async {
    try {
      return await _channel.invokeMethod<String>('getInitialDeepLink');
    } on MissingPluginException {
      return null;
    }
  }

  Future<String?> openOpmlDocument() async {
    try {
      return await _channel.invokeMethod<String>('openOpmlDocument');
    } on MissingPluginException {
      return null;
    }
  }

  Future<bool> saveOpmlDocument(String text) async {
    try {
      return await _channel.invokeMethod<bool>(
            'saveOpmlDocument',
            {'text': text},
          ) ??
          false;
    } on MissingPluginException {
      return false;
    }
  }

  Future<bool> openUrl(String url) async {
    if (!_isHttpUrl(url)) return false;
    try {
      return await _channel.invokeMethod<bool>('openUrl', {'url': url}) ??
          false;
    } on MissingPluginException {
      return false;
    }
  }

  Future<bool> shareArticle(String title, String url) async {
    if (!_isHttpUrl(url)) return false;
    try {
      return await _channel.invokeMethod<bool>(
            'shareArticle',
            {'title': title, 'url': url},
          ) ??
          false;
    } on MissingPluginException {
      return false;
    }
  }

  static bool isValidAiCredentialRef(String value) {
    return _aiCredentialRefPattern.hasMatch(value);
  }

  Future<bool> setAiCredential(String credentialRef, String secret) async {
    if (!isValidAiCredentialRef(credentialRef) ||
        secret.trim().isEmpty ||
        secret.length > 8192) {
      return false;
    }
    try {
      return await _channel.invokeMethod<bool>(
            'setAiCredential',
            {'credentialRef': credentialRef, 'secret': secret},
          ) ??
          false;
    } on MissingPluginException {
      return false;
    }
  }

  Future<String?> getAiCredential(String credentialRef) async {
    if (!isValidAiCredentialRef(credentialRef)) return null;
    try {
      return await _channel.invokeMethod<String>(
        'getAiCredential',
        {'credentialRef': credentialRef},
      );
    } on MissingPluginException {
      return null;
    }
  }

  Future<bool> deleteAiCredential(String credentialRef) async {
    if (!isValidAiCredentialRef(credentialRef)) return false;
    try {
      return await _channel.invokeMethod<bool>(
            'deleteAiCredential',
            {'credentialRef': credentialRef},
          ) ??
          false;
    } on MissingPluginException {
      return false;
    }
  }

  static bool _isHttpUrl(String value) {
    final uri = Uri.tryParse(value);
    return uri != null &&
        (uri.scheme == 'http' || uri.scheme == 'https') &&
        uri.host.isNotEmpty;
  }
}

class PlaybackState {
  final String mediaId;
  final String title;
  final String source;
  final int durationMs;
  final int positionMs;
  final double speed;
  final bool playing;
  final bool buffering;
  final String? error;

  const PlaybackState({
    required this.mediaId,
    required this.title,
    required this.source,
    required this.durationMs,
    required this.positionMs,
    required this.speed,
    required this.playing,
    required this.buffering,
    required this.error,
  });

  static const empty = PlaybackState(
    mediaId: '',
    title: '',
    source: '',
    durationMs: 0,
    positionMs: 0,
    speed: 1,
    playing: false,
    buffering: false,
    error: null,
  );

  factory PlaybackState.fromMap(Map<Object?, Object?> value) => PlaybackState(
        mediaId: value['mediaId'] as String? ?? '',
        title: value['title'] as String? ?? '',
        source: value['source'] as String? ?? '',
        durationMs: (value['durationMs'] as num?)?.toInt() ?? 0,
        positionMs: (value['positionMs'] as num?)?.toInt() ?? 0,
        speed: (value['speed'] as num?)?.toDouble() ?? 1,
        playing: value['playing'] as bool? ?? false,
        buffering: value['buffering'] as bool? ?? false,
        error: value['error'] as String?,
      );
}

final platformService = PlatformService();
