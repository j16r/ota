mod articles;
mod error;
mod index;
mod query;
mod templates;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::response::{status::NotFound, Redirect};
use rocket::{get, launch, post, routes, State};
use rocket_dyn_templates::serde::Serialize;
use rocket_dyn_templates::Template;

use crate::articles::{Article, NewArticleRequest};
use crate::index::{local::Local, Index};
use crate::templates::register_helpers;
use crate::query::Query;

#[derive(Clone)]
pub struct App {
    index: Arc<Mutex<Box<dyn Index>>>,
}

#[post("/articles", data = "<article_request>")]
fn create_article(
    index_state: &State<Arc<App>>,
    article_request: Form<NewArticleRequest>,
) -> Result<Template, error::Error> {
    let mut ctx = IndexContext::default();
    let article = Article::new(&article_request);
    if index_state.index.lock().unwrap().update(&article).is_err() {
        ctx.flash = Some("Error creating article".into());
    } else {
        ctx.article = Some(article);
    }
    Ok(Template::render("admin", ctx))
}

#[get("/")]
fn redirect_to_root() -> Redirect {
    Redirect::to("/index")
}

#[get("/articles/<path..>")]
fn serve_article(
    index_state: &State<Arc<App>>,
    path: PathBuf,
) -> Result<Template, NotFound<String>> {
    let query: Query = match path.to_str().unwrap().try_into() {
        Ok(v) => v,
        _ => return Err(NotFound("".to_string())),
    };
    let ctx = IndexContext::default();
    match index_state.index.lock().unwrap().first(&query) {
        Ok(a) => Ok(Template::render(a.article().body, ctx)),
        Err(_e) => {
        // Err(e) if e == Error::ArticleNotFound => {
            return Err(NotFound("".to_string()));
        },
        // _ => todo!("generic error / 500"),
    }
}

#[derive(Serialize, Debug, Default)]
struct IndexContext {
    debug: bool,
    flash: Option<String>,
    article: Option<Article>,
}

#[get("/articles")]
fn serve_articles() -> Template {
    let ctx = IndexContext::default();
    Template::render("articles/index", ctx)
}

// TODO: Authentication
#[get("/admin")]
fn serve_admin() -> Template {
    let ctx = IndexContext::default();
    Template::render("admin", ctx)
}

#[get("/index")]
fn serve_index() -> Template {
    let ctx = IndexContext::default();
    Template::render("index", ctx)
}

#[launch]
fn server() -> _ {
    let index = Local::new("data").unwrap();
    let state = Arc::new(App {
        index: Arc::new(Mutex::new(Box::new(index))),
    });

    rocket::build()
        .mount(
            "/",
            routes![
                redirect_to_root,
                create_article,
                serve_article,
                serve_articles,
                serve_index,
                serve_admin,
            ],
        )
        .mount("/static", FileServer::from(relative!("site")))
        .manage(state.clone())
        .attach(Template::custom(move |engines| {
            let inner_state = Arc::clone(&state);
            register_helpers(&mut engines.handlebars, inner_state);
        }))
}
