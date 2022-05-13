// use std::fs::File;
// use std::io::prelude::*;
use std::io::Read;
use std::sync::Arc;

use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext,
    RenderError,
};
// use serde::Serialize;
use serde_json::value::Value;

use crate::articles::{lookup_article, lookup_articles};

use crate::App;

// Provides a helper to embed an article in the current template
fn wrapped_article_helper(state: Arc<App>) -> Box<dyn HelperDef + Sync + Send> {
    Box::new(
        move |h: &Helper,
              handlebars: &Handlebars,
              _: &Context,
              _: &mut RenderContext,
              out: &mut dyn Output|
              -> HelperResult {
            let query = h
                .param(0)
                .map(|v| v.value().as_str().unwrap())
                .ok_or_else(|| RenderError::new("requires an article query"))?;
            let mut buffer = String::new();

            let mut index = state.index.lock().unwrap();
            lookup_article(&mut index, query)
                .unwrap()
                .read_to_string(&mut buffer)
                .unwrap();

            out.write(dbg!(handlebars.render_template(&buffer, &()))?.as_ref())?;
            Ok(())
        },
    )
}

// Articles returns all articles that match a pattern, can be used for pagination
fn wrapped_articles_helper(_state: Arc<App>) -> Box<dyn HelperDef + Sync + Send> {
    Box::new(
        move |h: &Helper,
              handlebars: &Handlebars,
              _: &Context,
              _: &mut RenderContext,
              out: &mut dyn Output|
              -> HelperResult {
            let query = h
                .param(0)
                .map(|v| v.value().as_str().unwrap())
                .ok_or_else(|| RenderError::new("requires an article query"))?;

            for article in lookup_articles(query).unwrap().iter_mut() {
                let mut buffer = String::new();
                article.read_to_string(&mut buffer).unwrap();
                out.write(dbg!(handlebars.render_template(&buffer, &()))?.as_ref())?;
            }

            Ok(())
        },
    )
}

handlebars_helper!(hex_helper: |v: i64| format!("0x{:x}", v));

fn flash_helper(
    _: &Helper,
    _: &Handlebars,
    context: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(Value::String(text)) = context.data().get("flash") {
        out.write(r#"<p class="_flash"`>"#)?;
        out.write(text)?;
        out.write(r#"</p>"#)?;
    }
    Ok(())
}

fn admin_article_title_helper(
    _: &Helper,
    _: &Handlebars,
    context: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Value::String(ref text) = context.data()["article"]["title"] {
        out.write(&handlebars::html_escape(text))?;
    }
    Ok(())
}

fn admin_article_body_helper(
    _: &Helper,
    _: &Handlebars,
    context: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Value::String(ref text) = context.data()["article"]["body"] {
        out.write(&handlebars::html_escape(text))?;
    }
    Ok(())
}

pub fn register_helpers(handlebars: &mut Handlebars, state: Arc<App>) {
    handlebars.set_strict_mode(true);

    // User helpers
    handlebars.register_helper("hex", Box::new(hex_helper));
    handlebars.register_helper("article", wrapped_article_helper(state.clone()));
    handlebars.register_helper("articles", wrapped_articles_helper(state));

    // Internal helpers
    handlebars.register_helper("_flash", Box::new(flash_helper));
    handlebars.register_helper("_admin_article_title", Box::new(admin_article_title_helper));
    handlebars.register_helper("_admin_article_body", Box::new(admin_article_body_helper));
}
