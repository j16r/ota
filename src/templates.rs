use handlebars::Handlebars;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use serde::Serialize;

// Provides a helper to embed an article in the current template
//handlebars_helper!(article: |article: PathBuf| render(article));
handlebars_helper!(article: |name: str| render(name, &()));

// Articles returns all articles that match a pattern, can be used for pagination
//handlebars_helper!(articles: |names: [str]| render_collection(names));

handlebars_helper!(hex: |v: i64| format!("0x{:x}", v));

fn handlebars() -> Handlebars {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_helper("hex", Box::new(hex));
    handlebars.register_helper("article", Box::new(article));
    //handlebars.register_helper("articles", Box::new(articles));
    handlebars
}

pub fn render<T>(name: &str, context: &T) -> String 
where
    T: Serialize {
    let path = Path::new("data/articles/").join(name);

    let mut buffer = String::new();
    
    let mut file = File::open(path).or_else(|_| {
        let fallback_path = Path::new("templates/").join(format!("{}.hbs", name));
        File::open(fallback_path)
    }).unwrap();
    file.read_to_string(&mut buffer).unwrap();

    let handlebars = handlebars();
    handlebars.render_template(&buffer, context).unwrap()
}

//pub fn render_collection(name: [str]) -> String {
//}
