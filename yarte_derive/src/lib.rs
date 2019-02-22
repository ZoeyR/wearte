extern crate proc_macro;

#[macro_use]
extern crate nom;
#[macro_use]
extern crate quote;

mod generator;
mod input;
mod parser;

use proc_macro::TokenStream;
use syn;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use yarte_config::{read_config_file, Config};

use crate::input::{Print, Source, TemplateInput};
use crate::parser::{parse, parse_partials, Helper, Node};

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    build_template(&ast).parse().unwrap()
}

fn build_template(ast: &syn::DeriveInput) -> String {
    let config_toml = read_config_file();
    let config = Config::new(&config_toml);

    let input = TemplateInput::new(ast, &config);
    let source = match input.source {
        Source::Source(ref s) => s.clone(),
        Source::Path(_) => get_template_source(&input.path),
    };

    let mut sources = BTreeMap::new();

    let mut check = vec![(input.path.clone(), source)];
    while let Some((path, src)) = check.pop() {
        find_partials(&input, &parse_partials(&src), &path, &mut check);
        sources.insert(path, src);
    }

    let mut parsed = BTreeMap::new();
    for (p, s) in &sources {
        parsed.insert(p, parse(s));
    }

    if input.print == Print::Ast || input.print == Print::All {
        eprintln!("{:?}\n", parsed);
    }

    let code = generator::generate(&input, &parsed);
    if input.print == Print::Code || input.print == Print::All {
        eprintln!("{}", code);
    }

    code
}

fn append_extension(input: &TemplateInput, path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.extension().is_some() {
        p
    } else {
        if let Some(ext) = &input.path.extension() {
            p.with_extension(ext)
        } else {
            p
        }
    }
}

fn find_partials(
    input: &TemplateInput,
    nodes: &[Node],
    path: &PathBuf,
    check: &mut Vec<(PathBuf, String)>,
) {
    for n in nodes {
        match n {
            Node::Partial(_, partial) => {
                let extends = input.config.find_template(
                    append_extension(input, partial).to_str().unwrap(),
                    Some(&path),
                );
                let source = get_template_source(&extends);
                check.push((extends, source));
            }
            Node::Helper(h) => match h {
                Helper::If((_, _, ref ifs), elsif, els) => {
                    find_partials(input, ifs, path, check);
                    for (_, _, b) in elsif {
                        find_partials(input, b, path, check);
                    }
                    if let Some((_, b)) = els {
                        find_partials(input, b, path, check);
                    }
                }
                Helper::Each(_, _, b) | Helper::With(_, _, b) | Helper::Unless(_, _, b) => {
                    find_partials(input, b, path, check)
                }
                _ => unimplemented!(),
            },
            _ => (),
        }
    }
}

fn get_template_source(tpl_path: &Path) -> String {
    match fs::read_to_string(tpl_path) {
        Err(_) => panic!(
            "unable to open template file '{}'",
            tpl_path.to_str().unwrap()
        ),
        Ok(mut source) => {
            if source.ends_with('\n') {
                let _ = source.pop();
            }
            source
        }
    }
}
