use handlebars::Handlebars;
use std::io::prelude::*;
use serde::Serialize;

use articles::{lookup_article, lookup_articles};
use error::Error;

// Provides a helper to embed an article in the current template
handlebars_helper!(article_helper: |name: str| render_inline(name, &()));

// Articles returns all articles that match a pattern, can be used for pagination
//handlebars_helper!(articles_helper: |names: [str]| render_collection(names));
handlebars_helper!(articles_helper: |name: str| format!("article {:?}", name));

handlebars_helper!(hex_helper: |v: i64| format!("0x{:x}", v));

fn handlebars() -> Handlebars {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_helper("hex", Box::new(hex_helper));
    handlebars.register_helper("article", Box::new(article_helper));
    handlebars.register_helper("articles", Box::new(articles_helper));
    handlebars
}

pub fn render_inline<T>(query: &str, context: &T) -> String
where
    T: Serialize {
    render(query, context).unwrap_or("".into())
}

pub fn render<T>(query: &str, context: &T) -> Result<String, Error>
where
    T: Serialize {

    let mut buffer = String::new();

    lookup_article(query)?.read_to_string(&mut buffer)?;

    let handlebars = handlebars();
    handlebars.render_template(&buffer, context).map_err(|e| e.into())
}

pub fn render_collection<T>(query: &str, context: &T) -> Result<String, Error>
where
    T: Serialize {
    let mut buffer = String::new();

    for article in lookup_articles(&query)?.iter_mut() {
        article.read_to_string(&mut buffer)?;
    }

    let handlebars = handlebars();
    handlebars.render_template(&buffer, context).map_err(|e| e.into())
}
