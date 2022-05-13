use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::articles::Article;
use crate::index::{Entry, Index};
use crate::query::Query;

// auto migrate
// create table article (
// id : unique primary key,
// tags:
//
//
pub struct RelationalDB {}

impl RelationalDB {
    pub fn new<T: Into<PathBuf> + AsRef<Path>>(_path: T) -> Result<Self> {
        Ok(RelationalDB {})
    }
}

pub struct RelationalIndexIterator {
    results: Vec<RelationalEntry>,
}

impl Iterator for RelationalIndexIterator {
    type Item = Box<dyn Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

struct RelationalEntry {
    article: Article,
}

impl Entry for RelationalEntry {
    fn article(&self) -> Article {
        self.article.clone()
    }

    fn body(&self) -> Result<Box<dyn std::io::Read>> {
        unimplemented!();
    }
}

impl Index for RelationalDB {
    fn update(&mut self, article: &Article) -> Result<Box<dyn Entry>> {
        // let mut file = File::create(article.path)?;
        // file.write_all(article.body.as_bytes())?;
        Ok(Box::new(RelationalEntry {
            article: article.clone(),
        }))
        // id: article.id.clone(),
        // properties: PropertySet::new(),
        // tags: HashSet::new(),
        // // path: self.path.clone(),
        // })
    }

    fn search(&mut self, _query: &Query) -> Result<Box<dyn Iterator<Item = Box<dyn Entry>>>> {
        Ok(Box::new(RelationalIndexIterator { results: vec![] }))
    }
}
