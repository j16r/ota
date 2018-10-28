#[macro_export]
macro_rules! impl_from_error {
    ($from:path, $to:tt::$ctor:tt) => {
        impl From<$from> for $to {
            fn from(e: $from) -> Self {
                $to::$ctor(e)
            }
        }
    };
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    TemplateError(handlebars::TemplateError),
    TemplateRenderError(handlebars::TemplateRenderError),
}

impl_from_error!(std::io::Error, Error::IoError);
impl_from_error!(handlebars::TemplateError, Error::TemplateError);
impl_from_error!(handlebars::TemplateRenderError, Error::TemplateRenderError);
