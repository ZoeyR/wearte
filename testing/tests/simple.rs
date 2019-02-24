use wearte::Template;

use std::collections::HashMap;

#[derive(Template)]
#[template(path = "simple.html")]
struct VariablesTemplate<'a> {
    strvar: &'a str,
    num: i64,
    i18n: String,
}

#[test]
fn test_variables() {
    let s = VariablesTemplate {
        strvar: "foo",
        num: 42,
        i18n: "Iñtërnâtiônàlizætiøn".to_string(),
    };
    assert_eq!(
        s.call().unwrap(),
        "hello world, foo\n\
         with number: 42\n\
         Iñtërnâtiônàlizætiøn is important\n\
         in vars too: Iñtërnâtiônàlizætiøn"
    );
    assert_eq!(VariablesTemplate::mime(), "text/html");
}

#[derive(Template)]
#[template(path = "hello.html")]
struct EscapeTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_escape() {
    let s = EscapeTemplate { name: "<>&\"'/" };

    assert_eq!(s.call().unwrap(), "Hello, &lt;&gt;&amp;&quot;&#x27;&#x2f;!");
}

#[derive(Template)]
#[template(path = "simple-no-escape.txt")]
struct VariablesTemplateNoEscape<'a> {
    strvar: &'a str,
    num: i64,
    i18n: String,
}

#[test]
fn test_variables_no_escape() {
    let s = VariablesTemplateNoEscape {
        strvar: "foo",
        num: 42,
        i18n: "Iñtërnâtiônàlizætiøn".to_string(),
    };
    assert_eq!(
        s.call().unwrap(),
        "hello world, foo\n\
         with number: 42\n\
         Iñtërnâtiônàlizætiøn is important\n\
         in vars too: Iñtërnâtiônàlizætiøn"
    );
}

#[derive(Template)]
#[template(path = "if.html")]
struct IfTemplate {
    cond: bool,
}

#[test]
fn test_if() {
    let s = IfTemplate { cond: true };
    assert_eq!(s.call().unwrap(), "true");
}

#[derive(Template)]
#[template(path = "else.html")]
struct ElseTemplate {
    cond: bool,
}

#[test]
fn test_else_false() {
    let s = ElseTemplate { cond: false };
    assert_eq!(s.call().unwrap(), "     \n    false\n");
}

#[test]
fn test_else_true() {
    let s = ElseTemplate { cond: true };
    assert_eq!(s.call().unwrap(), "     \n true");
}

#[derive(Template)]
#[template(path = "else-if.html")]
struct ElseIfTemplate {
    cond: bool,
    check: bool,
}

#[test]
fn test_else_if() {
    let s = ElseIfTemplate {
        cond: false,
        check: true,
    };
    assert_eq!(s.call().unwrap(), " checked ");
}

#[derive(Template)]
#[template(source = "  {{! foo !}}", ext = "txt")]
struct CommentTemplate {}

#[test]
fn test_comment() {
    let t = CommentTemplate {};
    assert_eq!(t.call().unwrap(), "");
}

#[derive(Template)]
#[template(source = "{{# if !foo }}Hello{{/if }}", ext = "txt")]
struct NegationTemplate {
    foo: bool,
}

#[test]
fn test_negation() {
    let t = NegationTemplate { foo: false };
    assert_eq!(t.call().unwrap(), "Hello");
}

#[derive(Template)]
#[template(source = "{{# if foo > -2 }}Hello{{/ if }}", ext = "txt")]
struct MinusTemplate {
    foo: i8,
}

#[test]
fn test_minus() {
    let t = MinusTemplate { foo: 1 };
    assert_eq!(t.call().unwrap(), "Hello");
}

#[derive(Template)]
#[template(source = "{{ foo[\"bar\"] }}", ext = "txt")]
struct IndexTemplate {
    foo: HashMap<String, String>,
}

#[test]
fn test_index() {
    let mut foo = HashMap::new();
    foo.insert("bar".into(), "baz".into());
    let t = IndexTemplate { foo };
    assert_eq!(t.call().unwrap(), "baz");
}

#[derive(Template)]
#[template(path = "tuple-attr.html")]
struct TupleAttrTemplate<'a> {
    tuple: (&'a str, &'a str),
}

#[test]
fn test_tuple_attr() {
    let t = TupleAttrTemplate {
        tuple: ("foo", "bar"),
    };
    assert_eq!(t.call().unwrap(), "foobar");
}

struct Holder {
    a: usize,
}

struct NestedHolder {
    holder: Holder,
}

#[derive(Template)]
#[template(path = "nested-attr.html")]
struct NestedAttrTemplate {
    inner: NestedHolder,
}

#[test]
fn test_nested_attr() {
    let t = NestedAttrTemplate {
        inner: NestedHolder {
            holder: Holder { a: 5 },
        },
    };
    assert_eq!(t.call().unwrap(), "5");
}

#[derive(Template)]
#[template(path = "literals.html")]
struct LiteralsTemplate {}

#[test]
fn test_literals() {
    let s = LiteralsTemplate {};
    assert_eq!(s.call().unwrap(), "a");
}

#[derive(Template)]
#[template(source = "foo", ext = "txt")]
struct Empty;

#[test]
fn test_empty() {
    assert_eq!(Empty.call().unwrap(), "foo");
}

struct Foo {
    a: usize,
}

#[derive(Template)]
#[template(path = "attr.html")]
struct AttrTemplate {
    inner: Foo,
}

#[test]
fn test_attr() {
    let t = AttrTemplate {
        inner: Foo { a: 1 },
    };
    assert_eq!(t.call().unwrap(), "1");
}

#[derive(Template)]
#[template(path = "option.html")]
struct OptionTemplate {
    var: Option<usize>,
}

#[test]
fn test_option() {
    let t = OptionTemplate { var: Some(1) };
    assert_eq!(t.call().unwrap(), "some: 1");
}
