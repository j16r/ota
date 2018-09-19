use handlebars::Handlebars;
use std::io::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};

// Provides a helper to embed an article in the current template
//handlebars_helper!(article: |article: PathBuf| render_article(article));

// Articles returns all articles that match a pattern, can be used for pagination
//handlebars_helper!(articles: |article: PathBuf| render_article(article));

handlebars_helper!(hex: |v: i64| format!("0x{:x}", v));

fn handlebars() -> Handlebars {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_helper("hex", Box::new(hex));
    handlebars
}

pub fn render_article(article: PathBuf) -> String {
    let path = Path::new("data/articles/").join(article);

    let mut buffer = String::new();
    
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut buffer).unwrap();

    let handlebars = handlebars();
    handlebars.render_template(&buffer, &json!({})).unwrap()
}
