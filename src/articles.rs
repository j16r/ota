use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{File, create_dir_all, read_dir, ReadDir};
use std::io::prelude::*;
use std::io::{self, ErrorKind};
use std::iter;
use std::path::{Path, PathBuf};

use chrono::prelude::*;
use rand::Rng;
use regex::Regex;
use rocket::form::FromForm;
use serde_derive::{Serialize, Deserialize};

type PropertySet = HashMap<String, String>;

#[derive(Serialize, Default, Debug)]
pub struct Article {
    name: String,
    body: String,
    id: Option<String>,
    properties: PropertySet,
    tags: HashSet<String>
}

fn random_string() -> String {
    let mut rng = rand::thread_rng();
    iter::repeat(16).map(|_| rng.gen_range(b'A', b'Z') as char).collect::<String>()
}

impl Article {
    pub fn new(request: &NewArticleRequest) -> Article {
        let name = Article::generate_name(&request.body);
        let mut article = Article{
            name,
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
    pub body: String,
    pub id: Option<String>,
    pub properties: String,
    pub tags: String,
}

fn article_file_name<'a>(path: &'a str) -> Cow<'a, str> {
    let article_filename_regex = Regex::new(r"[^A-Za-z0-9]+").unwrap();
    article_filename_regex.replace_all(path, "_")
}

pub fn create(article: Article) -> std::io::Result<()> {
    let now: DateTime<Utc> = Utc::now();

    // TODO: path is full we wanna clip off the article here
    let location = format!(
        "data/articles/{}/{}.hbs",
        now.format("%Y/%m/%d"),
        &article_file_name(&article.name));
    let path = Path::new(&location);
    let dir = path.parent().unwrap();
    create_dir_all(&dir)?;

    let mut file = File::create(path)?;
    file.write_all(article.body.as_bytes())?;

    update_index(article, path)?;

    Ok(())
}

fn update_index(article: Article, location: &Path) -> std::io::Result<()> {
    println!("update_index({:?}, {:?})", article, location);
    if let Some(id) = article.id {
        return update_index_id(&id, location, Path::new("data/index/"));
    }

    Ok(())
}

fn update_index_id(segment: &str, location: &Path, search_path: &Path) -> std::io::Result<()> {
    println!("update_index_id({:?}, {:?}, {:?})", segment, location, search_path);

    let reader = read_dir(search_path)?;
    for entry in reader {
        let path = entry?.path();
        if path.is_dir() {
            let path_str = path.file_name().unwrap().to_str().unwrap();
            println!("path {:?}", path_str);
            if segment == path_str {
                println!("full match");

                // TODO: an article already exists, should we preserve an index to it somehow?
                return create_index_entry(&path, location);

            } else if path_str.starts_with(segment) {
                println!("collision");
                // collision

            } else if segment.starts_with(path_str) {
                println!("segment match");
                let segment = segment.get(path_str.len()..).unwrap();
                return update_index_id(segment, location, &path);

            } else {
                println!("no match");
                break
            }
        }
    }

    println!("empty");
    let destination = search_path.join(segment);
    create_index_entry(&destination, location)
}

fn create_index_entry(destination: &Path, location: &Path) -> std::io::Result<()> {
    println!("creating index at {:?} for location {:?}", destination, location);

    create_dir_all(&destination)?;

    let mut file = File::create(&destination.join("0"))?;
    file.write_all(location.to_str().unwrap().as_bytes())
}

pub fn lookup_article(query_str: &str) -> std::io::Result<File> {
//Result<String, std::io::Error> {
    println!("lookup_article(query_str: {:?})", query_str);
    let query : Query = query_str.into();

    let path = match find_first_matching_path(&query) {
        Ok(r) => r,
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            println!("failed to find article, trying fallback...");
            load_fallback(&query)?
        },
        Err(e) => return Err(e)
    };
    println!("using article {:?}", path);
    // Ok(path.into_os_string().into_string().unwrap())
    File::open(path)
}

fn find_first_matching_path(query: &Query) -> std::io::Result<PathBuf> {
    scan_articles(&query, &mut index_reader("data/index")?)
        .and_then(|mut scanner| {
            scanner.pop().ok_or_else(|| io::Error::new(ErrorKind::NotFound, "not found"))
        })
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

#[derive(Default)]
struct Query {
    id: Option<String>,
    properties: Vec<PropertyFilter>,
    tags: Vec<String>
}

impl fmt::Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Query {{ id: {:?} }}", self.id)
    }
}


enum PropertyFilter {
    Equals(String),
    Lt(String),
    Gt(String)
}

impl<'a> From<&'a str> for Query {
    fn from(query: &'a str) -> Self {
        let mut result = Query{id: None, properties: Vec::new(), tags: Vec::new()};

        for capture in query.split(" ") {
            if capture.starts_with("@") {
                let id = &capture[1..];
                if result.id == None {
                    result.id = Some(id.into());
                } else {
                    panic!("duplicate id in query string");
                }
            } else {
                result.tags.push(capture.into());
            }
        }

        result
    }
}

enum Key {
    ID{id: String},
    Property{key: String, value: String},
    Tag{name: String}
}

trait Index {
    fn from(&self, entry: &Entry) -> Self;
    fn find_matches(&mut self, query: &Query) -> std::io::Result<Vec<Entry>>;
}

enum Entry {
    Branch(String),
    Article(PathBuf),
}

struct DirectoryIndex {
    path: PathBuf,
    reader: ReadDir
}

fn index_reader(path: &str) -> std::io::Result<DirectoryIndex> {
    let path_buf = Path::new(path).to_path_buf();
    Ok(DirectoryIndex{
        path: path_buf.clone(),
        reader: read_dir(path_buf)?,
    })
}

impl Index for DirectoryIndex {
    fn from(&self, entry: &Entry) -> Self {
        if let Entry::Branch(branch) = entry {
            let path_buf = self.path.join(branch);
            DirectoryIndex{
                path: path_buf.clone(),
                reader: read_dir(path_buf).unwrap(),
            }
        } else {
            unreachable!("from called with non Branch");
        }
    }

    fn find_matches(&mut self, query: &Query) -> std::io::Result<Vec<Entry>> {
        let mut matches : Vec<Entry> = Vec::new();
        match self.reader.next() {
            Some(entry) => {
                if let Some(ref id) = query.id {
                    let path = entry.unwrap().path();
                    if path.is_dir() {
                        let path_str = path.to_str().unwrap();
                        if path == path {
                            // TODO: scan for latest file
                            let mut file = File::open(path.join("0"))?;
                            let mut buffer = String::new();
                            file.read_to_string(&mut buffer)?;

                            matches.push(Entry::Article(buffer.into()));
                        } else if id.starts_with(path_str) {
                            matches.push(Entry::Branch(path_str.into()));
                        }
                    }
                }
            },
            _ => (),
        }
        Ok(matches)
    }
}

fn scan_articles<I>(query: &Query, index: &mut I) -> std::io::Result<Vec<PathBuf>>
    where I: Index {
    let mut matches : Vec<PathBuf> = Vec::new();
    if let Some(ref id) = query.id {
        for entry in index.find_matches(query)?.iter() {
            match entry {
                Entry::Article(path) => matches.push(path.into()),
                Entry::Branch(branch) => {
                    let new_id = id.get(branch.len()..).unwrap();
                    let new_query = Query{
                        id: Some(new_id.to_string()),
                        ..Default::default()
                    };
                    matches.append(
                        &mut scan_articles(&new_query, &mut index.from(entry))?.clone());
                }
            }
        }
    }
    Ok(matches)
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

    #[test]
    fn test_query_from_str() {
        let mut query : Query;
        query = "@index".into();
        assert_eq!(query.id, Some("index".to_string()));

        query = "tag".into();
        assert_eq!(query.tags, vec!["tag".to_string()]);

        query = "tag1 tag2".into();
        assert_eq!(query.tags, vec!["tag1".to_string(), "tag2".to_string()]);
    }

    struct TestIndex {}

    impl Index for TestIndex {
        fn from(&self, entry: &Entry) -> Self {
            TestIndex{}
        }

        fn find_matches(&mut self, query: &Query) -> std::io::Result<Vec<Entry>> {
            Ok(vec![Entry::Article(Path::new("index").into())])
        }
    }

    #[test]
    fn test_lookup_article() {
        let mut index = TestIndex{};

        let result : Vec<PathBuf> = vec!["index".into()];
        assert_eq!(scan_articles(&"@index".into(), &mut index).unwrap(), result);
    }
}
