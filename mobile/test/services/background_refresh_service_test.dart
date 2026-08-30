import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:papr_mobile/services/background_refresh_service.dart';

void main() {
  test('configured Android notification icon exists as a drawable resource',
      () {
    expect(
      File(
        'android/app/src/main/res/drawable/$androidNotificationIconName.xml',
      ).existsSync(),
      isTrue,
    );
  });

  test('background schedule respects off and Android frequency bounds', () {
    expect(isBackgroundRefreshEnabled(30), isTrue);
    expect(isBackgroundRefreshEnabled(refreshOffMinutes), isFalse);
    expect(backgroundRefreshFrequencyMinutes(5), 15);
    expect(backgroundRefreshFrequencyMinutes(60), 60);
    expect(backgroundRefreshFrequencyMinutes(240), 120);
  });

  test('notification eligibility respects count, enablement, and quiet hours',
      () {
    expect(
      shouldShowNewArticleNotification(
        count: 2,
        enabled: true,
        quietHours: false,
        now: DateTime(2026, 8, 29, 23),
      ),
      isTrue,
    );
    expect(
      shouldShowNewArticleNotification(
        count: 2,
        enabled: true,
        quietHours: true,
        now: DateTime(2026, 8, 29, 23),
      ),
      isFalse,
    );
    expect(
      shouldShowNewArticleNotification(
        count: 2,
        enabled: true,
        quietHours: true,
        now: DateTime(2026, 8, 29, 8),
      ),
      isTrue,
    );
    expect(
      shouldShowNewArticleNotification(
        count: 0,
        enabled: true,
        quietHours: false,
        now: DateTime(2026, 8, 29, 12),
      ),
      isFalse,
    );
  });

  test('notification summary is localized without article content', () {
    expect(newArticleNotificationBody('zh', 3), '3 篇新文章');
    expect(newArticleNotificationBody('ja', 3), '新着記事 3 件');
    expect(newArticleNotificationBody('en', 1), '1 new article');
    expect(newArticleNotificationBody('en', 3), '3 new articles');
  });
}
