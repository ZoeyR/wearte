// TODO: document

use std::{fmt, io};

pub use wearte_derive::Template;
pub use wearte_helpers::{helpers::MarkupAsStr, Error, Result};

pub mod rerun;

// TODO: document
pub trait Template: fmt::Display {
    // esto crea un string fmt sobre Template y te el String
    fn call(&self) -> Result<String> {
        let mut buf = String::with_capacity(Self::size_hint());
        self.call_into_fmt(&mut buf).map(|_| buf)
    }

    // esto es para un tipo string vect o algo asi
    fn call_into_fmt(&self, writer: &mut fmt::Write) -> fmt::Result {
        write!(writer, "{}", self)
    }

    // esto es para un archivo, stdout
    fn call_into_io(&self, writer: &mut io::Write) -> io::Result<()> {
        write!(writer, "{}", self)
    }

    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types
    fn mime() -> &'static str
    where
        Self: Sized;

    // heuristica de allocation
    fn size_hint() -> usize;
}

#[cfg(feature = "with-actix-web")]
pub mod actix_web {
    pub use actix_web::{
        error::ErrorInternalServerError, Error, HttpRequest, HttpResponse, Responder,
    };
}
