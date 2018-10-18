#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]
#![feature(extern_prelude)]

#[macro_use] extern crate handlebars;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate chrono;

extern crate regex;

mod templates;
mod articles;

use rocket::response::{NamedFile, Redirect, content};
use rocket_contrib::Template;
use std::path::{Path, PathBuf};
use rocket_contrib::Json;

use articles::{Article, create};
use templates::render;

const CLOUDFLARE_CDN_URL: &'static str = "https://cdnjs.cloudflare.com/ajax/libs/";

#[post("/articles", format = "application/json", data = "<article>")]
fn create_article(article: Json<Article>) -> std::io::Result<()> {
    create(Article{name: article.name.clone(), body: article.body.clone()})
}

#[get("/")]
fn redirect_to_root() -> Redirect {
    Redirect::to("/index.html")
}

#[get("/articles/<article..>")]
fn serve_article(article: PathBuf) -> Option<content::Html<String>> {
    Some(content::Html(render(article.to_str()?, &context())))
}

#[derive(Serialize)]
struct IndexContext {
    debug: bool,
    nocdn: bool,
    cdn_url: &'static str,
}

#[get("/index.html")]
fn serve_index() -> Option<content::Html<String>> {
    Some(content::Html(render("index", &context())))
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
