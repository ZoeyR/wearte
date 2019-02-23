use std::{fmt, fs};

use yarte_config::{read_config_file, Config};
pub use yarte_derive::*;
pub use yarte_helpers::{helpers::MarkupDisplay, Error, Result};

// TODO: document
pub trait Template: fmt::Display {
    fn render(&self) -> Result<String> {
        let mut buf = String::with_capacity(Self::size_hint());
        self.render_into(&mut buf)?;
        Ok(buf)
    }
    fn render_into(&self, writer: &mut fmt::Write) -> fmt::Result {
        write!(writer, "{}", self)
    }

    fn mime() -> &'static str
    where
        Self: Sized;

    fn size_hint() -> usize;
}

#[cfg(feature = "with-actix-web")]
pub mod actix_web {
    pub use actix_web::{
        error::ErrorInternalServerError, Error, HttpRequest, HttpResponse, Responder,
    };
}

// TODO:
pub fn rerun_if_templates_changed() {
    let file = read_config_file();
    let mut stack = Config::new(&file).dirs;
    loop {
        if let Some(dir) = stack.pop() {
            for entry in fs::read_dir(dir).expect("valid directory") {
                let entry = entry.expect("valid directory entry");
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    println!("cargo:rerun-if-changed={}", entry.path().to_str().unwrap());
                }
            }
        } else {
            break;
        }
    }
}
