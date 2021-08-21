use std::fs::{create_dir_all, File, read_dir, ReadDir};
use std::io::prelude::*;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use crate::articles::Article;
use crate::query::Query;

trait Index {
    fn from(&self, entry: &Entry) -> Self;
    fn find_matches(&mut self, query: &Query) -> std::io::Result<Vec<Entry>>;
}

enum Key {
    ID{id: String},
    Property{key: String, value: String},
    Tag{name: String}
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

// FIXME: I've forgotten how I intended to use this, but I suspect it needed more thought
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

pub fn update_index(article: Article, location: &Path) -> std::io::Result<()> {
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


pub fn find_first_matching_path(query: &Query) -> std::io::Result<PathBuf> {
    scan_articles(&query, &mut index_reader("data/index")?)
        .and_then(|mut scanner| {
            scanner.pop().ok_or_else(|| io::Error::new(ErrorKind::NotFound, "not found"))
        })
}

// scan_articles walks the index and finds every file path that matches
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
    use crate::index::*;
    use std::convert::TryInto;

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
        assert_eq!(scan_articles(&"@index".try_into().unwrap(), &mut index).unwrap(), result);
    }
}
