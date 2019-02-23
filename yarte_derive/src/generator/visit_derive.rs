use syn;
use syn::visit::Visit;

use std::path::PathBuf;

use yarte_config::Config;

use crate::get_template_source;

pub(crate) fn visit_derive<'a>(i: &'a syn::DeriveInput, config: &'a Config) -> Struct<'a> {
    StructBuilder::default().build(i, config)
}

#[derive(Debug)]
pub(crate) struct Struct<'a> {
    pub source: String,
    pub path: PathBuf,
    pub print: Print,
    pub escaping: EscapeMode,
    ident: &'a syn::Ident,
    generics: &'a syn::Generics,
}

impl<'a> Struct<'a> {
    pub fn implement_head(&self, t: &str) -> String {
        let (impl_generics, orig_ty_generics, where_clause) = self.generics.split_for_impl();

        format!(
            "{} {} for {}{} {{",
            quote!(impl#impl_generics),
            t,
            self.ident,
            quote!(#orig_ty_generics #where_clause)
        )
    }
}

struct StructBuilder {
    source: Option<String>,
    print: Option<String>,
    escaping: Option<String>,
    path: Option<String>,
    ext: Option<String>,
    // TODO: visit struct for wrapper resolution
}

impl Default for StructBuilder {
    fn default() -> Self {
        StructBuilder {
            source: None,
            print: None,
            escaping: None,
            path: None,
            ext: None,
        }
    }
}

impl StructBuilder {
    fn build<'n>(
        mut self,
        syn::DeriveInput {
            attrs,
            ident,
            generics,
            ..
        }: &'n syn::DeriveInput,
        config: &'n Config,
    ) -> Struct<'n> {
        for it in attrs {
            self.visit_attribute(it)
        }

        let (source, path) = match (self.source, self.ext) {
            (Some(src), Some(ext)) => (src, PathBuf::from(format!("{}.{}", ident, ext))),
            (None, None) => {
                let path = config.find_template(&self.path.expect("some valid path"), None);
                let source = get_template_source(path.as_path());
                (source, path)
            }
            (None, Some(_)) => panic!("'ext' attribute cannot be used with 'path' attribute"),
            (Some(_), None) => panic!("must include 'ext' attribute when using 'source' attribute"),
        };

        let escaping = self.escaping.map_or_else(
            || {
                if HTML_EXTENSIONS.contains(&path.extension().map_or("", |s| s.to_str().unwrap())) {
                    EscapeMode::Html
                } else {
                    EscapeMode::None
                }
            },
            |s| s.into(),
        );

        Struct {
            source,
            path,
            print: self.print.into(),
            escaping,
            generics,
            ident,
        }
    }
}

// TODO: extend
impl<'a> Visit<'a> for StructBuilder {
    fn visit_attribute(&mut self, i: &'a syn::Attribute) {
        match i.parse_meta() {
            Ok(m) => self.visit_meta(&m),
            Err(_) => (),
        }
    }

    fn visit_meta_list(&mut self, syn::MetaList { ident, nested, .. }: &'a syn::MetaList) {
        if ATTRIBUTES.contains(&ident.to_string().as_ref()) {
            use syn::punctuated::Punctuated;
            for el in Punctuated::pairs(nested) {
                let it = el.value();
                self.visit_nested_meta(it)
            }
        } else {
            panic!("not valid template attribute: {}", ident);
        }
    }

    fn visit_meta_name_value(
        &mut self,
        syn::MetaNameValue { ident, lit, .. }: &'a syn::MetaNameValue,
    ) {
        match ident.to_string().as_ref() {
            "path" => {
                if let syn::Lit::Str(ref s) = lit {
                    if self.source.is_some() {
                        panic!("must specify 'source' or 'path', not both");
                    }
                    self.path = Some(s.value());
                } else {
                    panic!("template path must be string literal");
                }
            }
            "source" => {
                if let syn::Lit::Str(ref s) = lit {
                    if self.path.is_some() {
                        panic!("must specify 'source' or 'path', not both");
                    }
                    self.source = Some(s.value());
                } else {
                    panic!("template source must be string literal");
                }
            }
            "print" => {
                if let syn::Lit::Str(ref s) = lit {
                    self.print = Some(s.value().into());
                } else {
                    panic!("print value must be string literal");
                }
            }
            "escape" => {
                if let syn::Lit::Str(ref s) = lit {
                    self.escaping = Some(s.value().into());
                } else {
                    panic!("escape value must be string literal");
                }
            }
            "ext" => {
                if let syn::Lit::Str(ref s) = lit {
                    self.ext = Some(s.value());
                } else {
                    panic!("ext value must be string literal");
                }
            }
            attr => panic!("unsupported annotation key '{}' found", attr),
        }
    }
}

#[derive(PartialEq, Debug)]
pub(crate) enum EscapeMode {
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

#[derive(PartialEq, Debug)]
pub(crate) enum Print {
    All,
    Ast,
    Code,
    None,
}

impl From<Option<String>> for Print {
    fn from(s: Option<String>) -> Print {
        match s {
            Some(s) => match s.as_ref() {
                "all" => Print::All,
                "ast" => Print::Ast,
                "code" => Print::Code,
                "none" => Print::None,
                v => panic!("invalid value for print option: {}", v),
            },
            None => Print::None,
        }
    }
}

static HTML_EXTENSIONS: [&str; 3] = ["html", "htm", "xml"];
static ATTRIBUTES: [&str; 2] = ["derive", "template"];

#[cfg(test)]
mod test {
    use super::*;
    use syn::parse_str;

    #[test]
    #[should_panic]
    fn test_panic() {
        let src = r#"
            #[derive(Template)]
            #[template(path = "no-exist.html")]
            struct Test;
        "#;
        let i = parse_str::<syn::DeriveInput>(src).unwrap();
        let config = Config::new("");
        let _ = visit_derive(&i, &config);
    }

    #[test]
    fn test() {
        let src = r#"
            #[derive(Template)]
            #[template(source = "", ext = "txt", print = "code")]
            struct Test;
        "#;
        let i = parse_str::<syn::DeriveInput>(src).unwrap();
        let config = Config::new("");
        let s = visit_derive(&i, &config);
        assert_eq!(s.source, "");
        assert_eq!(s.path, PathBuf::from("Test.txt"));
        assert_eq!(s.print, Print::Code);
        assert_eq!(s.escaping, EscapeMode::None);
    }
}
