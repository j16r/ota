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
use rocket::request::Form;
use rocket::response::{Redirect, content, status::NotFound};
use rocket_contrib::{templates::Template, serve::StaticFiles};
use std::io::ErrorKind;
use std::path::PathBuf;
use serde_derive::Serialize;

use crate::articles::{Article, NewArticleRequest, create};
use crate::templates::{render, render_index, render_admin};

#[post("/articles", data = "<article>")]
fn create_article(article: Form<NewArticleRequest>) -> Result<content::Html<String>, error::Error> {
    let mut ctx = context();
    if let Err(e) = create(Article::new(&article)) {
        ctx.flash = Some("Error creating article".into());
    }
    let template = render_admin(&ctx)?;
    Ok(content::Html(template))
}

#[get("/")]
fn redirect_to_root() -> Redirect {
    Redirect::to("/index")
}

#[get("/articles/<path..>")]
fn serve_article(path: PathBuf) -> Result<content::Html<String>, NotFound<String>> {
    dbg!("45");
    let article_query = match path.to_str() {
        Some(v) => v,
        None => return Err(NotFound("".to_string()))
    };
    dbg!("50");
    match render(&article_query, &context()) {
        Ok(t) => Ok(content::Html(t)),
        Err(error::Error::IoError(ref e)) if e.kind() == ErrorKind::NotFound => {
            Err(NotFound(format!("article not found for query: {:?}", article_query)))
        },
        Err(e) => {
            dbg!(&e);
            panic!("error serving {:?}", e)
        },
    }
}

#[derive(Serialize)]
struct IndexContext {
    debug: bool,
    flash: Option<String>,
}

fn context() -> IndexContext {
    IndexContext{
        debug: true,
        flash: None,
    }
}

// TODO: Authentication
#[get("/admin")]
fn serve_admin() -> Result<content::Html<String>, error::Error> {
    let template = render_admin(&context())?;
    Ok(content::Html(template))
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
               serve_admin,
        ])
        .mount("/static", StaticFiles::from("site"))
        .attach(Template::fairing())
        .launch();
}
