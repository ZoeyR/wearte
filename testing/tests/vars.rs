use yarte::Template;

#[derive(Template)]
#[template(source = "{{ let v = s }}{{ v }}", ext = "txt")]
struct LetTemplate<'a> {
    s: &'a str,
}

#[test]
fn test_let() {
    let t = LetTemplate { s: "foo" };
    assert_eq!(t.render().unwrap(), "foo");
}

#[derive(Template)]
#[template(path = "let.html")]
struct LetTupleTemplate<'a> {
    s: &'a str,
    t: (&'a str, &'a str),
}

#[test]
fn test_let_tuple() {
    let t = LetTupleTemplate {
        s: "foo",
        t: ("bar", "bazz"),
    };
    assert_eq!(t.render().unwrap(), "foo\nbarbazz");
}
