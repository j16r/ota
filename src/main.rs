#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]

#[macro_use] extern crate handlebars;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate chrono;

extern crate regex;

use chrono::prelude::*;
use handlebars::Handlebars;
use regex::Regex;
use rocket::response::{NamedFile, Redirect};
use rocket_contrib::{Template, Json};
use std::borrow::Cow;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

const CLOUDFLARE_CDN_URL: &'static str = "https://cdnjs.cloudflare.com/ajax/libs/";

#[get("/")]
fn redirect_to_root() -> Redirect {
    Redirect::to("/index.html")
}

#[derive(Serialize, Deserialize)]
struct Article {
    name: String,
    body: String,
}

fn article_file_name<'a>(path: &'a str) -> Cow<'a, str> {
    let article_filename_regex = Regex::new(r"[^A-Za-z]").unwrap();
    article_filename_regex.replace_all(path, "_")
}

#[post("/articles", format = "application/json", data = "<article>")]
fn create_article(article: Json<Article>) -> std::io::Result<()> {
    let now: DateTime<Utc> = Utc::now();
    // TODO: path is full we wanna clip of the article here
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

// Provides a helper to embed an article in the current template
//handlebars_helper!(article: |article: PathBuf| render_article(article));

// Articles returns all articles that match a pattern, can be used for pagination
//handlebars_helper!(articles: |article: PathBuf| render_article(article));

handlebars_helper!(hex: |v: i64| format!("0x{:x}", v));

fn render_article(article: PathBuf) -> String {
    let path = Path::new("data/articles/").join(article);

    let mut handlebars = Handlebars::new();
    let mut buffer = String::new();

    handlebars.register_helper("hex", Box::new(hex));
    
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut buffer).unwrap();

    handlebars.render_template(&buffer, &json!({})).unwrap()
}

//fn lookup_article(pattern: String) -> String {
    //"year:2018"
    //"month:>1"
    //""
//}

#[get("/articles/<article..>")]
fn serve_article(article: PathBuf) -> String {
    render_article(article)
}

#[derive(Serialize)]
struct IndexContext {
    debug: bool,
    nocdn: bool,
    cdn_url: &'static str,
}

#[get("/index.html")]
fn serve_index() -> Template {
    let context = IndexContext{
        debug: true,
        nocdn: true,
        cdn_url: CLOUDFLARE_CDN_URL,
    };
    Template::render("index", &context)
}

#[get("/static/<file..>")]
fn serve_static_assets(file: PathBuf) -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new("site/").join(file))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![
                           redirect_to_root,
                           serve_index,
                           serve_article,
                           serve_static_assets,
                           create_article])
        .attach(Template::fairing())
        .launch();
}

#[cfg(test)]
mod tests {
    use article_file_name;

    #[test]
    fn test_article_file_name() {
        assert!(article_file_name("abcd") == "abcd");
        assert!(article_file_name("a!@#bcd") == "a___bcd");
        assert!(article_file_name("") == "");
    }
}
