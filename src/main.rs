mod articles;
mod error;
mod index;
mod query;
mod templates;

use std::io::ErrorKind;
use std::path::PathBuf;

use rocket::form::Form;
use rocket::response::{Redirect, status::NotFound, content::Html};
use rocket::{get, post, routes, launch};
use rocket::fs::{FileServer, relative};
use serde_derive::Serialize;

use crate::articles::{Article, NewArticleRequest, create};
use crate::templates::{handlebars, render, render_index, render_admin};

#[post("/articles", data = "<article_request>")]
fn create_article(article_request: Form<NewArticleRequest>) -> Result<Html<String>, error::Error> {
    let mut ctx = IndexContext::default();
    let article = Article::new(&article_request);
    if let Err(_) = create(&article) {
        ctx.flash = Some("Error creating article".into());
    } else {
        ctx.article = Some(article);
    }
    let template = render_admin(&ctx)?;
    Ok(Html(template))
}

#[get("/")]
fn redirect_to_root() -> Redirect {
    Redirect::to("/index")
}

#[get("/articles/<path..>")]
fn serve_article(path: PathBuf) -> Result<Html<String>, NotFound<String>> {
    let article_query = match path.to_str() {
        Some(v) => v,
        None => return Err(NotFound("".to_string()))
    };
    let ctx = IndexContext::default();
    match render(&article_query, &ctx) {
        Ok(t) => Ok(Html(t)),
        Err(error::Error::IoError(ref e)) if e.kind() == ErrorKind::NotFound => {
            Err(NotFound(format!("article not found for query: {:?}", article_query)))
        },
        Err(e) => {
            panic!("error serving {:?}", e)
        },
    }
}

#[derive(Serialize, Debug, Default)]
struct IndexContext {
    debug: bool,
    flash: Option<String>,
    article: Option<Article>,
}

// TODO: Authentication
#[get("/admin")]
fn serve_admin() -> Result<Html<String>, error::Error> {
    let ctx = IndexContext::default();
    let template = render_admin(&ctx)?;
    Ok(Html(template))
}

#[get("/index")]
fn serve_index() -> Result<Html<String>, error::Error> {
    let ctx = IndexContext::default();
    let template = render_index(&ctx)?;
    Ok(Html(template))
}

#[launch]
fn server() -> _ {
    rocket::build()
        .mount("/", routes![
               redirect_to_root,
               create_article,
               serve_article,
               serve_index,
               serve_admin,
        ])
        .mount("/static", FileServer::from(relative!("site")))
        .manage(handlebars())
}
