use yarte::Template;

#[derive(Template)]
#[template(path = "for.html")]
struct ForTemplate<'a> {
    strings: Vec<&'a str>,
}

#[test]
fn test_for() {
    let s = ForTemplate {
        strings: vec!["A", "alfa", "1"],
    };
    assert_eq!(s.render().unwrap(), "0. A(first)1. alfa2. 1");
}

#[derive(Template)]
#[template(path = "nested-for.html")]
struct NestedForTemplate<'a> {
    seqs: &'a [&'a [&'a str]],
}

#[test]
fn test_nested_for() {
    let alpha: &[&str] = &vec!["a", "b", "c"];
    let numbers: &[&str] = &vec!["one", "two"];
    let seqs: &[&[&str]] = &vec![alpha, numbers];
    let s = NestedForTemplate { seqs };
    assert_eq!(s.render().unwrap(), "1\n  0a1b2c2\n  0one1two");
}

#[derive(Template)]
#[template(path = "precedence-for.html")]
struct PrecedenceTemplate<'a> {
    strings: &'a [&'a str],
}

#[test]
fn test_precedence_for() {
    let strings: &[&str] = &vec!["A", "alfa", "1"];
    let s = PrecedenceTemplate { strings };
    assert_eq!(s.render().unwrap(), "0. A2foo1. alfa42. 16");
}

#[derive(Template)]
#[template(path = "for-range.html")]
struct ForRangeTemplate {
    init: i32,
    end: i32,
}

#[test]
fn test_for_range() {
    let s = ForRangeTemplate { init: -1, end: 1 };
    assert_eq!(s.render().unwrap(), "foo\nfoo\nbar\nbar\nfoo\nbar\nbar\n");
}
