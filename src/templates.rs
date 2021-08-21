use handlebars::{Handlebars, Helper, HelperResult, Context, RenderContext, Output, handlebars_helper, RenderError};
use std::io::prelude::*;
use serde::Serialize;
use serde_json::value::Value;
use std::fs::File;

use crate::articles::{lookup_article, lookup_articles};
use crate::error::Error;

// Provides a helper to embed an article in the current template
fn article_helper(
    h: &Helper,
    handlebars: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output) -> HelperResult {

    let query = h.param(0).map(|v| v.value().as_str().unwrap())
        .ok_or(RenderError::new("requires an article query"))?;
    let mut buffer = String::new();

    lookup_article(query)?.read_to_string(&mut buffer)?;

    out.write(dbg!(handlebars.render_template(&buffer, &()))?.as_ref())?;
    Ok(())
}

// Articles returns all articles that match a pattern, can be used for pagination
// handlebars_helper!(articles_helper: |expression: str| render_collection(expression, &()));
fn articles_helper(
    h: &Helper,
    handlebars: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output) -> HelperResult {

    let query = h.param(0).map(|v| v.value().as_str().unwrap())
        .ok_or(RenderError::new("requires an article query"))?;

    for article in lookup_articles(&query).unwrap().iter_mut() {
        let mut buffer = String::new();
        article.read_to_string(&mut buffer).unwrap();
        out.write(dbg!(handlebars.render_template(&buffer, &()))?.as_ref())?;
    }

    Ok(())
}

handlebars_helper!(hex_helper: |v: i64| format!("0x{:x}", v));

fn flash_helper(_: &Helper, _: &Handlebars, context: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    if let Some(Value::String(text)) = context.data().get("flash") {
        out.write(r#"<p class="_flash"`>"#)?;
        out.write(text)?;
        out.write(r#"</p>"#)?;
    }
    Ok(())
}

// TODO: needs to safely format html, doesn't need a param
fn admin_article_title_helper(_: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, _: &mut dyn Output) -> HelperResult {
   Ok(())
}
fn admin_article_body_helper(_: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, _: &mut dyn Output) -> HelperResult {
   Ok(())
}

pub fn handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    // User helpers
    handlebars.register_helper("hex", Box::new(hex_helper));
    handlebars.register_helper("article", Box::new(article_helper));
    handlebars.register_helper("articles", Box::new(articles_helper));

    // Internal helpers
    handlebars.register_helper("_flash", Box::new(flash_helper));
    handlebars.register_helper("_admin_article_title", Box::new(admin_article_title_helper));
    handlebars.register_helper("_admin_article_body", Box::new(admin_article_body_helper));

    handlebars
}

pub fn render_admin<T>(context: &T) -> Result<String, Error>
where
    T: Serialize {

    let mut buffer = String::new();
    File::open("templates/admin.hbs")?.read_to_string(&mut buffer)?;

    let handlebars = handlebars();
    handlebars.render_template(&buffer, context).map_err(|e| e.into())
}

pub fn render_index<T>(context: &T) -> Result<String, Error>
where
    T: Serialize {

    let mut buffer = String::new();
    File::open("templates/index.hbs")?.read_to_string(&mut buffer)?;

    let handlebars = handlebars();
    handlebars.render_template(&buffer, context).map_err(|e| e.into())
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

pub fn render_collection<T>(query: &str, context: &T) -> String
where
    T: Serialize {

    let mut buffer = String::new();

    for article in lookup_articles(&query).unwrap().iter_mut() {
        article.read_to_string(&mut buffer).unwrap();
    }

    let handlebars = handlebars();
    handlebars.render_template(&buffer, context).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::IndexContext;
    use crate::templates::*;

    #[test]
    fn test_render_admin() {
        let output = render_admin(&IndexContext::default()).unwrap();
        assert_eq!(r#"<!doctype html>
<html lang="en" style="height: 100%">
  <head>
    <meta charset="utf-8">
    <title>ota</title>
    <meta name="description" content="ota">
    <link href="/static/reset.css" rel="stylesheet" type="text/css"/>
    <link href="/static/intro.css" rel="stylesheet" type="text/css"/>
    <link href="/static/admin.css" rel="stylesheet" type="text/css"/>
  </head>
  <div class="admin">
    
    <form method="post" action="/articles">
        <label for="title">Title:</label>
        <input type="text" name="title" id="title"> </input>
        <br/>
        <textarea rows=50>
          
        </textarea>
        <br/>
        <input type="submit" value="Save"/>
        <input type="hidden" name="properties">
        </input>
        <input type="hidden" name="tags">
        </input>
    </form>
  </div>
  
</html>
"#, output);
    }
}
