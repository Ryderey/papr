import 'dart:async';

import 'package:flutter/services.dart';

class PlatformService {
  static const _channel = MethodChannel('com.papr.papr_mobile/platform');

  final _deepLinks = StreamController<String>.broadcast();

  PlatformService() {
    _channel.setMethodCallHandler((call) async {
      if (call.method == 'deepLink' && call.arguments is String) {
        _deepLinks.add(call.arguments as String);
      }
    });
  }

  Stream<String> get deepLinks => _deepLinks.stream;

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
    final uri = Uri.tryParse(url);
    if (uri == null || (uri.scheme != 'http' && uri.scheme != 'https')) {
      return false;
    }
    try {
      return await _channel.invokeMethod<bool>('openUrl', {'url': url}) ??
          false;
    } on MissingPluginException {
      return false;
    }
  }

  Future<bool> shareArticle(String title, String url) async {
    final uri = Uri.tryParse(url);
    if (uri == null || (uri.scheme != 'http' && uri.scheme != 'https')) {
      return false;
    }
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
}

final platformService = PlatformService();
