use std::borrow::Cow;
use std::collections::HashSet;
use std::fs::{create_dir_all, remove_dir, rename, File};
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};
use regex::Regex;
use walkdir::WalkDir;

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
    walker: walkdir::IntoIter,
}

impl Iterator for LocalIterator {
    type Item = Box<dyn Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref id) = self.query.id {
            dbg!(&id);
            if let Some(entry) = self.walker.next() {
                let entry = entry.unwrap();
                dbg!(&entry);
                let entry_path = entry.file_name().to_str().unwrap().to_owned();
                dbg!(&entry_path);

                if &entry_path == id {
                    return Some(Box::new(LocalEntry {
                        article: Article {
                            id: Some(id.clone()),
                            name: String::new(),
                            title: String::new(),
                            body: String::new(),
                            properties: PropertySet::new(),
                            tags: HashSet::new(),
                        },
                        path: entry.path().to_path_buf(),
                    }));
                }

                // if let Some(remainder) = entry_path.strip_prefix(&location_str) {

                // let path = dir.unwrap();
                // dbg!(&path);
                // if path.file_type().is_dir() {
                //     let path_str = path.to_str().unwrap();
                //     if path_str == id {
                //         // TODO: scan for latest file
                //         return Some(Box::new(LocalEntry {
                //             article: Article {
                //                 id: Some(id.clone()),
                //                 name: String::new(),
                //                 title: String::new(),
                //                 body: String::new(),
                //                 properties: PropertySet::new(),
                //                 tags: HashSet::new(),
                //             },
                //             path: path.to_path_buf(),
                //         }));
                //     }
                // }
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
        let article_location = format!(
            "articles/{}/{}.html.hbs",
            now.format("%Y%m%d"),
            &article_file_name(&article.name)
        );
        let path = Path::new(&article_location);
        let dir = path.parent().unwrap();
        update_dir_trie(&self.path, dir)?;
        // create_dir_all(&dir)?;

        let mut article_file = File::create(&self.path.join(&article_location))?;
        article_file.write_all(article.body.as_bytes())?;

        // 3 different kinds of attributes
        // ids: unique/discrete, best to index in a trie
        // tags strings, not unique per article
        // properties are key:value, keys are unique, values are not

        // let name = article.name {

        if let Some(id) = &article.id {
            let id_index_location = format!("index/id/{}/location", id);
            let path = Path::new(&id_index_location);
            let dir = path.parent().unwrap();
            update_dir_trie(&self.path, dir)?;

            let mut id_index_file = File::create(&self.path.join(&id_index_location))?;
            id_index_file.write_all(article_location.as_str().as_bytes())?;
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
            walker: WalkDir::new(&self.path.join("index/id"))
                .sort_by_file_name()
                .min_depth(1)
                .into_iter(),
        }))
    }
}

fn move_contents(from: &Path, to: &Path) -> std::io::Result<()> {
    eprintln!("move_contents({:?}, {:?})", &from, &to);
    for entry in WalkDir::new(&from).min_depth(1).max_depth(1).into_iter() {
        let entry = entry?;
        rename(
            entry.path(),
            to.join(entry.path().strip_prefix(from).unwrap()),
        )?;
    }
    Ok(())
}

fn update_dir_trie(root: &Path, location: &Path) -> std::io::Result<()> {
    eprintln!("update_dir_trie({:?}, {:?})", &root, &location);
    for entry in WalkDir::new(root)
        .sort_by_file_name()
        .min_depth(1)
        .max_depth(1)
        .into_iter()
    {
        let entry = entry?;

        match common_prefix(
            entry.file_name().to_str().unwrap(),
            location.to_str().unwrap(),
        ) {
            (prefix, suffix, remainder) if prefix.is_empty() => {
                assert!(!suffix.is_empty());
                assert!(!remainder.is_empty());
                continue;
            }
            (prefix, suffix, remainder) if !suffix.is_empty() => {
                let new_root = root.join(Path::new(prefix));
                let new_path = new_root.join(suffix);

                create_dir_all(&new_path)?;
                if !remainder.is_empty() {
                    update_dir_trie(&new_root, Path::new(remainder))?;
                }
                move_contents(entry.path(), &new_path)?;
                return remove_dir(&entry.path());
            }
            (_, _, remainder) if !remainder.is_empty() => {
                return update_dir_trie(entry.path(), Path::new(remainder));
            }
            m => unreachable!("attempted to store empty limb in trie {:?}", m),
        }
    }
    create_dir_all(&root.join(location))
}

fn common_prefix<'a, 'b>(a: &'a str, b: &'b str) -> (&'b str, &'a str, &'b str) {
    eprintln!("common_prefix({}, {})", a, b);
    let at = a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count();
    (
        b.get(..at).unwrap(),
        a.get(at..).unwrap(),
        b.get(at..).unwrap(),
    )
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
        let prefix = path.to_owned();
        WalkDir::new(path)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|d| {
                d.unwrap()
                    .path()
                    .strip_prefix(&prefix)
                    .unwrap()
                    .to_str()
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
    fn test_common_prefix() {
        assert_eq!(common_prefix("a", "b"), ("", "a", "b"));
        assert_eq!(common_prefix("a", "a"), ("a", "", ""));
        assert_eq!(common_prefix("aa", "aa"), ("aa", "", ""));
        assert_eq!(common_prefix("aab", "aac"), ("aa", "b", "c"));
    }

    #[test]
    fn test_update_dir_trie() {
        let temp = TempDir::new("update_dir_trie_test").unwrap();
        let root = temp.path().to_owned();
        assert!(enumerate_dirs(&root).is_empty());

        update_dir_trie(&root, Path::new("a")).unwrap();
        assert_eq!(enumerate_dirs(&root), ["a"]);

        update_dir_trie(&root, Path::new("b")).unwrap();
        assert_eq!(enumerate_dirs(&root), ["a", "b"]);

        // Sees an existing root part of the branch, splits off the "b"
        update_dir_trie(&root, Path::new("ab")).unwrap();
        assert_eq!(enumerate_dirs(&root), ["a", "a/b", "b"]);

        // Create a longer path that will be split later
        update_dir_trie(&root, Path::new("da")).unwrap();
        assert_eq!(enumerate_dirs(&root), ["a", "a/b", "b", "da"]);

        update_dir_trie(&root, Path::new("d")).unwrap();
        assert_eq!(enumerate_dirs(&root), ["a", "a/b", "b", "d", "d/a"]);

        // Tests move contents
        update_dir_trie(&root, Path::new("caa")).unwrap();
        assert_eq!(enumerate_dirs(&root), ["a", "a/b", "b", "caa", "d", "d/a"]);

        update_dir_trie(&root, Path::new("ca")).unwrap();
        assert_eq!(
            enumerate_dirs(&root),
            ["a", "a/b", "b", "ca", "ca/a", "d", "d/a"]
        );

        update_dir_trie(&root, Path::new("c")).unwrap();
        assert_eq!(
            enumerate_dirs(&root),
            ["a", "a/b", "b", "c", "c/a", "c/a/a", "d", "d/a"]
        );

        // Tests recursion
        update_dir_trie(&root, Path::new("caab")).unwrap();
        assert_eq!(
            enumerate_dirs(&root),
            ["a", "a/b", "b", "c", "c/a", "c/a/a", "c/a/a/b", "d", "d/a"]
        );

        // Tests where two paths have a shared root but don't neatly fit into each other
        update_dir_trie(&root, Path::new("eea")).unwrap();
        assert_eq!(
            enumerate_dirs(&root),
            ["a", "a/b", "b", "c", "c/a", "c/a/a", "c/a/a/b", "d", "d/a", "eea"]
        );

        update_dir_trie(&root, Path::new("eeb")).unwrap();
        assert_eq!(
            enumerate_dirs(&root),
            ["a", "a/b", "b", "c", "c/a", "c/a/a", "c/a/a/b", "d", "d/a", "ee", "ee/a", "ee/b"]
        );
    }
}
