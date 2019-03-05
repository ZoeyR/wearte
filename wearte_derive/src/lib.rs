//!
//! `wearte_derive` is the core of the crate, and where the procedural macro `derive(Tempalte)` is
//! implemented. `wearte_derive` will have as input a template file and the definition of the struct
//! that goes together. With this, wearte will parse the file and the struct to create an **ast**,
//! and will provide the user's struct the functionality of `fmt` for the template.
//! Derivation is implements `fmt::Display`, superTrait of `Template`, and if activated,
//! `actix_web::Responder` will be implemented on the user's struct.
//! `Template` is defined in the main wearte crate and implements `fmt` in functions like
//! `call`, `call_into_fmt`, `call_into_io`, `mime `, and `size_hint `.
//!
extern crate proc_macro;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate quote;

mod generator;
mod logger;
mod parser;

use proc_macro::TokenStream;
use syn;

use std::collections::BTreeMap;

use wearte_config::{read_config_file, Config};

use crate::generator::{visit_derive, Print};
use crate::logger::log;
use crate::parser::{parse, parse_partials, Node};
use wearte_config::PrintConfig;

#[proc_macro_derive(Template, attributes(template))]
pub fn derive(input: TokenStream) -> TokenStream {
    build(&syn::parse(input).unwrap())
}

#[inline]
fn build(i: &syn::DeriveInput) -> TokenStream {
    let config_toml: &str = &read_config_file();
    let config = &Config::new(config_toml);

    let s = visit_derive(i, &config);

    let mut sources = BTreeMap::new();

    let mut check = vec![(s.path.clone(), s.src.clone())];
    while let Some((path, src)) = check.pop() {
        for n in &parse_partials(&src) {
            match n {
                Node::Partial(_, partial, _) => {
                    check.push(config.get_partial(&path, partial));
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

    if config.print_override == PrintConfig::Ast
        || config.print_override == PrintConfig::All
        || s.print == Print::Ast
        || s.print == Print::All
    {
        eprintln!("{:?}\n", parsed);
    }

    let code = generator::generate(&config, &s, &parsed);
    if config.print_override == PrintConfig::Code
        || config.print_override == PrintConfig::All
        || s.print == Print::Code
        || s.print == Print::All
    {
        log(&code, s.path.to_str().unwrap().to_owned(), &config.debug);
    }

    code.parse().unwrap()
}
