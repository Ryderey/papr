use std::sync::Arc;

use crate::db::Db;
use crate::dto::{ArticleDetail, ArticleFilter, ArticleSummary};
use crate::error::CoreError;

pub struct ArticleService {
    db: Arc<Db>,
}

impl ArticleService {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub async fn list_articles(
        &self,
        filter: ArticleFilter,
    ) -> Result<Vec<ArticleSummary>, CoreError> {
        // Phase 1: synchronous SQLite call. `async` signature reserves room for
        // async I/O or a connection pool later.
        tokio::task::block_in_place(|| self.db.list_articles(&filter))
    }

    pub async fn get_article_detail(
        &self,
        article_id: i64,
    ) -> Result<ArticleDetail, CoreError> {
        tokio::task::block_in_place(|| self.db.get_article_detail(article_id))
    }
}
