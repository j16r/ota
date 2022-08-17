use std::collections::HashSet;
use std::fs::{self, create_dir_all, remove_dir, rename, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusty_ulid::Ulid;
use serde_yaml;
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
    articles_walker: walkdir::IntoIter,
    id_walker: walkdir::IntoIter,
}

impl LocalIterator {
    fn try_next(&mut self) -> Result<Option<<LocalIterator as Iterator>::Item>> {
        if let Some(ref id) = self.query.id {
            loop {
                if let Some(entry) = self.id_walker.next() {
                    let entry = entry.unwrap();
                    if !entry.file_type().is_dir() {
                        continue;
                    }

                    match common_prefix(entry.file_name().to_str().unwrap(), id) {
                        (_, _, remainder) if remainder.is_empty() => {
                            let key: Ulid = fs::read_to_string(entry.path().join("id"))
                                .unwrap()
                                .parse()
                                .unwrap();
                            return Ok(Some(Box::new(LocalEntry {
                                article: Article {
                                    key,
                                    id: id.to_owned(),
                                    title: String::new(),
                                    body: String::new(),
                                    properties: PropertySet::new(),
                                    tags: HashSet::new(),
                                },
                                path: entry.path().to_owned(),
                            })));
                        }
                        m => unreachable!("attempted to lookup empty limb in trie {:?}", m),
                    }
                } else {
                    return Ok(None);
                }
            }

            // Tags specified
        } else if !self.query.tags.is_empty() {
            eprintln!("Searching for tags {:?}", self.query.tags);

            return Ok(None);

            // All case
        } else {
            loop {
                if let Some(entry) = self.articles_walker.next() {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(err) => {
                            if let Some(e) = err.io_error() {
                                // Special case: no index exists on disk
                                if e.kind() == std::io::ErrorKind::NotFound {
                                    return Ok(None);
                                }
                            }
                            return Err(err.into());
                        }
                    };
                    dbg!(&entry);
                    if !entry.file_type().is_file() {
                        continue;
                    }
                    let entry_path = entry.path();
                    let key: Ulid = entry_path.file_prefix().unwrap().to_str().unwrap().parse().unwrap();
                    let entry_parent = entry_path.parent().unwrap();
                    let mut article: Article = serde_yaml::from_str(&fs::read_to_string(&entry_path)?)?;
                    return Ok(Some(Box::new(LocalEntry {
                        article, 
                        path: entry.path().to_owned(),
                    })));
                } else {
                    return Ok(None);
                }
            }
        }
    }
}

impl Iterator for LocalIterator {
    type Item = Box<dyn Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().unwrap()
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

fn datetime_to_filename(time: &DateTime<Utc>) -> String {
    format!("{}", time.format("%Y%m%d%H%M%S.%f"))
}

impl Index for Local {
    // update adds a new entry into the index, returning the location where
    // the article is to be stored
    fn update(&mut self, article: &Article) -> Result<Box<dyn Entry>> {
        let now: DateTime<Utc> = article.timestamp().parse().unwrap();

        let key = article.key.to_string();
        let article_root = self.path.join("articles");
        create_dir_all(&article_root)?;
        let path = update_dir_trie(&article_root, Path::new(&datetime_to_filename(&now)))?;

        let article_path = path.join(format!("{}.yaml", key));
        let mut article_file = File::create(&article_path)?;
        article_file.write_all(serde_yaml::to_string(&article)?.as_bytes())?;

        let key_root = self.path.join("index/key");
        create_dir_all(&key_root)?;
        let path = update_dir_trie(&key_root, Path::new(&key))?;

        let mut key_index_file = File::create(&path.join("key"))?;
        key_index_file.write_all(key.as_bytes())?;

        let id_root = self.path.join("index/id");
        create_dir_all(&id_root)?;
        let path = update_dir_trie(&id_root, Path::new(&article.id))?;

        let mut id_index_file = File::create(&path.join("id"))?;
        id_index_file.write_all(key.as_bytes())?;

        for tag in article.tags.iter() {
            let tag_root = self.path.join("index/tags");
            create_dir_all(&tag_root)?;
            let path = update_dir_trie(&tag_root, Path::new(&tag))?;

            let mut tag_index_file = File::create(&path.join("tag"))?;
            tag_index_file.write_all(tag.as_bytes())?;
        }

        Ok(Box::new(LocalEntry {
            path,
            article: article.clone(),
        }))
    }

    // search returns an iterator that returns all articles that match the supplied query
    fn search(&mut self, query: &Query) -> Result<Box<dyn Iterator<Item = Box<dyn Entry>>>> {
        Ok(Box::new(LocalIterator {
            query: query.clone(),
            articles_walker: WalkDir::new(&self.path.join("articles"))
                .sort_by_file_name()
                .min_depth(1)
                .into_iter(),
            id_walker: WalkDir::new(&self.path.join("index/id"))
                .sort_by_file_name()
                .min_depth(1)
                .into_iter(),
        }))
    }
}

fn move_contents(from: &Path, to: &Path) -> std::io::Result<()> {
    for entry in WalkDir::new(&from).min_depth(1).max_depth(1).into_iter() {
        let entry = entry?;
        rename(
            entry.path(),
            to.join(entry.path().strip_prefix(from).unwrap()),
        )?;
    }
    Ok(())
}

fn update_dir_trie(root: &Path, location: &Path) -> std::io::Result<PathBuf> {
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
                move_contents(entry.path(), &new_path)?;
                remove_dir(&entry.path())?;
                if !remainder.is_empty() {
                    return update_dir_trie(&new_root, Path::new(remainder));
                }
                return Ok(new_root);
            }
            (_, _, remainder) if !remainder.is_empty() => {
                return update_dir_trie(entry.path(), Path::new(remainder));
            }
            m => unreachable!("attempted to store empty limb in trie {:?}", m),
        }
    }
    let new_path = root.join(location);
    create_dir_all(&new_path)?;
    Ok(new_path)
}

fn common_prefix<'a, 'b>(a: &'a str, b: &'b str) -> (&'b str, &'a str, &'b str) {
    let at = a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count();
    (
        b.get(..at).unwrap(),
        a.get(at..).unwrap(),
        b.get(at..).unwrap(),
    )
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

        let mut out = update_dir_trie(&root, Path::new("a")).unwrap();
        assert!(out.ends_with("a"));
        assert_eq!(enumerate_dirs(&root), ["a"]);

        out = update_dir_trie(&root, Path::new("b")).unwrap();
        assert_eq!("b", out.file_name().unwrap());
        assert!(out.ends_with("b"));
        assert_eq!(enumerate_dirs(&root), ["a", "b"]);

        // Sees an existing root part of the branch, splits off the "b"
        out = update_dir_trie(&root, Path::new("ab")).unwrap();
        assert!(out.ends_with("a/b"));
        assert_eq!(enumerate_dirs(&root), ["a", "a/b", "b"]);

        // Create a longer path that will be split later
        out = update_dir_trie(&root, Path::new("da")).unwrap();
        assert!(out.ends_with("da"));
        assert_eq!(enumerate_dirs(&root), ["a", "a/b", "b", "da"]);

        out = update_dir_trie(&root, Path::new("d")).unwrap();
        assert!(out.ends_with("d"));
        assert_eq!(enumerate_dirs(&root), ["a", "a/b", "b", "d", "d/a"]);

        // Tests move contents
        out = update_dir_trie(&root, Path::new("caa")).unwrap();
        assert!(out.ends_with("caa"));
        assert_eq!(enumerate_dirs(&root), ["a", "a/b", "b", "caa", "d", "d/a"]);

        out = update_dir_trie(&root, Path::new("ca")).unwrap();
        assert!(out.ends_with("ca"));
        assert_eq!(
            enumerate_dirs(&root),
            ["a", "a/b", "b", "ca", "ca/a", "d", "d/a"]
        );

        out = update_dir_trie(&root, Path::new("c")).unwrap();
        assert!(out.ends_with("c"));
        assert_eq!(
            enumerate_dirs(&root),
            ["a", "a/b", "b", "c", "c/a", "c/a/a", "d", "d/a"]
        );

        // Tests recursion
        out = update_dir_trie(&root, Path::new("caab")).unwrap();
        assert!(out.ends_with("c/a/a/b"));
        assert_eq!(
            enumerate_dirs(&root),
            ["a", "a/b", "b", "c", "c/a", "c/a/a", "c/a/a/b", "d", "d/a"]
        );

        // Tests where two paths have a shared root but don't neatly fit into each other
        out = update_dir_trie(&root, Path::new("eea")).unwrap();
        assert!(out.ends_with("eea"));
        assert_eq!(
            enumerate_dirs(&root),
            ["a", "a/b", "b", "c", "c/a", "c/a/a", "c/a/a/b", "d", "d/a", "eea"]
        );

        out = update_dir_trie(&root, Path::new("eeb")).unwrap();
        assert!(out.ends_with("ee/b"));
        assert_eq!(
            enumerate_dirs(&root),
            ["a", "a/b", "b", "c", "c/a", "c/a/a", "c/a/a/b", "d", "d/a", "ee", "ee/a", "ee/b"]
        );
    }
}
