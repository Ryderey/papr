// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Japanese (`ja`).
class AppLocalizationsJa extends AppLocalizations {
  AppLocalizationsJa([String locale = 'ja']) : super(locale);

  @override
  String get appTitle => 'Papr';

  @override
  String get navArticles => '記事';

  @override
  String get navSubscriptions => '購読';

  @override
  String get navSaved => '保存済み';

  @override
  String get navSettings => '設定';

  @override
  String get articlesTitle => '記事';

  @override
  String get savedTitle => '保存済み';

  @override
  String get subscriptionsTitle => '購読';

  @override
  String get settingsTitle => '設定';

  @override
  String get themeLabel => 'テーマ';

  @override
  String get themeSystem => 'システム';

  @override
  String get themeLight => 'ライト';

  @override
  String get themeDark => 'ダーク';

  @override
  String get languageLabel => '言語';

  @override
  String get languageEnglish => '英語';

  @override
  String get languageChinese => '簡体字中国語';

  @override
  String get languageJapanese => '日本語';

  @override
  String get refreshInterval => '更新間隔';

  @override
  String minutes(int count) {
    return '$count 分';
  }

  @override
  String get articleTitle => '記事';

  @override
  String byAuthor(String author) {
    return '作者：$author';
  }

  @override
  String get noContent => '本文がありません';

  @override
  String get noArticles => '記事はまだありません';

  @override
  String get noSavedArticles => '保存済みの記事はありません';

  @override
  String get noFeeds => '購読はまだありません。+ で追加できます。';

  @override
  String get selectSubscription => '購読を選択して記事を表示';

  @override
  String errorMessage(String message) {
    return 'エラー：$message';
  }

  @override
  String get refreshTooltip => '更新';

  @override
  String refreshComplete(int count) {
    return '更新完了：新着 $count 件。';
  }

  @override
  String refreshPartial(int newCount, int errorCount) {
    return '更新完了：新着 $newCount 件、$errorCount 件失敗。';
  }

  @override
  String refreshFailed(String message) {
    return '更新失敗：$message';
  }

  @override
  String get addFeed => '購読を追加';

  @override
  String get feedUrl => 'Feed またはウェブサイト URL';

  @override
  String get feedUrlHint => 'https://example.com/feed.xml';

  @override
  String get cancel => 'キャンセル';

  @override
  String get add => '追加';

  @override
  String get feedUrlRequired => 'Feed またはウェブサイト URL を入力してください';

  @override
  String get feedTitle => '購読名';

  @override
  String appearanceSaveFailed(String message) {
    return '表示設定を保存できません：$message';
  }

  @override
  String get unknownError => 'エラーが発生しました';

  @override
  String get manageFolders => 'フォルダーを管理';

  @override
  String get folderName => 'フォルダー名';

  @override
  String get createFolder => 'フォルダーを作成';

  @override
  String get rename => '名前を変更';

  @override
  String get delete => '削除';

  @override
  String get deleteFolderTitle => 'フォルダーを削除しますか？';

  @override
  String get deleteFolderMessage => 'このフォルダーの購読は「未分類」に移動します。';

  @override
  String get uncategorized => '未分類';

  @override
  String get feedActions => '購読の操作';

  @override
  String get renameFeed => '購読名を変更';

  @override
  String get moveToFolder => 'フォルダーへ移動';

  @override
  String get refreshIntervalTitle => '購読の更新間隔';

  @override
  String get useGlobalInterval => '全体設定を使用';

  @override
  String get refreshOff => '自動更新しない';

  @override
  String get deleteFeedTitle => '購読を削除しますか？';

  @override
  String deleteFeedMessage(String title) {
    return '「$title」とダウンロード済みの記事を削除しますか？';
  }

  @override
  String refreshOneComplete(String title, int count) {
    return '「$title」を更新しました：新着 $count 件。';
  }

  @override
  String get directory => 'ディレクトリ';

  @override
  String get search => '検索';

  @override
  String get directoryHint => '出版物やトピックを検索';

  @override
  String get noResults => '一致する購読はありません';

  @override
  String get opml => 'OPML';

  @override
  String get importOpml => 'OPML テキストをインポート';

  @override
  String get exportOpml => 'OPML をエクスポート';

  @override
  String get opmlRequired => '選択した OPML ファイルは空です。';

  @override
  String opmlImported(int imported, int failed) {
    return '$imported 件をインポート、$failed 件が失敗しました。';
  }

  @override
  String get opmlExported => 'OPML ファイルを保存しました。';

  @override
  String get close => '閉じる';

  @override
  String get errorEmptyFolderName => 'フォルダー名を入力してください。';

  @override
  String get errorFolderNameExists => '同じ名前のフォルダーが既にあります。';

  @override
  String get errorInvalidFolderOrder => 'フォルダー順が別の場所で変更されました。再読み込みしてお試しください。';

  @override
  String get errorEmptyFeedTitle => '購読名を入力してください。';

  @override
  String get errorFeedAlreadyExists => 'この購読は既に追加されています。';

  @override
  String get errorEmptyFeedUrl => '購読 URL を入力してください。';

  @override
  String get errorFeedNotFound => 'このアドレスに Feed が見つかりません。';

  @override
  String get errorInvalidFeedUrl => '有効な Feed またはウェブサイト URL を入力してください。';

  @override
  String get errorNetwork => 'ネットワーク要求に失敗しました。接続を確認してください。';

  @override
  String get errorParse => '購読データを読み取れませんでした。';
}
