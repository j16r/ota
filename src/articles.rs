use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::prelude::*;
use handlebars::RenderError;
use regex::Regex;
use rocket::form::FromForm;
use rusty_ulid::Ulid;
use serde_derive::{Deserialize, Serialize};

use crate::index::{self, Index};
use crate::query::Query;

pub type PropertySet = HashMap<String, String>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Article {
    pub key: Ulid,
    pub id: String,
    pub title: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub body: String,
    pub properties: PropertySet,
    pub tags: HashSet<String>,
}

impl Article {
    // TODO: NewArticleRequest is untrusted so should be validated / sanitized
    pub fn new(request: &NewArticleRequest) -> Article {
        let mut article = Article {
            key: Ulid::generate(),
            // TODO: ID should be optionally specified by user, or generated if not as a slug
            id: request.id.clone(), //slug_from_title(&request.title).to_string(),
            title: request.title.clone(),
            body: request.body.clone(),
            properties: PropertySet::new(),
            tags: HashSet::new(),
        };
        for property in request.properties.split_whitespace() {
            let (key, value) = property.split_once(':').unwrap();
            article.properties.insert(key.into(), value.into());
        }
        Self::add_default_properties(&mut article.properties);
        for tag in request.tags.split_whitespace() {
            article.tags.insert(tag.into());
        }
        article
    }

    fn add_default_properties(properties: &mut PropertySet) {
        let now: DateTime<Utc> = Utc::now();
        properties.insert("timestamp".to_string(), now.to_string());
        properties.insert("epoch".to_string(), now.timestamp().to_string());
        properties.insert("year".to_string(), now.format("%Y").to_string());
        properties.insert("month".to_string(), now.format("%m").to_string());
        properties.insert("day".to_string(), now.format("%d").to_string());
    }

    pub fn epoch(&self) -> i64 {
        self.properties["epoch"].parse().unwrap()
    }

    pub fn timestamp(&self) -> String {
        self.properties["timestamp"].clone()
    }
}

fn slug_from_title(title: &str) -> Cow<str> {
    let scrubbing_regex = Regex::new(r"[^A-Za-z0-9]+").unwrap();
    scrubbing_regex.replace_all(title, "_")
}

#[derive(Serialize, Default, Deserialize, FromForm, Debug)]
pub struct NewArticleRequest {
    pub id: String,
    pub title: String,
    pub body: String,
    pub properties: String,
    pub tags: String,
}

pub fn lookup_article(
    index: &mut Box<dyn Index>,
    query_str: &str,
) -> Result<Box<dyn std::io::Read>> {
    println!("lookup_article(query_str: {:?})", query_str);
    let query: Query = query_str.try_into()?;

    match index.first(&query) {
        Ok(r) => r.body(),
        Err(e) => match e.downcast_ref::<index::Error>() {
            Some(index::Error::ArticleNotFound) => {
                println!("failed to find article, trying fallback...");
                File::open(load_fallback(&query)?)
                    .map(|f| Box::new(f) as Box<dyn std::io::Read>)
                    .map_err(|_| RenderError::new("error finding fallback article").into())
            }
            _ => {
                eprintln!("error from index {:?}", &e);
                Err(e)
            }
        },
    }
}

fn load_fallback(query: &Query) -> Result<PathBuf> {
    if let Some(ref id) = query.id {
        println!("query#id = {:?}", id);
        return Ok(Path::new("templates/").join(format!("{}.html.hbs", id)));
    }
    Err(index::Error::ArticleNotFound.into())
}

// pub fn lookup_articles(
//     index: &mut Box<dyn Index>,
//     query_str: &str) -> std::io::Result<Vec<File>> {

//     println!("lookup_articles(query_str: {:?})", query_str);
//     let query: Query = query_str.try_into()?;

//     for article in index.search(&query) {
//     }
//     Ok(vec![])
//     // let path = Path::new("data/articles/").join(query);

//     // Ok(vec![File::open(path).or_else(|_| {
//     // let fallback_path = Path::new("templates/").join(format!("{}.hbs", query));
//     // File::open(fallback_path)
//     // })])
// }

#[cfg(test)]
mod tests {
    use crate::articles::*;

    #[test]
    fn test_new() {
        Article::new(&NewArticleRequest {
            ..Default::default()
        });
    }

    #[test]
    fn test_slug() {
        assert_eq!(slug_from_title("abcd"), "abcd");
        assert_eq!(slug_from_title("a!@#bcd"), "a_bcd");
        assert_eq!(slug_from_title("number 10"), "number_10");
        assert_eq!(slug_from_title(""), "");
    }
}
