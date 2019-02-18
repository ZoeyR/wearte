use std::{
    fmt,
    fs::{self, DirEntry},
    io,
    path::Path,
};

pub use yarte_helpers::{helpers, helpers::MarkupDisplay, Error, Result};

/// Main `Template` trait; implementations are generally derived
pub trait Template: fmt::Display {
    /// Helper method which allocates a new `String` and renders into it
    fn render(&self) -> Result<String> {
        let mut buf = String::new();
        self.render_into(&mut buf)?;
        Ok(buf)
    }
    /// Renders the template to the given `writer` buffer
    fn render_into(&self, writer: &mut fmt::Write) -> fmt::Result {
        write!(writer, "{}", self)
    }
    /// Helper function to inspect the template's extension
    fn extension() -> Option<&'static str>
    where
        Self: Sized;
}

pub use yarte_config::{read_config_file, Config};
pub use yarte_derive::*;
pub use yarte_helpers::*;

#[cfg(feature = "with-actix-web")]
pub mod actix_web {
    extern crate actix_web;
    extern crate mime_guess;

    // actix_web technically has this as a pub fn in later versions, fs::file_extension_to_mime.
    // Older versions that don't have it exposed are easier this way. If ext is empty or no
    // associated type was found, then this returns `application/octet-stream`, in line with how
    // actix_web handles it in newer releases.
    pub use self::actix_web::{
        error::ErrorInternalServerError, Error, HttpRequest, HttpResponse, Responder,
    };
    use self::mime_guess::get_mime_type;

    pub fn respond(t: &super::Template, ext: &str) -> Result<HttpResponse, Error> {
        let rsp = t
            .render()
            .map_err(|_| ErrorInternalServerError("Template parsing error"))?;
        let ctype: &str = &get_mime_type(ext).to_string();
        Ok(HttpResponse::Ok().content_type(ctype).body(rsp))
    }
}

fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

/// Build script helper to rebuild crates if contained templates have changed
///
/// Iterates over all files in the template directories and writes a
/// `cargo:rerun-if-changed=` line for each of them to stdout.
///
/// This helper method can be used in build scripts (`build.rs`) in crates
/// that have templates, to make sure the crate gets rebuilt when template
/// source code changes.
pub fn rerun_if_templates_changed() {
    let file = read_config_file();
    for template_dir in &Config::new(&file).dirs {
        visit_dirs(template_dir, &|e: &DirEntry| {
            println!("cargo:rerun-if-changed={}", e.path().to_str().unwrap());
        })
        .unwrap();
    }
}
