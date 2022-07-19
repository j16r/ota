use std::borrow::Cow;
use std::collections::HashSet;
use std::fs::{create_dir_all, read_dir, File, ReadDir};
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};
use regex::Regex;

use crate::articles::{Article, PropertySet};
use crate::index::{Entry, Index};
use crate::query::Query;

pub struct Local {
    path: PathBuf,
}

impl Local {
    pub fn new<T: Into<PathBuf> + AsRef<Path>>(path: T) -> Result<Self> {
        Ok(Local { path: path.into() })
    }
}

pub struct LocalIterator {
    query: Query,
    reader: ReadDir,
}

impl Iterator for LocalIterator {
    type Item = Box<dyn Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref id) = self.query.id {
            dbg!(&id);
            if let Some(dir) = self.reader.next() {
                dbg!(&dir);
                let path = dir.unwrap().path();
                if path.is_dir() {
                    let path_str = path.to_str().unwrap();
                    if path_str == id {
                        // TODO: scan for latest file
                        return Some(Box::new(LocalEntry {
                            article: Article {
                                id: Some(id.clone()),
                                name: String::new(),
                                title: String::new(),
                                body: String::new(),
                                properties: PropertySet::new(),
                                tags: HashSet::new(),
                            },
                            path: path.to_path_buf(),
                        }));
                    }
                }
            }
        } else {
            // Empty query means get everything, we use the date index to return most recent
            // results at the top
        }
        None
    }
}

struct LocalEntry {
    path: PathBuf,
    article: Article,
}

impl Entry for LocalEntry {
    fn article(&self) -> Article {
        self.article.clone()
    }

    fn body(&self) -> Result<Box<dyn std::io::Read>> {
        Ok(Box::new(File::open(&self.path)?))
    }
}

impl Index for Local {
    // update adds a new entry into the index, returning the location where
    // the article is to be stored
    fn update(&mut self, article: &Article) -> Result<Box<dyn Entry>> {
        let now: DateTime<Utc> = article.timestamp().parse().unwrap();

        // TODO: path is full we wanna clip off the article here
        let article_location = self.path.join(format!(
            "articles/{}/{}.html.hbs",
            now.format("%Y/%m/%d"),
            &article_file_name(&article.name)
        ));
        let path = Path::new(&article_location);
        let dir = path.parent().unwrap();
        create_dir_all(&dir)?;

        let mut article_file = File::create(&article_location)?;
        article_file.write(article.body.as_bytes())?;

        // 3 different kinds of attributes
        // ids: unique/discrete, best to index in a trie
        // tags strings, not unique per article
        // properties are key:value, keys are unique, values are not

        // let name = article.name {

        if let Some(id) = &article.id {
            let id_index_location = self.path.join(format!("index/id/{}/location", id));
            let dir = id_index_location.parent().unwrap();
            create_dir_all(dir)?;

            let mut id_index_file = File::create(id_index_location)?;
            id_index_file.write(article_location.as_os_str().as_bytes())?;
        }

        // for tag in article.tags.iter() {
        // }

        // for (key, value) in article.properties.iter() {
        // }

        // let reader = read_dir(self.path)?;
        // for entry in reader {
        //     let path = entry?.path();
        //     if path.is_dir() {
        //         let path_str = path.file_name().unwrap().to_str().unwrap();
        //         println!("path {:?}", path_str);
        //         if segment == path_str {
        //             println!("full match");

        //             // TODO: an article already exists, should we preserve an index to it somehow?
        //             return create_index_entry(&path, location);
        //         } else if path_str.starts_with(segment) {
        //             println!("collision");
        //             // collision
        //         } else if segment.starts_with(path_str) {
        //             println!("segment match");
        //             let segment = segment.get(path_str.len()..).unwrap();
        //             return update_index_id(segment, location, &path);
        //         } else {
        //             println!("no match");
        //             break;
        //         }
        //     }
        // }

        // println!("empty");
        // let destination = search_path.join(segment);
        // create_index_entry(&destination, location)
        // Ok(())
        Ok(Box::new(LocalEntry {
            path: path.to_path_buf(),
            article: article.clone(),
        }))
        // Ok(Entry {
        //     id: article.id.clone(),
        //     properties: PropertySet::new(),
        //     tags: HashSet::new(),
        //     // path: self.path.clone(),
        // })
    }

    // search returns an iterator that returns all articles that match the supplied query
    fn search(&mut self, query: &Query) -> Result<Box<dyn Iterator<Item = Box<dyn Entry>>>> {
        println!("reading directory {:?}", &self.path);
        Ok(Box::new(LocalIterator {
            query: query.clone(),
            reader: read_dir(&self.path)?,
        }))
    }
}

fn update_dir_trie(path: &Path) -> std::io::Result<()> {
    create_dir_all(path)
}

fn article_file_name(path: &str) -> Cow<str> {
    let article_filename_regex = Regex::new(r"[^A-Za-z0-9]+").unwrap();
    article_filename_regex.replace_all(path, "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;
    use walkdir::WalkDir;

    fn enumerate_dirs(path: &Path) -> Vec<String> {
        let path_str = path.to_str().unwrap();
        WalkDir::new(path)
            .into_iter()
            .filter_map(|d| {
                d.unwrap()
                    .path()
                    .to_str()
                    .unwrap()
                    .strip_prefix(path_str)
                    .map(|s| s.to_owned())
            })
            .filter(|s| !s.is_empty())
            .collect()
    }

    #[test]
    fn test_article_file_name() {
        assert_eq!(article_file_name("abcd"), "abcd");
        assert_eq!(article_file_name("a!@#bcd"), "a_bcd");
        assert_eq!(article_file_name("number 10"), "number_10");
        assert_eq!(article_file_name(""), "");
    }

    #[test]
    fn test_article_update_only_id() {
        let dir = TempDir::new("").unwrap();
        // assert!(enumerate_dirs(&dir.path()).is_empty());

        update_dir_trie(&dir.path().join("a")).unwrap();
        assert_eq!(enumerate_dirs(&dir.path()), ["/a"]);

        update_dir_trie(&dir.path().join("b")).unwrap();
        assert_eq!(enumerate_dirs(&dir.path()), ["/a", "/b"]);

        update_dir_trie(&dir.path().join("ab")).unwrap();
        assert_eq!(enumerate_dirs(&dir.path()), ["/a", "/a/b", "/b"]);

        update_dir_trie(&dir.path().join("ad")).unwrap();
        assert_eq!(enumerate_dirs(&dir.path()), ["/a", "/a/b", "/a/d", "/b"]);
    }
}
