#![feature(plugin)]
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate handlebars;
extern crate rand;
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate chrono;

extern crate regex;

mod templates;
mod articles;
mod error;

use rocket::response::{NamedFile, Redirect, content, status::NotFound};
use rocket_contrib::templates::Template;
use std::path::{Path, PathBuf};
use rocket_contrib::json::Json;
use std::io::ErrorKind;

use crate::articles::{Article, NewArticleRequest, create};
use crate::templates::{render, render_index};

#[post("/articles", format = "application/json", data = "<article>")]
fn create_article(article: Json<NewArticleRequest>) -> std::io::Result<()> {
    create(Article::new(&article))
}

#[get("/")]
fn redirect_to_root() -> Redirect {
    Redirect::to("/index")
}

#[get("/articles/<path..>")]
fn serve_article(path: PathBuf) -> Result<content::Html<String>, NotFound<String>> {
    let article_query = match path.to_str() {
        Some(v) => v,
        None => return Err(NotFound("".to_string()))
    };
    match render(&article_query, &context()) {
        Ok(t) => Ok(content::Html(t)),
        Err(error::Error::IoError(ref e)) if e.kind() == ErrorKind::NotFound => {
            Err(NotFound(format!("article not found for query: {:?}", article_query)))
        },
        Err(e) => panic!("error serving {:?}", e)
    }
}

#[derive(Serialize)]
struct IndexContext {
    debug: bool,
}

fn context() -> IndexContext {
    IndexContext{
        debug: true,
    }
}

#[get("/static/<file..>")]
fn serve_static_assets(file: PathBuf) -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new("site/").join(file))
}

#[get("/index")]
fn serve_index() -> Result<content::Html<String>, error::Error> {
    let template = render_index(&context())?;
    Ok(content::Html(template))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![
               redirect_to_root,
               create_article,
               serve_article,
               serve_index,
               serve_static_assets,
        ])
        .attach(Template::fairing())
        .launch();
}
