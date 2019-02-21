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
        let mut buf = String::with_capacity(Self::size_hint());
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

    /// Helper function to inspect the template's mime
    fn mime() -> &'static str
    where
        Self: Sized;

    fn size_hint() -> usize;
}

pub use yarte_config::{read_config_file, Config};
pub use yarte_derive::*;
pub use yarte_helpers::*;

#[cfg(feature = "with-actix-web")]
pub mod actix_web {
    pub use actix_web::{
        error::ErrorInternalServerError, Error, HttpRequest, HttpResponse, Responder,
    };
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
