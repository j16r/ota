pub mod local;
pub mod rdb;

use anyhow::Result;
use thiserror::Error;

use crate::articles::Article;
use crate::query::Query;

pub trait Entry {
    fn article(&self) -> Article;
    fn body(&self) -> Result<Box<dyn std::io::Read>>;
}

pub trait Index: Send + Sync {
    fn update(&mut self, article: &Article) -> Result<Box<dyn Entry>>;
    fn search(&mut self, query: &Query) -> Result<Box<dyn Iterator<Item = Box<dyn Entry>>>>;

    fn first(&mut self, query: &Query) -> Result<Box<dyn Entry>> {
        self.search(query)?
            .next()
            .ok_or_else(|| Error::ArticleNotFound.into())
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("no articles found that match that query")]
    ArticleNotFound,
    #[error("internal server error")]
    InternalError,
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::index::local::Local;
    use crate::index::*;
    use crate::NewArticleRequest;
    use tempdir::TempDir;

    #[test]
    fn test_index() {
        let dir = TempDir::new("index_test").unwrap();
        let mut index = Local::new(dir.path()).unwrap();

        let article = Article::new(&NewArticleRequest {
            id: Some("main".to_string()),
            ..Default::default()
        });

        index.update(&article).unwrap();

        let result: Vec<Article> = index
            .search(&"@main".try_into().unwrap())
            .unwrap()
            .map(|e| e.article())
            .collect();
        assert!(result.len() == 1);
        assert_eq!(result[0].id, Some("main".to_string()));
    }
}
