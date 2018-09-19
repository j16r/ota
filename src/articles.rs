use chrono::prelude::*;
use regex::Regex;
use std::borrow::Cow;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::prelude::*;
use std::path::{Path};

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
        article_file_name(&article.name).into_owned());
    let path = Path::new(&path_str);
    let dir = path.parent().unwrap();
    match create_dir_all(&dir) {
        Ok(()) => json!({"status": "ok"}),
        Err(_) => json!({"status": "error"}),
    };
    let mut file = File::create(path)?;
    file.write_all(article.body.as_bytes())?;
    Ok(())
}

//fn lookup_article(pattern: String) -> String {
    //"year:2018"
    //"month:>1"
    //""
//}

#[cfg(test)]
mod tests {
    use articles::article_file_name;

    #[test]
    fn test_article_file_name() {
        assert!(article_file_name("abcd") == "abcd");
        assert!(article_file_name("a!@#bcd") == "a___bcd");
        assert!(article_file_name("") == "");
    }
}
