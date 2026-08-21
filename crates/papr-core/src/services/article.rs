use std::sync::Arc;

use crate::db::Db;
use crate::dto::{ArticleCounts, ArticleDetail, ArticleFilter, ArticleSummary, TagSummary};
use crate::error::{CoreError, ErrorCategory};
use crate::{extraction, ingestion};

pub struct ArticleService {
    db: Arc<Db>,
    http: Arc<reqwest::Client>,
}

impl ArticleService {
    pub fn new(db: Arc<Db>, http: Arc<reqwest::Client>) -> Self {
        Self { db, http }
    }

    pub async fn list_articles(
        &self,
        filter: ArticleFilter,
    ) -> Result<Vec<ArticleSummary>, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.list_articles(&filter))
            .await
            .map_err(blocking_error)?
    }

    pub async fn count_articles(&self, filter: ArticleFilter) -> Result<i64, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.count_articles(&filter))
            .await
            .map_err(blocking_error)?
    }

    pub async fn article_counts(&self) -> Result<ArticleCounts, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.article_counts())
            .await
            .map_err(blocking_error)?
    }

    pub async fn list_tags(&self) -> Result<Vec<TagSummary>, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.list_tag_summaries())
            .await
            .map_err(blocking_error)?
    }

    pub async fn get_article_detail(&self, article_id: i64) -> Result<ArticleDetail, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.get_article_detail(article_id))
            .await
            .map_err(blocking_error)?
    }

    pub async fn set_read(&self, article_id: i64, value: bool) -> Result<(), CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.set_article_read(article_id, value))
            .await
            .map_err(blocking_error)?
    }

    pub async fn set_starred(&self, article_id: i64, value: bool) -> Result<(), CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.set_article_starred(article_id, value))
            .await
            .map_err(blocking_error)?
    }

    pub async fn set_read_later(&self, article_id: i64, value: bool) -> Result<(), CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.set_article_read_later(article_id, value))
            .await
            .map_err(blocking_error)?
    }

    pub async fn mark_all_read(&self, filter: ArticleFilter) -> Result<i64, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.mark_all_read(&filter))
            .await
            .map_err(blocking_error)?
    }

    pub async fn extract_fulltext(&self, article_id: i64) -> Result<String, CoreError> {
        let detail = self.get_article_detail(article_id).await?;
        let url = detail.url.ok_or_else(|| {
            CoreError::coded(
                ErrorCategory::InvalidInput,
                "articleUrlMissing",
                Some(article_id.to_string()),
            )
        })?;
        let (bytes, content_type, final_url) = ingestion::fetch::get(&self.http, &url).await?;
        let page_html = ingestion::fetch::decode_html(&bytes, content_type.as_deref());
        let lead_image = extraction::lead_image(&page_html, &final_url);
        let extraction_url = final_url.clone();
        let extracted = tokio::task::spawn_blocking(move || {
            extraction::extract_article(&page_html, &extraction_url)
        })
        .await
        .map_err(blocking_error)??;

        let db = Arc::clone(&self.db);
        let stored_html = extracted.clone();
        tokio::task::spawn_blocking(move || {
            db.set_extracted_html(article_id, &stored_html, lead_image.as_deref())
        })
        .await
        .map_err(blocking_error)??;
        Ok(extracted)
    }
}

fn blocking_error(error: tokio::task::JoinError) -> CoreError {
    CoreError::Platform(format!("blocking task failed: {error}"))
}
