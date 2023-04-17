use std::fs::{self, create_dir_all, remove_dir, rename, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Utc};
use rusty_ulid::Ulid;
use serde_yaml;
use walkdir::WalkDir;

use crate::articles::Article;
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
    path: PathBuf,
    query: Query,
    articles_walker: walkdir::IntoIter,
    id_walker: walkdir::IntoIter,
}

impl LocalIterator {
    fn load_article(&mut self, key: &Ulid) -> Result<Article> {
        eprintln!("load_article({:?})", key);
        let mut walker = WalkDir::new(&self.path.join("index/key"))
            .sort_by_file_name()
            .min_depth(1)
            .into_iter();
        let mut search = key.to_string();
        loop {
            if let Some(entry) = walker.next() {
                let entry = entry?;
                if entry.file_type().is_file() {
                    continue;
                }

                dbg!(&entry);
                match common_prefix(entry.file_name().to_str().unwrap(), &search) {
                    (_, _, remainder) if remainder.is_empty() => {
                        dbg!(&remainder);
                        let article: Article = serde_yaml::from_str(&fs::read_to_string(
                            entry.path().join("meta.yaml"),
                        )?)?;
                        return Ok(article);
                    }
                    (prefix, _, remainder) if !prefix.is_empty() && !remainder.is_empty() => {
                        dbg!(&prefix, &remainder);
                        search = remainder.to_owned();
                        continue;
                    }
                    (a, b, remainder) if b.is_empty() && remainder.is_empty() => {
                        dbg!(&a, &b, &remainder);
                        continue;
                    }
                    m => unreachable!("attempted to lookup empty limb in trie {:?}", m),
                }

                // let entry_path = entry.path();
                // dbg!(&entry_path);
                // let article: Article = serde_yaml::from_str(&fs::read_to_string(&entry_path)?)?;
                // return Ok(article);
            } else {
                todo!("not found");
            }
        }
    }

    fn load_article_body(&mut self, key: &Ulid) -> Result<String> {
        eprintln!("load_article_body({:?})", key);
        let mut walker = WalkDir::new(&self.path.join("articles"))
            .sort_by_file_name()
            .min_depth(1)
            .into_iter();
        for entry in walker {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            dbg!(&entry);
            let entry_path = entry.path();
            let found_key: Ulid = entry_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .strip_suffix(".html.hbs")
                .unwrap()
                .parse()?;
            if key == &found_key {
                dbg!(&key, &found_key);
                return fs::read_to_string(entry_path).map_err(anyhow::Error::from);
            }
        }
        Err(anyhow!("article with key {} not found", key))
    }

    fn try_next(&mut self) -> Result<Option<<LocalIterator as Iterator>::Item>> {
        eprintln!("try_next({:?})", self.query);
        if let Some(ref id) = self.query.id {
            loop {
                if let Some(entry) = self.id_walker.next() {
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
                    if !entry.file_type().is_dir() {
                        continue;
                    }

                    dbg!(&entry);
                    match common_prefix(entry.file_name().to_str().unwrap(), id) {
                        (_, _, remainder) if remainder.is_empty() => {
                            dbg!(&remainder);
                            let key: Ulid =
                                fs::read_to_string(dbg!(entry.path().join("key.txt")))?.parse()?;
                            let mut article = self.load_article(&key)?;
                            article.body = self.load_article_body(&key)?;
                            return Ok(Some(Box::new(LocalEntry {
                                article,
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
                    let key: Ulid = entry_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .strip_suffix(".html.hbs")
                        .unwrap()
                        .parse()?;
                    let mut article = self.load_article(&key)?;
                    article.body = fs::read_to_string(entry_path)?;
                    return Ok(Some(Box::new(LocalEntry {
                        article,
                        path: entry_path.to_path_buf(),
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

        // First store the raw article body as a handlebars template
        let key = article.key.to_string();
        let article_root = self.path.join("articles");
        create_dir_all(&article_root)?;
        let path = update_dir_trie(&article_root, Path::new(&datetime_to_filename(&now)))?;

        let article_path = path.join(format!("{}.html.hbs", key));
        let mut article_file = File::create(&article_path)?;
        article_file.write_all(article.body.as_bytes())?;

        // We store meta data in the key index, for fast lookup
        let key_root = self.path.join("index/key");
        create_dir_all(&key_root)?;
        let path = update_dir_trie(&key_root, Path::new(&key))?;

        let mut key_index_file = File::create(&path.join("meta.yaml"))?;
        key_index_file.write_all(serde_yaml::to_string(&article)?.as_bytes())?;

        // All other indexes could be a symlink to the meta data, or the article
        let id_root = self.path.join("index/id");
        create_dir_all(&id_root)?;
        let path = update_dir_trie(&id_root, Path::new(&article.id))?;

        let mut key_index_file = File::create(&path.join("key.txt"))?;
        key_index_file.write_all(key.as_bytes())?;

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
            path: self.path.clone(),
            query: query.clone(),
            articles_walker: WalkDir::new(self.path.join("articles"))
                .sort_by_file_name()
                .min_depth(1)
                .into_iter(),
            id_walker: WalkDir::new(self.path.join("index/id"))
                .sort_by_file_name()
                .min_depth(1)
                .into_iter(),
        }))
    }
}

fn move_contents(from: &Path, to: &Path) -> std::io::Result<()> {
    for entry in WalkDir::new(from).min_depth(1).max_depth(1).into_iter() {
        let entry = entry?;
        rename(
            entry.path(),
            to.join(entry.path().strip_prefix(from).unwrap()),
        )?;
    }
    Ok(())
}

fn update_dir_trie(root: &Path, location: &Path) -> Result<PathBuf> {
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
                remove_dir(entry.path())?;
                if !remainder.is_empty() {
                    return update_dir_trie(&new_root, Path::new(remainder));
                }
                return Ok(new_root);
            }
            (_, _, remainder) if !remainder.is_empty() => {
                return update_dir_trie(entry.path(), Path::new(remainder));
            }
            (prefix, suffix, remainder) if suffix.is_empty() && remainder.is_empty() => {
                bail!("entry already exists in trie: {}", prefix);
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
