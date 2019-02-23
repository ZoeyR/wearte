extern crate proc_macro;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate quote;

mod generator;
mod parser;

use proc_macro::TokenStream;
use syn;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use yarte_config::{read_config_file, Config};

use crate::generator::{visit_derive, Print};
use crate::parser::{parse, parse_partials, Node};

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let i: syn::DeriveInput = syn::parse(input).unwrap();
    build_template(&i).parse().unwrap()
}

fn build_template(i: &syn::DeriveInput) -> String {
    let config_toml: &str = &read_config_file();
    let config = &Config::new(config_toml);

    let s = visit_derive(i, &config);

    let mut sources = BTreeMap::new();

    let mut check = vec![(s.path.clone(), s.source.clone())];
    while let Some((path, src)) = check.pop() {
        for n in &parse_partials(&src) {
            match n {
                Node::Partial(_, partial) => {
                    let extends = config.find_template(
                        append_extension(&s.path, partial).to_str().unwrap(),
                        Some(&path),
                    );
                    let source = get_source(&extends);
                    check.push((extends, source));
                }
                _ => unreachable!(),
            }
        }
        sources.insert(path, src);
    }

    let mut parsed = BTreeMap::new();
    for (p, src) in &sources {
        parsed.insert(p, parse(src));
    }

    if s.print == Print::Ast || s.print == Print::All {
        eprintln!("{:?}\n", parsed);
    }

    let code = generator::generate(&s, &parsed);
    if s.print == Print::Code || s.print == Print::All {
        eprintln!("{}", code);
    }

    code
}

fn append_extension(parent: &PathBuf, path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.extension().is_some() {
        p
    } else {
        if let Some(ext) = parent.extension() {
            p.with_extension(ext)
        } else {
            p
        }
    }
}

pub(crate) fn get_source(path: &Path) -> String {
    match fs::read_to_string(path) {
        Err(_) => panic!("unable to open template file '{:?}'", path),
        Ok(mut source) => match source
            .as_bytes()
            .iter()
            .enumerate()
            .rev()
            .find_map(|(j, x)| {
                if x.is_ascii_whitespace() {
                    None
                } else {
                    Some(j)
                }
            }) {
            Some(j) => {
                source.drain(j + 1..);
                source
            }
            None => source,
        },
    }
}
