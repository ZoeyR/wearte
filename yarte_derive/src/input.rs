use proc_macro2::TokenStream;
use quote::ToTokens;
use syn;

use std::path::PathBuf;

use yarte_config::Config;

pub struct TemplateInput<'a> {
    pub ast: &'a syn::DeriveInput,
    pub config: &'a Config,
    pub source: Source,
    pub print: Print,
    pub escaping: EscapeMode,
    pub ext: Option<String>,
    pub path: PathBuf,
}

impl<'a> TemplateInput<'a> {
    /// Extract the template metadata from the `DeriveInput` structure. This
    /// mostly recovers the data for the `TemplateInput` fields from the
    /// `template()` attribute list fields; it also finds the of the `_parent`
    /// field, if any.
    pub fn new<'n>(ast: &'n syn::DeriveInput, config: &'n Config) -> TemplateInput<'n> {
        // Check that an attribute called `template()` exists and that it is
        // the proper type (list).
        let mut meta = None;
        for attr in &ast.attrs {
            match attr.interpret_meta() {
                Some(m) => {
                    if m.name() == "template" {
                        meta = Some(m)
                    }
                }
                None => {
                    let mut tokens = TokenStream::new();
                    attr.to_tokens(&mut tokens);
                    panic!("unable to interpret attribute: {}", tokens)
                }
            }
        }

        let meta_list = match meta.expect("no attribute 'template' found") {
            syn::Meta::List(inner) => inner,
            _ => panic!("attribute 'template' has incorrect type"),
        };

        // Loop over the meta attributes and find everything that we
        // understand. Raise panics if something is not right.
        // `source` contains an enum that can represent `path` or `source`.
        let mut source = None;
        let mut print = Print::None;
        let mut escaping = None;
        let mut ext = None;
        for nm_item in meta_list.nested {
            if let syn::NestedMeta::Meta(ref item) = nm_item {
                if let syn::Meta::NameValue(ref pair) = item {
                    match pair.ident.to_string().as_ref() {
                        "path" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                if source.is_some() {
                                    panic!("must specify 'source' or 'path', not both");
                                }
                                source = Some(Source::Path(s.value()));
                            } else {
                                panic!("template path must be string literal");
                            }
                        }
                        "source" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                if source.is_some() {
                                    panic!("must specify 'source' or 'path', not both");
                                }
                                source = Some(Source::Source(s.value()));
                            } else {
                                panic!("template source must be string literal");
                            }
                        }
                        "print" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                print = s.value().into();
                            } else {
                                panic!("print value must be string literal");
                            }
                        }
                        "escape" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                escaping = Some(s.value().into());
                            } else {
                                panic!("escape value must be string literal");
                            }
                        }
                        "ext" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                ext = Some(s.value());
                            } else {
                                panic!("ext value must be string literal");
                            }
                        }
                        attr => panic!("unsupported annotation key '{}' found", attr),
                    }
                }
            }
        }

        // Validate the `source` and `ext` value together, since they are
        // related. In case `source` was used instead of `path`, the value
        // of `ext` is merged into a synthetic `path` value here.
        let source = source.expect("template path or source not found in attributes");
        let path = match (&source, &ext) {
            (&Source::Path(ref path), None) => config.find_template(path, None),
            (&Source::Source(_), Some(ext)) => PathBuf::from(format!("{}.{}", ast.ident, ext)),
            (&Source::Path(_), Some(_)) => {
                panic!("'ext' attribute cannot be used with 'path' attribute")
            }
            (&Source::Source(_), None) => {
                panic!("must include 'ext' attribute when using 'source' attribute")
            }
        };

        TemplateInput {
            ast,
            config,
            source,
            print,
            escaping: escaping.unwrap_or_else(|| EscapeMode::from_path(&path)),
            ext,
            path,
        }
    }
}

pub enum Source {
    Path(String),
    Source(String),
}

#[derive(PartialEq)]
pub enum EscapeMode {
    Html,
    None,
}

impl From<String> for EscapeMode {
    fn from(s: String) -> EscapeMode {
        use self::EscapeMode::*;
        match s.as_ref() {
            "html" => Html,
            "none" => None,
            v => panic!("invalid value for escape option: {}", v),
        }
    }
}

impl EscapeMode {
    fn from_path(path: &PathBuf) -> EscapeMode {
        let extension = path.extension().map(|s| s.to_str().unwrap()).unwrap_or("");
        if HTML_EXTENSIONS.contains(&extension) {
            EscapeMode::Html
        } else {
            EscapeMode::None
        }
    }
}

#[derive(PartialEq)]
pub enum Print {
    All,
    Ast,
    Code,
    None,
}

impl From<String> for Print {
    fn from(s: String) -> Print {
        use self::Print::*;
        match s.as_ref() {
            "all" => All,
            "ast" => Ast,
            "code" => Code,
            "none" => None,
            v => panic!("invalid value for print option: {}", v),
        }
    }
}

const HTML_EXTENSIONS: [&str; 3] = ["html", "htm", "xml"];
