#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket_contrib;
extern crate rocket;
#[macro_use] extern crate serde_derive;

use std::path::{Path, PathBuf};
use rocket::response::{NamedFile, Redirect};
use rocket_contrib::Template;

const CLOUDFLARE_CDN_URL: &'static str = "https://cdnjs.cloudflare.com/ajax/libs/";

#[get("/")]
fn redirect_to_root() -> Redirect {
    Redirect::to("/index.html")
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
                           serve_static_assets])
        .attach(Template::fairing())
        .launch();
}
