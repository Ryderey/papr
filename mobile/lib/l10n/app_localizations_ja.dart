// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Japanese (`ja`).
class AppLocalizationsJa extends AppLocalizations {
  AppLocalizationsJa([String locale = 'ja']) : super(locale);

  @override
  String get githubScheduleUnsaved => '変更は保存後に適用されます。';

  @override
  String get githubScheduleSaved => '同期設定を保存しました';

  @override
  String get githubScheduleSaveFailed => '同期設定を保存できませんでした。再試行してください。';

  @override
  String get githubAutomatic => '自動同期';

  @override
  String get githubUploadDelay => 'ローカル変更の送信待機時間';

  @override
  String get githubCloudInterval => 'クラウド更新の確認間隔';

  @override
  String get githubScheduleHint => 'この端末のみの設定です。送信間隔は最低60秒です。';

  @override
  String get githubManualOnly => '自動同期は無効です。必要なときに今すぐ同期を実行してください。';

  @override
  String get errorGithubInvalidSchedule => '同期スケジュールの設定が無効です。';

  @override
  String get githubBackgroundInterval => 'バックグラウンド同期の間隔';

  @override
  String get githubBackgroundHint => 'Androidの省電力設定により実行が遅れる場合があります。';

  @override
  String githubSeconds(int count) {
    return '$count 秒';
  }

  @override
  String get githubScope =>
      '購読、フォルダー、記事のタイトルとリンク、既読・スター・後で読む状態を同期します。通常の記事は90日間、保存した記事は無期限に保持します。本文と設定は端末内に保持します。';

  @override
  String get githubLoading => '読み込み中…';

  @override
  String githubPending(int count) {
    return '未送信の操作：$count';
  }

  @override
  String githubLastSuccess(String time) {
    return '最終同期：$time';
  }

  @override
  String get githubNever => '未同期';

  @override
  String githubRejected(int count) {
    return '確認が必要な操作：$count';
  }

  @override
  String get githubUncertain => '前回の送信結果を確認できませんでした。再試行時にクラウドの受信状態を確認します。';

  @override
  String githubRetryAt(String time) {
    return '再試行可能時刻：$time';
  }

  @override
  String get githubCancel => 'キャンセル';

  @override
  String get githubToken => 'GitHubの細粒度トークン';

  @override
  String get githubUpdateToken => 'トークンを更新';

  @override
  String get githubCredentialUpdated => 'トークンを更新しました';

  @override
  String get githubCompleted => '同期が完了しました';

  @override
  String get githubOwner => 'リポジトリ所有者';

  @override
  String get githubRepo => 'プライベートリポジトリ名';

  @override
  String get githubBranch => 'ブランチ（空欄は既定）';

  @override
  String get githubOtherBackend => '既存の同期サービスを切断してからGitHubに接続してください。';

  @override
  String get githubSetup =>
      '初期化済みのプライベートリポジトリと、そのリポジトリのみのContents読み書き権限を持つ細粒度トークンを使用してください。';

  @override
  String get githubPreview => '接続をプレビュー';

  @override
  String githubPreviewCounts(
      int localFeeds, int remoteFeeds, int localArticles, int remoteArticles) {
    return '購読：端末 $localFeeds、クラウド $remoteFeeds。記事：端末 $localArticles、クラウド $remoteArticles。';
  }

  @override
  String githubPreviewExcluded(int feeds, int articles, int warnings) {
    return '除外：$feeds件のソース、$articles件の記事。注意：$warnings件。';
  }

  @override
  String get githubPreviewHint =>
      '初回同期では購読と保存状態を統合します。既存のクラウドの購読設定が優先され、本文は送信されません。';

  @override
  String get githubConfirm => '接続を確認';

  @override
  String get githubErrorUnknown =>
      'GitHub同期に失敗しました。端末の操作は保持されています。接続を確認して再試行してください。';

  @override
  String get errorGithubAuthenticationFailed => 'トークンが無効または期限切れです。更新してください。';

  @override
  String get errorGithubPermissionDenied => 'このリポジトリのContents読み書き権限が必要です。';

  @override
  String get errorGithubRateLimited => 'GitHubの制限に達しました。表示された時刻まで待ってください。';

  @override
  String get errorGithubNetwork => 'GitHubに接続できません。未送信の操作は保持されています。';

  @override
  String get errorGithubRepositoryUnavailable =>
      'リポジトリまたはブランチにアクセスできません。名前と権限を確認してください。';

  @override
  String get errorGithubPrivateRepositoryRequired => 'プライベートリポジトリを使用してください。';

  @override
  String get errorGithubOtherBackendConnected => '既存の同期サービスを切断してください。';

  @override
  String get errorGithubPreviewChanged => 'クラウドが更新されました。再度プレビューしてください。';

  @override
  String get errorGithubSyncBusy => '同期中です。';

  @override
  String get errorGithubHistoryRewritten => 'クラウドの履歴が変更されました。確認後に再接続してください。';

  @override
  String get errorGithubDatasetChanged => '同期データセットが変更されました。確認後に再接続してください。';

  @override
  String get errorGithubRestoredDatabase => '復元されたデータベースを検出しました。確認後に再接続してください。';

  @override
  String get errorGithubDatabaseCloneDetected =>
      '別のインストールのデータベースです。この端末で再接続してください。';

  @override
  String get errorGithubInstallationMissing => '端末の安全な識別情報がありません。再接続してください。';

  @override
  String get errorGithubCapacityExceeded => '同期データのサイズ制限を超えました。';

  @override
  String get errorGithubSyncCancelled => '同期をキャンセルしました。未送信の操作は保持されています。';

  @override
  String get errorGithubConcurrentRetryLimit => 'クラウドが繰り返し更新されました。後で再試行してください。';

  @override
  String get errorGithubWriteRejected => '書き込みが拒否されました。ブランチ保護と権限を確認してください。';

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
  String get autoRefresh => 'バックグラウンド更新';

  @override
  String get autoRefreshDescription => 'Android がバックグラウンド処理を許可したときに購読を更新します';

  @override
  String get newArticleNotifications => '新着記事の通知';

  @override
  String get newArticleNotificationsDescription => 'バックグラウンド更新後に要約通知を1件表示します';

  @override
  String newArticleNotificationBody(int count) {
    return '新着記事 $count 件';
  }

  @override
  String get notificationQuietHours => '夜間の通知停止';

  @override
  String get notificationQuietHoursDescription => '22:00 から 08:00 までは通知しません';

  @override
  String get notificationPermissionDenied => '通知権限が許可されていません。';

  @override
  String get resetPreferences => '設定をリセット';

  @override
  String get resetPreferencesTitle => '設定をリセットしますか？';

  @override
  String get resetPreferencesMessage =>
      'テーマ、言語、閲覧、バックグラウンドの設定を初期値に戻しますか？購読とアカウントは残ります。';

  @override
  String get resetPreferencesDone => '設定をリセットしました。';

  @override
  String get resetPreferencesFailed => '設定をリセットできませんでした。もう一度お試しください。';

  @override
  String get clearAllData => 'すべてのデータを消去';

  @override
  String get clearAllDataTitle => 'アプリの全データを消去しますか？';

  @override
  String get clearAllDataMessage =>
      'ローカルの購読、記事、設定、キャッシュ、保存済みの認証情報を削除しますか？アプリは終了し、元に戻せません。';

  @override
  String get clearAllDataFailed =>
      'アプリのデータを消去できませんでした。Android の設定からもう一度お試しください。';

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
  String get errorAiNoVisibleOutput => 'AI から表示可能な内容が返されませんでした。もう一度生成してください。';

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
  String get audioEpisodes => '音声エピソード';

  @override
  String get audioEpisode => 'Podcast 音声';

  @override
  String get playAudio => '再生';

  @override
  String get pauseAudio => '一時停止';

  @override
  String get nowPlaying => '再生中';

  @override
  String get noActivePlayback => '再生中の音声はありません。';

  @override
  String get rewind15 => '15 秒戻る';

  @override
  String get forward30 => '30 秒進む';

  @override
  String get playbackSpeed => '再生速度';

  @override
  String get stopPlayback => '再生を停止';

  @override
  String get errorInvalidPlaybackUrl => '音声のアドレスが無効です。';

  @override
  String get errorPlaybackNetwork => '音声を読み込めません。ネットワークを確認して再試行してください。';

  @override
  String get errorPlaybackUnavailable => 'この端末では音声を再生できません。';

  @override
  String get errorPlaybackFailed => '音声の再生に失敗しました。';

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

  @override
  String get aiTranslation => 'AI 翻訳';

  @override
  String get translationNoProfile => '翻訳する前に、有効な AI プロファイルを設定してください。';

  @override
  String get targetLanguage => '翻訳先の言語';

  @override
  String get translateArticle => '翻訳';

  @override
  String translationCacheInfo(String language) {
    return 'キャッシュ済み翻訳：$language';
  }

  @override
  String get translationPreparing => '翻訳を準備しています…';

  @override
  String translationProgress(int completed, int total) {
    return '$completed/$total セクションを翻訳済み';
  }

  @override
  String get noTranslationYet => '完了した翻訳はまだありません。';

  @override
  String get syncTitle => 'リーダー同期';

  @override
  String get syncProvider => 'サービス';

  @override
  String get syncServerUrl => 'サーバー URL';

  @override
  String get syncUsername => 'ユーザー名';

  @override
  String get syncCredential => 'GReader パスワード';

  @override
  String get syncFieldRequired => '入力してください。';

  @override
  String get syncConnect => '接続';

  @override
  String get syncConnected => '接続済み';

  @override
  String get syncNotConnected => 'リーダーサービスに接続していません。';

  @override
  String get syncTestSucceeded => '接続テストに成功しました。';

  @override
  String get syncNow => '今すぐ同期';

  @override
  String get syncCompleted => '同期が完了しました。';

  @override
  String get syncDisconnect => '接続を解除';

  @override
  String get syncDisconnectConfirm => '接続を解除し、保存済みの認証情報を削除しますか？';

  @override
  String get syncReplaceConnection => '接続を変更';

  @override
  String get syncNeverCompleted => '同期履歴なし';

  @override
  String syncLastSuccess(String time) {
    return '前回の同期：$time';
  }

  @override
  String get errorInvalidSyncProfile => 'サーバー URL、ユーザー名、パスワードを確認してください。';

  @override
  String get errorSyncCredentialMissing => '保存済みのパスワードがありません。再接続してください。';

  @override
  String get errorSyncNotConnected => '先にリーダーサービスへ接続してください。';

  @override
  String get errorSyncAuthFailed => '認証に失敗しました。GReader パスワードを確認してください。';

  @override
  String get errorSyncUnavailable => 'リーダーサービスを利用できません。後でもう一度お試しください。';

  @override
  String get errorSyncProviderFailed => 'リーダーサービスの処理に失敗しました。もう一度お試しください。';

  @override
  String get errorSyncInvalidResponse => 'リーダーサービスから予期しない応答が返されました。';

  @override
  String get errorSyncTooManyItems => '一度に同期する項目が多すぎます。履歴を減らして再試行してください。';

  @override
  String get errorSyncCredentialStore => '保存済みパスワードに安全にアクセスできません。';
}
