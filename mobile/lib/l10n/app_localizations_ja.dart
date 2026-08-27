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

  @override
  String get retry => '再試行';

  @override
  String get articleActions => '記事の操作';

  @override
  String get readerActions => 'リーダーの操作';

  @override
  String get markRead => '既読にする';

  @override
  String get markUnread => '未読にする';

  @override
  String get star => 'お気に入り';

  @override
  String get unstar => 'お気に入りを解除';

  @override
  String get readLater => 'あとで読む';

  @override
  String get removeReadLater => 'あとで読むから削除';

  @override
  String get searchArticles => 'タイトルと全文を検索';

  @override
  String get listOptions => '一覧オプション';

  @override
  String get hideRead => '既読記事を隠す';

  @override
  String get oldestFirst => '古い順';

  @override
  String get markAllRead => '現在の一覧をすべて既読にする';

  @override
  String markedAllRead(int count) {
    return '$count 件の記事を既読にしました。';
  }

  @override
  String get smartViews => 'スマートビュー';

  @override
  String get allArticles => 'すべての記事';

  @override
  String get unreadArticles => '未読';

  @override
  String get starredArticles => 'お気に入り';

  @override
  String get readLaterArticles => 'あとで読む';

  @override
  String get folders => 'フォルダー';

  @override
  String get tags => 'タグ';

  @override
  String get extractFulltext => '全文を抽出';

  @override
  String get reextractFulltext => '全文を再抽出';

  @override
  String get fulltextExtracted => '全文を抽出しました。';

  @override
  String get openInBrowser => 'ブラウザーで開く';

  @override
  String get shareArticle => '共有';

  @override
  String get platformActionFailed => 'この操作を実行できるアプリがありません。';

  @override
  String get readingSettings => '読書設定';

  @override
  String get readerFont => 'リーダーフォント';

  @override
  String get fontSystem => 'システム';

  @override
  String get fontSerif => 'セリフ';

  @override
  String get fontSans => 'サンセリフ';

  @override
  String get fontSize => '文字サイズ';

  @override
  String get lineHeight => '行間';

  @override
  String get readingWidth => '本文幅';

  @override
  String get showReadingTime => '読了時間を表示';

  @override
  String get autoExtractFulltext => '全文を自動抽出';

  @override
  String get autoExtractFulltextDescription => '全文キャッシュがない場合に元ページを取得します。';

  @override
  String readMinutes(int count) {
    return '読了目安 $count 分';
  }

  @override
  String get attachments => '添付ファイル';

  @override
  String get attachment => '添付ファイル';

  @override
  String get imageUnavailable => '画像を読み込めません';

  @override
  String get errorArticleNotFound => '記事が見つかりません。';

  @override
  String get errorArticleUrlMissing => 'この記事には元ページの URL がありません。';

  @override
  String get errorNoExtractableContent => '抽出できる本文が見つかりませんでした。';

  @override
  String get errorInvalidReadingSettings => '読書設定が対応範囲外です。';

  @override
  String get addHighlight => 'ハイライトを追加';

  @override
  String get highlights => 'ハイライト';

  @override
  String get noHighlights => 'ハイライトはまだありません。本文を選択して追加できます。';

  @override
  String get highlightColor => '色';

  @override
  String get highlightNote => 'メモ';

  @override
  String get highlightCreated => 'ハイライトを追加しました。';

  @override
  String get highlightSelectionUnavailable =>
      'この選択範囲は本文内で固定できません。もう一度選択してください。';

  @override
  String get highlightUnableToLocate => '一部のハイライトを現在の本文で見つけられません。';

  @override
  String get save => '保存';

  @override
  String get errorEmptyTagName => 'タグ名は空にできません。';

  @override
  String get errorTagNameExists => '同じ名前のタグが既にあります。';

  @override
  String get errorInvalidTagColor => '対応するタグ色を選択してください。';

  @override
  String get errorInvalidTagOrder => 'タグ順が別の場所で変更されました。再読み込みしてやり直してください。';

  @override
  String get errorRuleNotFound => 'このルールは既に存在しません。';

  @override
  String get errorEmptyRuleName => 'ルール名は空にできません。';

  @override
  String get errorEmptyRuleQuery => 'ルールのキーワードを少なくとも一つ入力してください。';

  @override
  String get errorInvalidRuleField => '対応するルール項目を選択してください。';

  @override
  String get errorInvalidRuleAction => '対応するルール操作を選択してください。';

  @override
  String get errorHighlightNotFound => 'このハイライトは既に存在しません。';

  @override
  String get errorEmptyHighlight => 'ハイライトを作成する文字を選択してください。';

  @override
  String get errorInvalidHighlightOffset => 'このハイライト位置は無効です。';

  @override
  String get errorInvalidHighlightColor => '対応するハイライト色を選択してください。';

  @override
  String get manageTags => 'タグを管理';

  @override
  String get manageRules => 'ルールを管理';

  @override
  String get tagName => 'タグ名';

  @override
  String get createTag => 'タグを作成';

  @override
  String get newRule => '新しいルール';

  @override
  String get ruleName => 'ルール名';

  @override
  String get ruleKeywords => 'キーワード（カンマ区切り）';

  @override
  String get ruleField => '一致項目';

  @override
  String get ruleAction => '操作';

  @override
  String get ruleEnabled => '有効';

  @override
  String get rulePreview => 'プレビュー';

  @override
  String rulePreviewResult(int count) {
    return '$count 件の記事に一致';
  }

  @override
  String get applyRule => '既存の記事に適用';

  @override
  String get applySkipRuleTitle => 'スキップルールを適用しますか？';

  @override
  String get applySkipRuleMessage =>
      '一致する未保存の記事を削除します。スター、後で読む、ハイライト済みの記事は残ります。';

  @override
  String applyRuleComplete(Object count) {
    return '$count 件の記事に適用しました。';
  }

  @override
  String get titleField => 'タイトル';

  @override
  String get authorField => '著者';

  @override
  String get contentField => '本文';

  @override
  String get anyField => 'すべての項目';

  @override
  String get skipAction => 'スキップ';

  @override
  String get readAction => '既読にする';

  @override
  String get starAction => 'スター';

  @override
  String get editTags => 'タグを編集';

  @override
  String get newTagHint => '新しいタグを作成';

  @override
  String get globalHighlights => 'すべてのハイライト';

  @override
  String get noGlobalHighlights => 'ハイライトはまだありません。';

  @override
  String get allFeeds => 'すべての購読';

  @override
  String get aiProfiles => 'AI プロファイル';

  @override
  String get noAiProfiles => 'AI プロファイルがありません。要約を使うには追加してください。';

  @override
  String get addAiProfile => 'AI プロファイルを追加';

  @override
  String get editAiProfile => 'AI プロファイルを編集';

  @override
  String get deleteAiProfileTitle => 'AI プロファイルを削除しますか？';

  @override
  String get profileName => 'プロファイル名';

  @override
  String get aiProtocol => 'プロトコル';

  @override
  String get aiModel => 'モデル';

  @override
  String get aiBaseUrl => 'Base URL';

  @override
  String get aiAuth => '認証';

  @override
  String get aiApiKey => 'API キー';

  @override
  String get aiApiKeyKeep => '空欄の場合は現在のキーを保持します。';

  @override
  String get openaiCompatible => 'OpenAI 互換';

  @override
  String get anthropic => 'Anthropic Messages';

  @override
  String get bearerAuth => 'Bearer トークン';

  @override
  String get xApiKeyAuth => 'x-api-key';

  @override
  String get noAuth => '認証なし';

  @override
  String get useForSummary => '要約に使用';

  @override
  String get testAiConnection => '接続をテスト';

  @override
  String get aiConnectionSucceeded => 'AI サービスへの接続に成功しました。';

  @override
  String get errorInvalidAiProfile => 'AI プロファイルの必須項目を入力してください。';

  @override
  String get errorNoAiCredential => 'このプロファイルの API キーを入力してください。';

  @override
  String get errorAiCredentialStore => 'API キーに安全にアクセスできません。';

  @override
  String get errorAiAuth => 'AI サービスが現在の認証情報を拒否しました。';

  @override
  String get aiSummary => 'AI 要約';

  @override
  String get summaryNoProfile => '要約を生成する前に、有効な AI プロファイルを設定してください。';

  @override
  String get configureAiProfile => 'AI プロファイルを設定';

  @override
  String get summaryTemplate => '要約テンプレート';

  @override
  String get summaryTemplateClassic => 'クラシック';

  @override
  String get summaryTemplateNews => 'ニュース 5W1H';

  @override
  String get summaryTemplateDecision => '読むか判断';

  @override
  String get summaryTemplateFunnel => '段階的要約';

  @override
  String get summaryTemplateArgument => '論点分析';

  @override
  String get summaryTemplateMinimal => '一文';

  @override
  String get summaryTemplateLegacy => '旧版';

  @override
  String summaryCacheInfo(String template, String language) {
    return 'キャッシュテンプレート：$template · 言語：$language';
  }

  @override
  String get regenerate => '再生成';

  @override
  String get stop => '停止';

  @override
  String get noSummaryYet => '完了した要約はまだありません。';

  @override
  String get askAboutSummary => 'この要約について質問';

  @override
  String get summaryQuestionHint => '上の要約だけを使って質問';

  @override
  String get send => '送信';
}
