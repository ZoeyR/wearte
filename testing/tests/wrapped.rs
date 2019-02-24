use wearte::Template;

#[derive(Template)]
#[template(
    source = "{{
              if true {
              let a = if true { \"&\" } else { \"&\" };
              a
              } else if true || !cond {
              \"&\"
              } else {
              \"&\"
              }
              }}",
    ext = "html"
)]
struct IfTemplate {
    cond: bool,
}

#[test]
fn test_if() {
    let t = IfTemplate { cond: true }; // instantiate your struct
    assert_eq!("&amp;", t.call().unwrap()); // then call it.
}

#[derive(Template)]
#[template(source = "{{ arr[0] }}", ext = "html")]
struct IndexTemplate<'a> {
    arr: Vec<&'a str>,
}

#[test]
fn test_index() {
    let t = IndexTemplate { arr: vec!["&"] }; // instantiate your struct
    assert_eq!("&amp;", t.call().unwrap()); // then call it.
}

#[derive(Template)]
#[template(source = "{{ arr[..1][0] }}", ext = "html")]
struct SliceTemplate<'a> {
    arr: &'a [&'a str],
}

#[test]
fn test_slice() {
    let arr: &[&str] = &vec!["&"];
    let t = SliceTemplate { arr }; // instantiate your struct
    assert_eq!("&amp;", t.call().unwrap()); // then call it.
}

fn repeat(s: &str, i: usize) -> String {
    s.repeat(i)
}

#[derive(Template)]
#[template(source = "{{ s.repeat(1) }}{{ repeat(s, 1) }}", ext = "html")]
struct CallTemplate<'a> {
    s: &'a str,
}

#[test]
fn test_call() {
    let t = CallTemplate { s: "&" }; // instantiate your struct
    assert_eq!("&amp;&amp;", t.call().unwrap()); // then call it.
}

#[derive(Template)]
#[template(source = "{{ [\"&\"][..1][0] }}", ext = "html")]
struct ArrayTemplate;

#[test]
fn test_array() {
    let t = ArrayTemplate; // instantiate your struct
    assert_eq!("&amp;", t.call().unwrap()); // then call it.
}

#[derive(Template)]
#[template(source = "{{ (\"&\", 1, 1.0, true).0 }}", ext = "html")]
struct TupleTemplate;

#[test]
fn test_tuple() {
    let t = TupleTemplate; // instantiate your struct
    assert_eq!("&amp;", t.call().unwrap()); // then call it.
}

#[derive(Template)]
#[template(
    source = "{{ cond }}{{ 1 + num }}{{ num }}{{ cond || true }}{{ 1.0 }}{{ [true][0..1][0] }}",
    ext = "html",
    print = "code"
)]
struct Simple {
    cond: bool,
    num: usize,
}

#[test]
fn test_simple() {
    let t = Simple { cond: true, num: 0 }; // instantiate your struct
    assert_eq!("true10true1true", t.call().unwrap()); // then call it.
}
