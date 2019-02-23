use std::fmt::Error;
use yarte::{Result, Template};

#[derive(Template)]
#[template(source = "Hello, {{ name }}!", ext = "txt")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_hello() {
    let t = HelloTemplate { name: "world" }; // instantiate your struct
    assert_eq!("Hello, world!", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(path = "hello.txt")]
struct HelloTxtTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_hello_txt() {
    let t = HelloTxtTemplate { name: "world" }; // instantiate your struct
    assert_eq!("Hello, world!", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "{{# if let Some(_) = cond -}}
    foo
{{- else if let Some(..) = cond -}}
    bar
{{/-if}}",
    ext = "txt"
)]
struct IgnoreTemplate {
    cond: Option<bool>,
}

#[test]
fn test_ignore() {
    let t = IgnoreTemplate { cond: Some(false) }; // instantiate your struct
    assert_eq!("foo", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "{{  name.chars().position(|x| x.eq(&'o')).is_some() }}",
    ext = "txt"
)]
struct ClosureTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_closure() {
    let t = ClosureTemplate { name: "world" }; // instantiate your struct
    assert_eq!("true", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "{{ let a = |a: &str| a.repeat(2) }}{{ a(name) }}",
    ext = "txt"
)]
struct LetClosureTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_let_closure() {
    let t = LetClosureTemplate { name: "world" }; // instantiate your struct
    assert_eq!("worldworld", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "{{ let a = |n| name.repeat(n) }}{{ a(1) }}",
    ext = "txt"
)]
struct LetClosureScopeTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_let_closure_scope() {
    let t = LetClosureScopeTemplate { name: "world" }; // instantiate your struct
    assert_eq!("world", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(source = "{{ let a = name }}Hello, {{ a }}!", ext = "txt")]
struct LetTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_let() {
    let t = LetTemplate { name: "world" }; // instantiate your struct
    assert_eq!("Hello, world!", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "Hello, {{ names.0.first }} {{ names.0.last }} and {{ names.1.first }} {{ names.1.last }}!",
    ext = "txt"
)]
struct WithFieldsTemplate<'a> {
    names: (Name<'a>, Name<'a>),
}

struct Name<'a> {
    first: &'a str,
    last: &'a str,
}

#[test]
fn test_with_fields() {
    let t = WithFieldsTemplate {
        names: (
            Name {
                first: "foo",
                last: "bar",
            },
            Name {
                first: "fOO",
                last: "bAR",
            },
        ),
    }; // instantiate your struct
    assert_eq!("Hello, foo bar and fOO bAR!", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "{{ let a = (name, name) }}Hello, {{ a.0 }}{{ a.1 }}!",
    ext = "txt"
)]
struct LetWithTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_let_with() {
    let t = LetWithTemplate { name: "world" }; // instantiate your struct
    assert_eq!("Hello, worldworld!", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "{{ let cond = !cond }}{{
              let a = if !cond {
              let a = if cond { \"world\" } else { \"foo\" };
              a
              } else if !cond {
              \"foo\"
              } else {
              \"bar\"
              }
              }}Hello, {{ self.cond }} {{ cond }} {{ a }}!",
    ext = "txt"
)]
struct LetIfTemplate {
    cond: bool,
}

#[test]
fn test_let_if() {
    let t = LetIfTemplate { cond: true }; // instantiate your struct
    assert_eq!("Hello, true false foo!", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "Hello, {{
                if let Some(cond) = cond {
                    if !cond {
                        let cond =  \"bar\";
                        cond
                    } else {
                        \"foo\"
                    }
                } else {
                    \"fun\"
                } }}!",
    ext = "txt"
)]
struct LetIfSomeTemplate {
    cond: Option<bool>,
}

#[test]
fn test_let_if_some() {
    let t = LetIfSomeTemplate { cond: Some(false) }; // instantiate your struct
    assert_eq!("Hello, bar!", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "Hello, {{
                if let Some(cond) = cond {
                    if !cond {
                        let cond =  \"bar\";
                        cond
                    } else {
                        \"foo\"
                    }
                } else if let Some(cond) = check {
                    if !cond {
                        let cond =  \"fun\";
                        cond
                    } else {
                        \"baa\"
                    }
                } else {
                    \"None\"
                } }}!",
    ext = "txt"
)]
struct LetElseIfSomeTemplate {
    cond: Option<bool>,
    check: Option<bool>,
}

#[test]
fn test_let_else_if_some() {
    let t = LetElseIfSomeTemplate {
        cond: Some(false),
        check: Some(false),
    }; // instantiate your struct
    assert_eq!("Hello, bar!", t.render().unwrap()); // then render it.
    let t = LetElseIfSomeTemplate {
        cond: Some(true),
        check: Some(false),
    }; // instantiate your struct
    assert_eq!("Hello, foo!", t.render().unwrap()); // then render it.
    let t = LetElseIfSomeTemplate {
        cond: None,
        check: Some(true),
    }; // instantiate your struct
    assert_eq!("Hello, baa!", t.render().unwrap()); // then render it.
    let t = LetElseIfSomeTemplate {
        cond: None,
        check: Some(false),
    }; // instantiate your struct
    assert_eq!("Hello, fun!", t.render().unwrap()); // then render it.
    let t = LetElseIfSomeTemplate {
        cond: None,
        check: None,
    }; // instantiate your struct
    assert_eq!("Hello, None!", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "Hello, {{#each conditions}}{{
                if let Some(cond) = cond {
                    if !cond {
                        let cond =  \"bar\";
                        cond
                    } else {
                        \"foo\"
                    }
                } else if let Some(cond) = check {
                    if !cond {
                        let cond =  \"fun\";
                        cond
                    } else {
                        \"baa\"
                    }
                } else {
                    \"None\"
                } }}{{/each}}!",
    ext = "txt"
)]
struct LetElseIfEachSomeTemplate {
    conditions: Vec<Cond>,
}

struct Cond {
    cond: Option<bool>,
    check: Option<bool>,
}

#[test]
fn test_let_else_if_each_some() {
    let mut conditions = vec![];
    for _ in 0..5 {
        conditions.push(Cond {
            cond: Some(false),
            check: Some(false),
        })
    }

    let t = LetElseIfEachSomeTemplate { conditions }; // instantiate your struct
    assert_eq!("Hello, barbarbarbarbar!", t.render().unwrap()); // then render it.

    let mut conditions = vec![];
    for _ in 0..5 {
        conditions.push(Cond {
            cond: None,
            check: None,
        })
    }

    let t = LetElseIfEachSomeTemplate { conditions }; // instantiate your struct
    assert_eq!("Hello, NoneNoneNoneNoneNone!", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "Hello, {{#each conditions}}
    {{#-if let Some(check) = cond }}
        {{#-if check }}
            {{ let cond = if check { \"&foo\" } else { \"&\"} }}
            {{
                if check {
                    cond
                } else if let Some(cond) = key.cond {
                    if cond {
                        \"1\"
                    } else {
                        \"2\"
                    }
                } else {
                   \"for\"
                }
            }}
        {{- else if let Some(_) = cond }}
        {{- else if let Some(cond) = key.check }}
            {{#-if cond -}}
                baa
            {{/-if }}
        {{- else -}}
            {{ cond.is_some() }}
        {{/-if-}}
        {{ cond.is_some() && true }}
    {{-else if let Some(cond) = check }}
        {{#-if cond -}}
            bar
        {{/-if}}
    {{- else -}}
        None
    {{/-if
}}{{/each}}!",
    ext = "html"
)]
struct ElseIfEachSomeTemplate {
    conditions: Vec<Cond>,
}

#[test]
fn test_else_if_each_some() {
    let mut conditions = vec![];
    for _ in 0..5 {
        conditions.push(Cond {
            cond: Some(true),
            check: Some(false),
        })
    }

    let t = ElseIfEachSomeTemplate { conditions }; // instantiate your struct
    assert_eq!(
        "Hello, &amp;footrue&amp;footrue&amp;footrue&amp;footrue&amp;footrue!",
        t.render().unwrap()
    ); // then render it.

    let mut conditions = vec![];
    for _ in 0..5 {
        conditions.push(Cond {
            cond: None,
            check: None,
        })
    }

    let t = ElseIfEachSomeTemplate { conditions }; // instantiate your struct
    assert_eq!("Hello, NoneNoneNoneNoneNone!", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "{{ let mut a = name.chars() }}
{{ let b: String = loop {
        if a.next().is_none() && true {
            let mut a = name.repeat(1);
            a.push('!');
            break a.repeat(2);
        } else {
            continue;
        }
    }
}}{{ b }}",
    ext = "html"
)]
struct LetLoopTemplate<'a> {
    name: &'a str,
}

#[test]
fn test_let_loop() {
    let t = LetLoopTemplate { name: "&foo" }; // instantiate your struct
    assert_eq!("&amp;foo!&amp;foo!", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(
    source = "
{{ let doubled = a.iter().map(|x| x * 2).collect::<Vec<_>>() }}
{{ let doubled: Vec<usize> = a.iter().map(|x| x * 2).collect() }}
{{#each doubled -}}
    {{ key + 1 }}
{{/-each}}",
    ext = "html"
)]
struct LetCollectTemplate {
    a: Vec<usize>,
}

#[test]
fn test_let_collect() {
    let t = LetCollectTemplate { a: vec![0, 1] }; // instantiate your struct
    assert_eq!("13", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(source = "{{ a? }}", ext = "html")]
struct TryTemplate {
    a: yarte::Result<usize>,
}

#[test]
fn test_try() {
    let t = TryTemplate { a: Err(Error) }; // instantiate your struct
    assert!(t.render().is_err()); // then render it.

    let t = TryTemplate { a: Ok(1) }; // instantiate your struct
    assert_eq!("1", t.render().unwrap()); // then render it.
}

#[derive(Template)]
#[template(source = "{{#unless self.not_is(some)? }}foo{{/unless}}", ext = "txt")]
struct TryMethodTemplate {
    some: bool,
}

impl TryMethodTemplate {
    fn not_is(&self, some: bool) -> Result<bool> {
        if some {
            Ok(false)
        } else {
            Err(Error)
        }
    }
}

#[test]
fn test_try_method() {
    let t = TryMethodTemplate { some: false }; // instantiate your struct
    assert!(t.render().is_err()); // then render it.

    let t = TryMethodTemplate { some: true }; // instantiate your struct
    assert_eq!("foo", t.render().unwrap()); // then render it.
}
