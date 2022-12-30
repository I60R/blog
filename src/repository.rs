use moka::future as moka;
use std::sync::Arc;

use crate::{database, article::{self, ListItem}};


#[derive(Clone)]
pub struct ArticlesRepository {
    db: database::Database,
    articles_cache: moka::Cache<u32, Arc<article::ListItem>>,
    article_cache: moka::Cache<String, Arc<article::Item>>,
}

impl ArticlesRepository {
    pub fn new(db: database::Database) -> ArticlesRepository {
        ArticlesRepository {
            db,
            articles_cache: moka::CacheBuilder::new(64)
                .time_to_live(std::time::Duration::from_secs(5 * 60))
                .build(),
            article_cache: moka::CacheBuilder::new(64)
                .time_to_live(std::time::Duration::from_secs(5 * 60))
                .build(),
        }
    }


    pub async fn fetch_articles(&mut self) -> Vec<Arc<ListItem>> {
        if self.articles_cache.entry_count() == 0 {
            let articles = self.db.fetch_articles().await;
            for a in articles {
                self.articles_cache.insert(a.id, Arc::new(a)).await;
            }
        }
        self.articles_cache.iter()
            .map(|(_k, v)| v)
            .collect()
    }

    pub async fn fetch_article(&mut self, title: &str) -> Option<Arc<article::Item>> {
        if let Some(article) = self.article_cache.get(title) {
            return Some(article)
        }
        if let Some(article) = self.db.fetch_article(title).await {
            self.article_cache.insert(title.to_string(), Arc::new(article)).await;
            self.article_cache.get(title)
        } else {
            None
        }
    }


    pub async fn fetch_next_article_title_after_id(&mut self, id: u32) -> String {
        if let Some(article) = self.articles_cache.get(&(id + 1)) {
            return article.title.clone()
        }
        self.db.fetch_next_article_title_after_id(id).await
    }

    pub async fn fetch_prev_article_title_before_id(&self, id: u32) -> String {
        if let Some(article) = self.articles_cache.get(&(id - 1)) {
            return article.title.clone()
        }
        self.db.fetch_prev_article_title_before_id(id).await
    }


    pub async fn create_article(&mut self, title: &str, body: &str) -> bool {
        self.articles_cache.invalidate_all();
        self.article_cache.invalidate(title).await;
        self.db.create_article(title, body).await
    }

    pub async fn delete_article(&mut self, title: &str) -> bool {
        self.articles_cache.invalidate_all();
        self.article_cache.invalidate(title).await;
        self.db.delete_article(title).await
    }
}