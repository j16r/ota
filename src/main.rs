#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::path::{Path};
use rocket::response::{NamedFile, Redirect};

#[get("/")]
fn redirect_to_root() -> Redirect {
    Redirect::to("/index.html")
}

#[get("/index.html")]
fn serve_static_index() -> Option<NamedFile> {
    NamedFile::open(Path::new("site/index.html")).ok()
}

fn main() {
    rocket::ignite().mount("/", routes![redirect_to_root, serve_static_index]).launch();
}
