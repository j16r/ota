#![feature(plugin)]
#![plugin(rocket_codegen)]
#![cfg_attr(feature="clippy", feature(plugin))]

extern crate rocket;

use std::path::{Path, PathBuf};
use rocket::response::{NamedFile, Redirect};

#[get("/")]
fn redirect_to_root() -> Redirect {
    Redirect::to("/index.html")
}

#[get("/<file..>")]
fn serve_static_assets(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("site/").join(file)).ok()
}

fn main() {
    rocket::ignite().mount("/", routes![
                           redirect_to_root,
                           serve_static_assets]).launch();
}
