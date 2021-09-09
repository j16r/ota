use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::{self, ErrorKind};
use std::iter;
use std::path::{Path, PathBuf};

use crate::index::{find_first_matching_path, update_index};
use crate::query::Query;
use chrono::prelude::*;
use rand::Rng;
use regex::Regex;
use rocket::form::FromForm;
use serde_derive::{Deserialize, Serialize};

type PropertySet = HashMap<String, String>;

#[derive(Serialize, Default, Debug)]
pub struct Article {
    pub id: Option<String>,
    name: String,
    title: String,
    body: String,
    properties: PropertySet,
    tags: HashSet<String>,
}

fn random_string() -> String {
    let mut rng = rand::thread_rng();
    iter::repeat(16)
        .map(|_| rng.gen_range(b'A', b'Z') as char)
        .collect::<String>()
}

impl Article {
    pub fn new(request: &NewArticleRequest) -> Article {
        let name = Article::generate_name(&request.title);
        let mut article = Article {
            name,
            title: request.title.clone(),
            body: request.body.clone(),
            id: request.id.clone(),
            ..Default::default()
        };
        //if let Some(ref properties) = request.properties {
        //article.properties = properties.clone();
        //}
        Article::add_default_properties(&mut article.properties);
        //if let Some(ref tags) = request.tags {
        //article.tags = tags.clone();
        //}
        article
    }

    fn generate_name(body: &str) -> String {
        for line in body.lines() {
            let name = line.trim();
            if !name.is_empty() {
                return name.to_string();
            }
        }
        format!("article-{}", random_string())
    }

    fn add_default_properties(properties: &mut PropertySet) {
        let now: DateTime<Utc> = Utc::now();
        properties.insert("timestamp".to_string(), now.to_string());
        properties.insert("epoch".to_string(), now.timestamp().to_string());
        properties.insert("year".to_string(), now.format("%Y").to_string());
        properties.insert("month".to_string(), now.format("%m").to_string());
        properties.insert("day".to_string(), now.format("%d").to_string());
    }
}

#[derive(Serialize, Deserialize, FromForm, Debug)]
pub struct NewArticleRequest {
    pub title: String,
    pub body: String,
    pub id: Option<String>,
    pub properties: String,
    pub tags: String,
}

fn article_file_name<'a>(path: &'a str) -> Cow<'a, str> {
    let article_filename_regex = Regex::new(r"[^A-Za-z0-9]+").unwrap();
    article_filename_regex.replace_all(path, "_")
}

pub fn create(article: &Article) -> std::io::Result<()> {
    let now: DateTime<Utc> = Utc::now();

    // TODO: path is full we wanna clip off the article here
    let location = format!(
        "data/articles/{}/{}.hbs",
        now.format("%Y/%m/%d"),
        &article_file_name(&article.name)
    );
    let path = Path::new(&location);
    let dir = path.parent().unwrap();
    create_dir_all(&dir)?;

    let mut file = File::create(path)?;
    file.write_all(article.body.as_bytes())?;

    update_index(&article, path)?;

    Ok(())
}

pub fn lookup_article(query_str: &str) -> std::io::Result<File> {
    println!("lookup_article(query_str: {:?})", query_str);
    let query: Query = query_str.try_into().unwrap();

    let path = match find_first_matching_path(&query) {
        Ok(r) => r,
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            println!("failed to find article, trying fallback...");
            load_fallback(&query)?
        }
        Err(e) => return Err(e),
    };
    println!("using article {:?}", path);
    // Ok(path.into_os_string().into_string().unwrap())
    File::open(path)
}

fn load_fallback(query: &Query) -> std::io::Result<PathBuf> {
    if let Some(ref id) = query.id {
        println!("query#id = {:?}", id);
        Ok(Path::new("templates/").join(format!("{}.hbs", id)))
    } else {
        return Err(io::Error::new(ErrorKind::NotFound, "not found"));
    }
}

pub fn lookup_articles(_query: &str) -> std::io::Result<Vec<File>> {
    Ok(vec![])
    //let path = Path::new("data/articles/").join(query);

    //Ok(vec![File::open(path).or_else(|_| {
    //let fallback_path = Path::new("templates/").join(format!("{}.hbs", query));
    //File::open(fallback_path)
    //})])
}

#[cfg(test)]
mod tests {
    use crate::articles::*;

    #[test]
    fn test_article_file_name() {
        assert_eq!(article_file_name("abcd"), "abcd");
        assert_eq!(article_file_name("a!@#bcd"), "a_bcd");
        assert_eq!(article_file_name("number 10"), "number_10");
        assert_eq!(article_file_name(""), "");
    }
}
