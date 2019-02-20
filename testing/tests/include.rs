use yarte::Template;

#[derive(Template)]
#[template(path = "include.html")]
struct IncludeTemplate<'a> {
    strs: &'a [&'a str],
}

#[test]
fn test_include() {
    let strs = vec!["foo", "bar"];
    let s = IncludeTemplate { strs: &strs };
    assert_eq!(s.render().unwrap(), "\n  INCLUDED: foo1\n  INCLUDED: bar2")
}

#[derive(Template)]
#[template(path = "include-dir.html")]
struct IncludeDirTemplate<'a> {
    strs: &'a [&'a str],
}

#[test]
fn test_include_dir() {
    let strs = vec!["foo", "bar"];
    let s = IncludeDirTemplate { strs: &strs };
    assert_eq!(
        s.render().unwrap(),
        "\n  INCLUDED-DIR: foo1\n  INCLUDED-DIR: bar2"
    )
}

#[derive(Template)]
#[template(path = "deep/include.html")]
struct IncludeDirDTemplate<'a> {
    strs: &'a [&'a str],
}

#[test]
fn test_include_dir_d() {
    let strs = vec!["foo", "bar"];
    let s = IncludeDirDTemplate { strs: &strs };
    assert_eq!(
        s.render().unwrap(),
        "\n  INCLUDED-DIR: foo1\n  INCLUDED-DIR: bar2"
    )
}
