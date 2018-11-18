#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]
#![feature(extern_prelude)]
#![feature(result_map_or_else)]

#[macro_use] extern crate handlebars;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate chrono;

extern crate regex;

mod templates;
mod articles;
mod error;

use rocket::response::{NamedFile, Redirect, content, status::NotFound};
use rocket_contrib::Template;
use std::path::{Path, PathBuf};
use rocket_contrib::Json;
use std::io::ErrorKind;

use articles::{Article, create};
use templates::render;

const CLOUDFLARE_CDN_URL: &'static str = "https://cdnjs.cloudflare.com/ajax/libs/";

#[post("/articles", format = "application/json", data = "<article>")]
fn create_article(article: Json<Article>) -> std::io::Result<()> {
    create(Article{
        name: article.name.clone(),
        body: article.body.clone()
    })
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
    nocdn: bool,
    cdn_url: &'static str,
}

fn context() -> IndexContext {
    IndexContext{
        debug: true,
        nocdn: true,
        cdn_url: CLOUDFLARE_CDN_URL,
    }
}

#[get("/static/<file..>")]
fn serve_static_assets(file: PathBuf) -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new("site/").join(file))
}

#[get("/index")]
fn serve_index() -> content::Html<String> {
    let template = render("@index", &context()).unwrap();
    content::Html(template)
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
