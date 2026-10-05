use std::sync::Arc;

use crate::ai::{
    build_follow_up_system_prompt, build_follow_up_user_message, build_summary_prompt,
    build_summary_user_message, stream_chat, stream_chat_inner, AiCancellation, AiProfile,
    AiStreamEvent, ResolvedAiCredential,
};
use crate::db::Db;
use crate::dto::{AiSummaryCache, SummaryTemplate};
use crate::error::CoreError;
use crate::translate;

/// Coordinates article reads, provider streaming, and success-only cache writes.
pub struct AiService {
    db: Arc<Db>,
    http: Arc<reqwest::Client>,
}

impl AiService {
    pub fn new(db: Arc<Db>, http: Arc<reqwest::Client>) -> Self {
        Self { db, http }
    }

    pub async fn summary_cache(
        &self,
        article_id: i64,
    ) -> Result<Option<AiSummaryCache>, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.get_ai_summary_cache(article_id))
            .await
            .map_err(blocking_error)?
    }

    /// Verify a provider with the exact streaming request path used by summaries.
    /// It deliberately has no article input and never writes a cache entry.
    pub async fn test_connection(
        &self,
        profile: &AiProfile,
        credential: Option<&ResolvedAiCredential>,
    ) -> Result<(), CoreError> {
        let cancellation = AiCancellation::default();
        stream_chat(
            &self.http,
            profile,
            credential,
            "connection-test",
            "You are testing an AI provider connection. Reply with exactly: OK",
            "Reply with exactly: OK",
            &cancellation,
            |_| true,
        )
        .await
        .map(|_| ())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn summarize<F>(
        &self,
        article_id: i64,
        profile: &AiProfile,
        credential: Option<&ResolvedAiCredential>,
        template: SummaryTemplate,
        language: &str,
        request_id: &str,
        cancellation: &AiCancellation,
        mut emit: F,
    ) -> Result<String, CoreError>
    where
        F: FnMut(AiStreamEvent) -> bool,
    {
        let db = Arc::clone(&self.db);
        let (title, body) = match tokio::task::spawn_blocking(move || db.article_text(article_id))
            .await
            .map_err(blocking_error)
            .and_then(|result| result)
        {
            Ok(article) => article,
            Err(error) => {
                emit(AiStreamEvent::Error {
                    request_id: request_id.to_string(),
                    code: error.code().to_string(),
                });
                return Err(error);
            }
        };
        let system = build_summary_prompt(template, language);
        let user = match build_summary_user_message(&title, &body) {
            Ok(user) => user,
            Err(error) => {
                emit(AiStreamEvent::Error {
                    request_id: request_id.to_string(),
                    code: error.code().to_string(),
                });
                return Err(error);
            }
        };

        let result = stream_chat_inner(
            &self.http,
            profile,
            credential,
            request_id,
            &system,
            &user,
            cancellation,
            &mut emit,
        )
        .await;

        match result {
            Ok(text) => {
                let db = Arc::clone(&self.db);
                let cache_text = text.clone();
                let language = language.to_string();
                let cache_result = tokio::task::spawn_blocking(move || {
                    db.set_ai_summary_cache(article_id, &cache_text, template, &language)
                })
                .await
                .map_err(blocking_error)
                .and_then(|result| result);
                if let Err(error) = cache_result {
                    emit(AiStreamEvent::Error {
                        request_id: request_id.to_string(),
                        code: error.code().to_string(),
                    });
                    return Err(error);
                }
                emit(AiStreamEvent::Completed {
                    request_id: request_id.to_string(),
                });
                Ok(text)
            }
            Err(error) => {
                emit(AiStreamEvent::Error {
                    request_id: request_id.to_string(),
                    code: error.code().to_string(),
                });
                Err(error)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn follow_up<F>(
        &self,
        profile: &AiProfile,
        credential: Option<&ResolvedAiCredential>,
        summary: &str,
        history: &[(String, String)],
        question: &str,
        language: &str,
        request_id: &str,
        cancellation: &AiCancellation,
        mut emit: F,
    ) -> Result<String, CoreError>
    where
        F: FnMut(AiStreamEvent) -> bool,
    {
        let system = build_follow_up_system_prompt(language);
        let user = match build_follow_up_user_message(summary, history, question) {
            Ok(user) => user,
            Err(error) => {
                emit(AiStreamEvent::Error {
                    request_id: request_id.to_string(),
                    code: error.code().to_string(),
                });
                return Err(error);
            }
        };
        stream_chat(
            &self.http,
            profile,
            credential,
            request_id,
            &system,
            &user,
            cancellation,
            emit,
        )
        .await
    }

    /// Translate article HTML through the configured LLM, reporting only
    /// batch-level progress and caching the result after full success.
    #[allow(clippy::too_many_arguments)]
    pub async fn translate_with_profile<F>(
        &self,
        article_id: i64,
        profile: &AiProfile,
        credential: Option<&ResolvedAiCredential>,
        language: &str,
        request_id: &str,
        cancellation: &AiCancellation,
        mut emit: F,
    ) -> Result<String, CoreError>
    where
        F: FnMut(AiStreamEvent) -> bool,
    {
        let db = Arc::clone(&self.db);
        let html = match tokio::task::spawn_blocking(move || db.article_html(article_id))
            .await
            .map_err(blocking_error)
            .and_then(|result| result)
        {
            Ok(html) => html,
            Err(error) => {
                emit(AiStreamEvent::Error {
                    request_id: request_id.to_string(),
                    code: error.code().to_string(),
                });
                return Err(error);
            }
        };
        let batches = translate::chunk_blocks(&html, translate::LLM_CHUNK_BUDGET);
        if batches.is_empty() {
            let error = CoreError::coded(
                crate::error::ErrorCategory::InvalidInput,
                "noArticleBody",
                None,
            );
            emit(AiStreamEvent::Error {
                request_id: request_id.to_string(),
                code: error.code().to_string(),
            });
            return Err(error);
        }
        let total = batches.len() as u32;
        if !emit(AiStreamEvent::Progress {
            request_id: request_id.to_string(),
            completed: 0,
            total,
        }) {
            return Err(CoreError::coded(
                crate::error::ErrorCategory::Ai,
                "aiCancelled",
                None,
            ));
        }

        let system = translate::system_prompt(translate::language_name(language));
        let mut translated = String::new();
        for (index, batch) in batches.iter().enumerate() {
            if cancellation.is_cancelled() {
                let error = CoreError::coded(crate::error::ErrorCategory::Ai, "aiCancelled", None);
                emit(AiStreamEvent::Error {
                    request_id: request_id.to_string(),
                    code: error.code().to_string(),
                });
                return Err(error);
            }
            let mut discard_deltas = |_| true;
            let raw = match stream_chat_inner(
                &self.http,
                profile,
                credential,
                request_id,
                &system,
                batch,
                cancellation,
                &mut discard_deltas,
            )
            .await
            {
                Ok(text) => text,
                Err(error) => {
                    emit(AiStreamEvent::Error {
                        request_id: request_id.to_string(),
                        code: error.code().to_string(),
                    });
                    return Err(error);
                }
            };
            translated.push_str(&crate::ingestion::sanitize::sanitize(
                &translate::strip_code_fence(&raw),
                None,
            ));
            if !emit(AiStreamEvent::Progress {
                request_id: request_id.to_string(),
                completed: index as u32 + 1,
                total,
            }) {
                return Err(CoreError::coded(
                    crate::error::ErrorCategory::Ai,
                    "aiCancelled",
                    None,
                ));
            }
        }

        let db = Arc::clone(&self.db);
        let cached = translated.clone();
        let language = language.to_string();
        let cache_result = tokio::task::spawn_blocking(move || {
            db.set_translation_cache(article_id, &cached, &language)
        })
        .await
        .map_err(blocking_error)
        .and_then(|result| result);
        if let Err(error) = cache_result {
            emit(AiStreamEvent::Error {
                request_id: request_id.to_string(),
                code: error.code().to_string(),
            });
            return Err(error);
        }
        emit(AiStreamEvent::Completed {
            request_id: request_id.to_string(),
        });
        Ok(translated)
    }
}

fn blocking_error(error: tokio::task::JoinError) -> CoreError {
    CoreError::Platform(format!("blocking task failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use crate::ai::{AiAuthMode, AiProtocol, AiPurpose};
    use crate::dto::{ArticleFilter, NewArticle};

    async fn serve_once(status: &str, body: &str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let status = status.to_string();
        let body = body.to_string();
        let (ready, started) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let _ = ready.send(());
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 4096];
            let _ = socket.read(&mut request).await;
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        });
        started.await.unwrap();
        format!("http://{address}/v1")
    }

    async fn serve_stalled_once() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (ready, started) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let _ = ready.send(());
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 4096];
            let _ = socket.read(&mut request).await;
            tokio::time::sleep(Duration::from_secs(5)).await;
        });
        started.await.unwrap();
        format!("http://{address}/v1")
    }

    fn profile(base_url: String) -> AiProfile {
        AiProfile {
            id: "test".into(),
            name: "Test".into(),
            protocol: AiProtocol::OpenaiChatCompletions,
            model: "test-model".into(),
            base_url,
            auth: AiAuthMode::None,
            headers: BTreeMap::new(),
            credential_ref: None,
            enabled: true,
            default_for: vec![AiPurpose::Summary],
        }
    }

    fn setup_db() -> (tempfile::TempDir, Arc<Db>, i64) {
        let temp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&temp.path().join("test.db")).unwrap());
        let feed_id = db.add_feed("https://example.com/feed.xml").unwrap();
        db.upsert_article(
            feed_id,
            &NewArticle {
                guid: "article".into(),
                url: Some("https://example.com/article".into()),
                title: "Article".into(),
                author: None,
                summary: None,
                content_html: Some("<p>Article body</p>".into()),
                body_text: "Article body".into(),
                image_url: None,
                published_at: None,
                enclosures: Vec::new(),
            },
        )
        .unwrap();
        let article_id = db.list_articles(&ArticleFilter::default()).unwrap()[0].id;
        (temp, db, article_id)
    }

    #[tokio::test]
    async fn connection_test_uses_the_summary_stream_path_without_writing_cache() {
        let (_temp, db, article_id) = setup_db();
        let body = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"OK\"}}]}\n",
            "data: [DONE]\n"
        );
        let profile = profile(serve_once("200 OK", body).await);
        let service = AiService::new(Arc::clone(&db), Arc::new(reqwest::Client::new()));

        service.test_connection(&profile, None).await.unwrap();

        assert_eq!(db.get_ai_summary_cache(article_id).unwrap(), None);
    }

    #[tokio::test]
    async fn summarize_caches_only_a_fully_successful_stream() {
        let (_temp, db, article_id) = setup_db();
        let body = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"hello \"}}]}\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"world\"}}]}\n",
            "data: [DONE]\n"
        );
        let profile = profile(serve_once("200 OK", body).await);
        let service = AiService::new(Arc::clone(&db), Arc::new(reqwest::Client::new()));
        let mut events = Vec::new();
        let text = service
            .summarize(
                article_id,
                &profile,
                None,
                SummaryTemplate::Classic,
                "zh-CN",
                "summary-1",
                &AiCancellation::default(),
                |event| {
                    events.push(event);
                    true
                },
            )
            .await
            .unwrap();

        assert_eq!(text, "hello world");
        assert!(matches!(
            events.last(),
            Some(AiStreamEvent::Completed { .. })
        ));
        assert_eq!(
            db.get_ai_summary_cache(article_id).unwrap(),
            Some(AiSummaryCache {
                summary: "hello world".into(),
                template: Some("classic".into()),
                language: Some("zh".into()),
            })
        );
    }

    #[tokio::test]
    async fn reasoning_only_stream_reports_no_visible_output() {
        let (_temp, db, article_id) = setup_db();
        let body = concat!(
            "data: {\"choices\":[{\"delta\":{\"reasoning\":\"thinking\"}}]}\n",
            "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"length\"}]}\n",
            "data: [DONE]\n"
        );
        let profile = profile(serve_once("200 OK", body).await);
        let service = AiService::new(Arc::clone(&db), Arc::new(reqwest::Client::new()));
        let mut events = Vec::new();

        let error = service
            .summarize(
                article_id,
                &profile,
                None,
                SummaryTemplate::Classic,
                "zh",
                "reasoning-only",
                &AiCancellation::default(),
                |event| {
                    events.push(event);
                    true
                },
            )
            .await
            .unwrap_err();

        assert_eq!(error.code(), "aiNoVisibleOutput");
        assert_eq!(
            events,
            [AiStreamEvent::Error {
                request_id: "reasoning-only".into(),
                code: "aiNoVisibleOutput".into(),
            }]
        );
        assert_eq!(db.get_ai_summary_cache(article_id).unwrap(), None);
    }

    #[tokio::test]
    async fn summary_preflight_failure_emits_a_terminal_error_event() {
        let (_temp, db, _) = setup_db();
        let service = AiService::new(Arc::clone(&db), Arc::new(reqwest::Client::new()));
        let mut events = Vec::new();

        let error = service
            .summarize(
                999_999,
                &profile("https://example.invalid/v1".into()),
                None,
                SummaryTemplate::Classic,
                "zh",
                "summary-missing",
                &AiCancellation::default(),
                |event| {
                    events.push(event);
                    true
                },
            )
            .await
            .unwrap_err();

        assert_eq!(error.code(), "articleNotFound");
        assert_eq!(
            events,
            [AiStreamEvent::Error {
                request_id: "summary-missing".into(),
                code: "articleNotFound".into(),
            }]
        );
    }

    #[tokio::test]
    async fn successful_regeneration_replaces_the_complete_cache_once() {
        let (_temp, db, article_id) = setup_db();
        let service = AiService::new(Arc::clone(&db), Arc::new(reqwest::Client::new()));
        let first_profile = profile(
            serve_once(
                "200 OK",
                "data: {\"choices\":[{\"delta\":{\"content\":\"first\"}}]}\n",
            )
            .await,
        );
        service
            .summarize(
                article_id,
                &first_profile,
                None,
                SummaryTemplate::Minimal,
                "en",
                "summary-first",
                &AiCancellation::default(),
                |_| true,
            )
            .await
            .unwrap();

        let second_profile = profile(
            serve_once(
                "200 OK",
                "data: {\"choices\":[{\"delta\":{\"content\":\"second\"}}]}\n",
            )
            .await,
        );
        service
            .summarize(
                article_id,
                &second_profile,
                None,
                SummaryTemplate::Classic,
                "zh-CN",
                "summary-second",
                &AiCancellation::default(),
                |_| true,
            )
            .await
            .unwrap();

        assert_eq!(
            db.get_ai_summary_cache(article_id).unwrap(),
            Some(AiSummaryCache {
                summary: "second".into(),
                template: Some("classic".into()),
                language: Some("zh".into()),
            })
        );
    }

    #[tokio::test]
    async fn mid_stream_error_keeps_the_previous_complete_cache() {
        let (_temp, db, article_id) = setup_db();
        db.set_ai_summary_cache(article_id, "old", SummaryTemplate::Minimal, "en")
            .unwrap();
        let body = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n",
            "data: {\"error\":{\"type\":\"rate_limit_error\",\"message\":\"private\"}}\n"
        );
        let profile = profile(serve_once("200 OK", body).await);
        let service = AiService::new(Arc::clone(&db), Arc::new(reqwest::Client::new()));
        let mut events = Vec::new();
        let error = service
            .summarize(
                article_id,
                &profile,
                None,
                SummaryTemplate::Classic,
                "zh",
                "summary-2",
                &AiCancellation::default(),
                |event| {
                    events.push(event);
                    true
                },
            )
            .await
            .unwrap_err();

        assert_eq!(error.code(), "aiRateLimited");
        assert!(matches!(
            events.last(),
            Some(AiStreamEvent::Error { code, .. }) if code == "aiRateLimited"
        ));
        assert_eq!(
            db.get_ai_summary_cache(article_id).unwrap(),
            Some(AiSummaryCache {
                summary: "old".into(),
                template: Some("minimal".into()),
                language: Some("en".into()),
            })
        );
    }

    #[tokio::test]
    async fn cancellation_wakes_a_request_waiting_for_the_provider() {
        let (_temp, db, article_id) = setup_db();
        let profile = profile(serve_stalled_once().await);
        let service = AiService::new(Arc::clone(&db), Arc::new(reqwest::Client::new()));
        let cancellation = AiCancellation::default();
        let cancel = cancellation.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(25)).await;
            cancel.cancel();
        });

        let result = tokio::time::timeout(
            Duration::from_secs(1),
            service.summarize(
                article_id,
                &profile,
                None,
                SummaryTemplate::Classic,
                "en",
                "summary-cancel",
                &cancellation,
                |_| true,
            ),
        )
        .await
        .expect("cancellation should wake the stalled request")
        .unwrap_err();
        assert_eq!(result.code(), "aiCancelled");
        assert_eq!(db.get_ai_summary_cache(article_id).unwrap(), None);
    }

    #[tokio::test]
    async fn translation_emits_only_batch_progress_and_caches_complete_html() {
        let (_temp, db, article_id) = setup_db();
        let profile = profile(
            serve_once(
                "200 OK",
                concat!(
                    "data: {\"choices\":[{\"delta\":{\"content\":\"<p>译文</p>\"}}]}\n",
                    "data: [DONE]\n"
                ),
            )
            .await,
        );
        let service = AiService::new(Arc::clone(&db), Arc::new(reqwest::Client::new()));
        let mut events = Vec::new();

        let translated = service
            .translate_with_profile(
                article_id,
                &profile,
                None,
                "zh-CN",
                "translate-1",
                &AiCancellation::default(),
                |event| {
                    events.push(event);
                    true
                },
            )
            .await
            .unwrap();

        assert_eq!(translated, "<p>译文</p>");
        assert!(events
            .iter()
            .all(|event| !matches!(event, AiStreamEvent::Delta { .. })));
        assert!(matches!(
            events.first(),
            Some(AiStreamEvent::Progress {
                completed: 0,
                total: 1,
                ..
            })
        ));
        assert!(matches!(
            events.last(),
            Some(AiStreamEvent::Completed { .. })
        ));
        let detail = db.get_article_detail(article_id).unwrap();
        assert_eq!(detail.translated_html.as_deref(), Some("<p>译文</p>"));
        assert_eq!(detail.translated_lang.as_deref(), Some("zh"));
    }

    #[tokio::test]
    async fn failed_translation_keeps_the_previous_complete_cache() {
        let (_temp, db, article_id) = setup_db();
        db.set_translation_cache(article_id, "<p>old</p>", "ja")
            .unwrap();
        let profile = profile(serve_once("500 Internal Server Error", "failure").await);
        let service = AiService::new(Arc::clone(&db), Arc::new(reqwest::Client::new()));

        let error = service
            .translate_with_profile(
                article_id,
                &profile,
                None,
                "zh",
                "translate-failure",
                &AiCancellation::default(),
                |_| true,
            )
            .await
            .unwrap_err();

        assert_eq!(error.code(), "aiNetwork");
        let detail = db.get_article_detail(article_id).unwrap();
        assert_eq!(detail.translated_html.as_deref(), Some("<p>old</p>"));
        assert_eq!(detail.translated_lang.as_deref(), Some("ja"));
    }

    #[tokio::test]
    async fn cancelled_translation_keeps_the_previous_complete_cache() {
        let (_temp, db, article_id) = setup_db();
        db.set_translation_cache(article_id, "<p>old</p>", "ja")
            .unwrap();
        let service = AiService::new(Arc::clone(&db), Arc::new(reqwest::Client::new()));
        let cancellation = AiCancellation::default();
        cancellation.cancel();
        let mut events = Vec::new();

        let error = service
            .translate_with_profile(
                article_id,
                &profile("http://127.0.0.1:1/v1".into()),
                None,
                "zh",
                "translate-cancelled",
                &cancellation,
                |event| {
                    events.push(event);
                    true
                },
            )
            .await
            .unwrap_err();

        assert_eq!(error.code(), "aiCancelled");
        assert!(matches!(
            events.last(),
            Some(AiStreamEvent::Error { code, .. }) if code == "aiCancelled"
        ));
        let detail = db.get_article_detail(article_id).unwrap();
        assert_eq!(detail.translated_html.as_deref(), Some("<p>old</p>"));
        assert_eq!(detail.translated_lang.as_deref(), Some("ja"));
    }
}
