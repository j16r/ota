#![feature(plugin)]
#![feature(proc_macro_hygiene, decl_macro)]

extern crate handlebars;
extern crate rand;
extern crate regex;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde_derive;
extern crate serde;
extern crate chrono;

mod templates;
mod articles;
mod error;

use rocket::{get, post, routes};
use rocket::response::{Redirect, content, status::NotFound};
use rocket_contrib::{templates::Template, serve::StaticFiles, json::Json};
use std::io::ErrorKind;
use std::path::PathBuf;
use serde_derive::Serialize;

use crate::articles::{Article, NewArticleRequest, create};
use crate::templates::{render, render_index, render_admin};

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
        ])
        .mount("/static", StaticFiles::from("site"))
        .attach(Template::fairing())
        .launch();
}
