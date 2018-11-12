use chrono::prelude::*;
use regex::Regex;
use std::borrow::Cow;
use std::fs::{File, create_dir_all, read_dir, ReadDir};
use std::io::prelude::*;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::fmt;

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

pub fn lookup_article(query_str: &str) -> std::io::Result<File> {
    let query : Query = query_str.into();
    let path = match scan_articles(&query_str.into(), &mut index_reader("data/index")?).pop() {
        Some(article) => article,
        None => {
            if let Some(id) = query.id {
                Path::new("templates/").join(format!("{}.hbs", id))
            } else {
                return Err(io::Error::new(ErrorKind::NotFound, "not found"));
            }
        }
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

//struct IndexIterator {
    //reader: ReadDir
//}

//impl Iterator for IndexIterator {
    //type Item = Key;

    //fn next(&mut self) -> Option<Key> {
        //loop {
            //match self.reader.next() {
                //Some(entry) => {
                    //let path = entry.unwrap().path();
                    //if !path.is_dir() {
                        //if path.starts_with("ids") {
                        //} else if path.starts_with("tags") {
                        //} else if path.starts_with("properties") {
                        //}

                        //return Some(Key::ID{id: path.to_str().unwrap().into()})
                    //}
                //},
                //None => return None,
            //}
        //}
    //}
//}

enum Key {
    ID{id: String},
    Property{key: String, value: String},
    Tag{name: String}
}

// we have an index, typically a file system which contains:
//  arcticles
//  ways to refer back to the article (text file containing an article path)
//      id
//          a/
//              ardvark
//              l/
//                  pha
//                  phabetical
//              note
//
// so if we have a key of alphabetical, we have to look up a, l, then [pha, phabetical]
//
// so [alphabetical]
// [a] alphabetical starts with a
// [ardvark, l] lphaebetical
// [pha, phabetical] phabetical
//

//scan starts with an iterator which returns either another iterator (with a name) or a terminal node

//so an index is an iterator that can return an iterator or a terminal

trait Index {
    fn from(&self, entry: &Entry) -> Self;
    fn find_matches(&mut self, query: &Query) -> Vec<Entry>;
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

    fn find_matches(&mut self, query: &Query) -> Vec<Entry> {
        let mut matches : Vec<Entry> = Vec::new();
        match self.reader.next() {
            Some(entry) => {
                if let Some(ref id) = query.id {
                    let path = entry.unwrap().path();
                    if path.is_dir() {
                        let path_str = path.to_str().unwrap();
                        if id.starts_with(path_str) {
                            matches.push(Entry::Branch(path_str.into()));
                        }
                    } else {
                        if path == path {
                            matches.push(Entry::Article(path));
                        }
                    }
                }
            },
            _ => (),
        }
        matches
    }
}

fn scan_articles<I>(query: &Query, index: &mut I) -> Vec<PathBuf>
    where I: Index {
    let mut matches : Vec<PathBuf> = Vec::new();
    if let Some(ref id) = query.id {
        for entry in index.find_matches(query).iter() {
            match entry {
                Entry::Article(path) => matches.push(path.into()),
                Entry::Branch(branch) => {
                    let new_id = id.get(branch.len()..).unwrap();
                    let new_query = Query{
                        id: Some(new_id.to_string()),
                        ..Default::default()
                    };
                    matches.append(
                        &mut scan_articles(&new_query, &mut index.from(entry)).clone());
                }
            }
        }
    }
    matches
}


//fn scan_articles<'a, I>(query: &Query, index: I) -> Vec<PathBuf>
//where
    //I: Iterator<Item = &'a Key> {
    //let mut matches : Vec<PathBuf> = Vec::new();
    //for key in index {
        //match key {
            //Key::ID{id} => return vec![id.into()],
            //Key::Tag{name} => matches.push(name.into()),
            //_ => (),
        //};
    //}
    //matches
//}

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
