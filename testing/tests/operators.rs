use wearte::Template;

#[derive(Template)]
#[template(path = "compare.html")]
struct CompareTemplate {
    a: usize,
    b: usize,
    c: usize,
}

#[test]
fn test_compare() {
    let t = CompareTemplate { a: 1, b: 1, c: 2 };
    assert_eq!(t.call().unwrap(), "tf\ntf\ntf\ntf\ntf\ntf");
}

#[derive(Template)]
#[template(path = "operators.html")]
struct OperatorsTemplate {
    a: usize,
    b: usize,
    c: usize,
}

#[test]
fn test_operators() {
    let t = OperatorsTemplate { a: 1, b: 1, c: 2 };
    assert_eq!(t.call().unwrap(), "muldivmodaddrshlshbandbxorborandor");
}

#[derive(Template)]
#[template(path = "precedence.html")]
struct PrecedenceTemplate {}

#[test]
fn test_precedence() {
    let t = PrecedenceTemplate {};
    assert_eq!(t.call().unwrap(), "6".repeat(7));
}

#[derive(Template)]
#[template(path = "unless-operators.html")]
struct UnlessOperatorsTemplate {
    a: usize,
    b: usize,
    c: usize,
}

#[test]
fn test_unless_operators() {
    let t = UnlessOperatorsTemplate { a: 1, b: 1, c: 2 };
    assert_eq!(t.call().unwrap(), "");
}
