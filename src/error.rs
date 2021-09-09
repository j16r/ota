use rocket::request::Request;
use rocket::response::Responder;
use rocket::{http::Status, Response};
use std::io::ErrorKind;

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
    RenderError(handlebars::RenderError),
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> Result<Response<'static>, Status> {
        let mut response = Response::build();
        match self {
            Error::IoError(ref e) if e.kind() == ErrorKind::NotFound => {
                response.status(Status::NotFound);
            }
            _ => return Err(Status::InternalServerError),
        };
        response.ok()
    }
}

impl_from_error!(std::io::Error, Error::IoError);
impl_from_error!(handlebars::TemplateError, Error::TemplateError);
impl_from_error!(handlebars::RenderError, Error::RenderError);
