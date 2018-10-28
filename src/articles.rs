use chrono::prelude::*;
use regex::Regex;
use std::borrow::Cow;
use std::fs::{File, create_dir_all, read_dir, ReadDir};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Article {
    pub name: String,
    pub body: String,
}

fn article_file_name<'a>(path: &'a str) -> Cow<'a, str> {
    let article_filename_regex = Regex::new(r"[^A-Za-z]").unwrap();
    article_filename_regex.replace_all(path, "_")
}

pub fn create(article: Article) -> std::io::Result<()> {
    let now: DateTime<Utc> = Utc::now();

    // TODO: path is full we wanna clip off the article here
    let path_str = format!(
        "data/articles/{}/{}",
        now.format("%Y/%m/%d"),
        &article_file_name(&article.name));
    let path = Path::new(&path_str);
    let dir = path.parent().unwrap();
    create_dir_all(&dir)?;

    let mut file = File::create(path)?;
    file.write_all(article.body.as_bytes())?;
    Ok(())
}

pub fn lookup_article(query: &str) -> std::io::Result<File> {
    let query : Query = query.into();
    let path = match scan_articles(&query, index_reader("data/index")?).pop() {
        Some(article) => article,
        None => Path::new("templates/").join(format!("{}.hbs", query.id.unwrap())),
    };
    File::open(path)
}

pub fn lookup_articles(query: &str) -> std::io::Result<Vec<File>> {
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

fn index_reader(path: &str) -> std::io::Result<IndexIterator> {
    Ok(IndexIterator{reader: read_dir(path.into())?})
}

struct IndexIterator {
    reader: ReadDir
}

impl Iterator for IndexIterator {
    type Item = Key;

    fn next(&mut self) -> Option<Key> {
        loop {
            match self.reader.next() {
                Some(entry) => {
                    let path = entry.unwrap().path();
                    if !path.is_dir() {
                        if path.starts_with("ids") {
                        } else if path.starts_with("tags") {
                        } else if path.starts_with("properties") {
                        }

                        return Some(Key::ID{id: path.to_str().unwrap().into()})
                    }
                },
                None => return None,
            }
        }
    }
}

enum Key {
    ID{id: String},
    Property{key: String, value: String},
    Tag{name: String}
}

fn scan_articles<'a, I>(query: &Query, index: I) -> Vec<PathBuf>
where
    I: Iterator<Item = &'a Key> {
    let mut matches : Vec<PathBuf> = Vec::new();
    for key in index {
        match key {
            Key::ID{id} => return vec![id.into()],
            Key::Tag{name} => matches.push(name.into()),
            _ => (),
        };
    }
    matches
}

#[cfg(test)]
mod tests {
    use articles::*;

    #[test]
    fn test_article_file_name() {
        assert_eq!(article_file_name("abcd"), "abcd");
        assert_eq!(article_file_name("a!@#bcd"), "a___bcd");
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

    #[test]
    fn test_lookup_article() {
        let index = [Key::ID{id: "index".to_string()}];

        let result : Vec<PathBuf> = vec!["index".into()];
        assert_eq!(scan_articles(&"@index".into(), index.iter()), result);
    }
}
