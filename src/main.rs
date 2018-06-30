#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate chrono;

use chrono::prelude::*;
use rocket::response::{NamedFile, Redirect};
use rocket_contrib::{Template, Json};
use std::fs::create_dir_all;
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

#[post("/articles", format = "application/json", data = "<article>")]
fn create_article(article: Json<Article>) {
    let now: DateTime<Utc> = Utc::now();
    let path = format!("data/articles/{}/{}", now.format("%Y/%m/%d"), article.name);
    match create_dir_all(path) {
        Ok(()) => json!({"status": "ok"}),
        Err(_) => json!({"status": "error"}),
    };
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

#[get("/<file..>")]
fn serve_static_assets(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("site/").join(file)).ok()
}

fn main() {
    rocket::ignite()
        .mount("/", routes![
                           redirect_to_root,
                           serve_index,
                           serve_static_assets,
                           create_article])
        .attach(Template::fairing())
        .launch();
}
