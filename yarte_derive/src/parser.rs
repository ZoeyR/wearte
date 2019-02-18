use memchr::memchr;
use nom;
use syn::{parse_str, Expr, Stmt};

use std::str::{self, from_utf8};

pub(crate) type Ws = (bool, bool);

#[derive(Debug, PartialEq)]
pub(crate) enum Node<'a> {
    Let(Stmt),
    Lit(&'a str, &'a str, &'a str),
    Comment(&'a str),
    Safe(Ws, Expr),
    Expr(Ws, Expr),
    Helper(Helper<'a>),
    Partial(Ws, &'a str),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Helper<'a> {
    Each((Ws, Ws), Expr, Vec<Node<'a>>),
    If(
        ((Ws, Ws), Expr, Vec<Node<'a>>),
        Vec<(Ws, Expr, Vec<Node<'a>>)>,
        Option<(Ws, Vec<Node<'a>>)>,
    ),
    // TODO:
    With((Ws, Ws), Expr, Vec<Node<'a>>),
    Unless((Ws, Ws), Expr, Vec<Node<'a>>),
    Defined((Ws, Ws), &'a str, Expr, Vec<Node<'a>>),
}

const ERR_ARGS: nom::ErrorKind = nom::ErrorKind::Custom(0);
const ERR_EXPR: nom::ErrorKind = nom::ErrorKind::Custom(1);
const ERR_HELPER: nom::ErrorKind = nom::ErrorKind::Custom(2);
const ERR_IDENT: nom::ErrorKind = nom::ErrorKind::Custom(3);
const ERR_IF: nom::ErrorKind = nom::ErrorKind::Custom(4);
const ERR_LOCAL: nom::ErrorKind = nom::ErrorKind::Custom(5);

pub(crate) fn parse(src: &str) -> Vec<Node> {
    match eat(Input(src.as_bytes())) {
        Ok((l, res)) => {
            if l.0.is_empty() {
                return res;
            }
            panic!(
                "problems parsing template source: {:?}",
                from_utf8(l.0).unwrap()
            );
        }
        Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
            match err.clone().into_error_kind() {
                ERR_EXPR => panic!(
                    "problems parsing wrapped or unwrapped expression: {:?}",
                    err
                ),
                ERR_ARGS => panic!("problems parsing arguments: {:?}", err),
                ERR_HELPER => panic!("problems parsing helper: {:?}", err),
                ERR_IDENT => panic!("problems parsing identification variable: {:?}", err),
                ERR_IF => panic!("problems parsing helper IF: {:?}", err),
                ERR_LOCAL => panic!("problems parsing LET block: {:?}", err),
                _ => panic!("problems parsing template source: {:?}", err),
            }
        }
        Err(nom::Err::Incomplete(_)) => panic!("parsing incomplete"),
    }
}

type Input<'a> = nom::types::CompleteByteSlice<'a>;

#[allow(non_snake_case)]
fn Input(input: &[u8]) -> Input {
    nom::types::CompleteByteSlice(input)
}

macro_rules! try_eat {
    ($nodes:ident, $i:ident, $at:ident, $j:ident, $($t:tt)+) => {
        match $($t)+ {
            Ok((c, n)) => {
                eat_lit!($nodes, &$i[..$at + $j]);
                $nodes.push(n);
                $i = c;
                0
            },
            Err(nom::Err::Failure(err)) => break Err(nom::Err::Failure(err)),
            Err(_) => $at + $j + 3,
        }
    };
}

macro_rules! eat_lit {
    ($nodes:ident, $i:expr) => {
        let i = &$i;
        if !i.is_empty() {
            let (l, lit, r) = trim(Input(i));
            $nodes.push(Node::Lit(
                from_utf8(l.0).unwrap(),
                from_utf8(lit.0).unwrap(),
                from_utf8(r.0).unwrap(),
            ));
        }
    };
}

macro_rules! kill {
    ($nodes:ident, $c:expr, $i:expr) => {{
        eat_lit!($nodes, $i);
        break Ok((Input($c), $nodes));
    }};
}

/// $callback: special expressions like {{ else if }}
macro_rules! make_eater {
    ($name:ident, $callback:ident) => {
        fn $name(mut i: Input) -> Result<(Input, Vec<Node>), nom::Err<Input>> {
            let mut nodes = vec![];
            let mut at = 0;

            loop {
                if let Some(j) = memchr(b'{', &i[at..]) {
                    let n = &i[at + j + 1..];
                    at = if 1 < n.len() {
                        if n[0] == b'{' {
                            match n[1] {
                                b'{' => try_eat!(nodes, i, at, j, safe(Input(&i[at + j + 3..]))),
                                b'!' => try_eat!(nodes, i, at, j, comment(Input(&i[at + j + 3..]))),
                                b'#' => try_eat!(nodes, i, at, j, helper(Input(&i[at + j + 3..]))),
                                b'>' => try_eat!(nodes, i, at, j, partial(Input(&i[at + j + 3..]))),
                                b'/' => kill!(nodes, &i[at + j + 3..], i[..at + j]),
                                _ => {
                                    $callback!(nodes, i, at, j);
                                    try_eat!(nodes, i, at, j, expr(Input(&i[at + j + 2..])))
                                }
                            }
                        } else {
                            // next
                            at + j + 2
                        }
                    } else {
                        kill!(nodes, &[], i.0);
                    };
                } else {
                    kill!(nodes, &[], i.0);
                }
            }
        }
    };
}

macro_rules! non {
    ($($t:tt)*) => {};
}

make_eater!(eat, non);

static IF: &[u8] = b"if";
static ELSE: &[u8] = b"else";

macro_rules! is_else {
    ($n:ident, $i:ident, $at:ident, $j:ident) => {
        if let Ok((c, _)) = do_parse!(
            Input(&$i[$at + $j + 2..]),
            opt!(tag!("-")) >> take_while!(ws) >> (())
        ) {
            if c.0.starts_with(ELSE) {
                kill!($n, &$i[$at + $j + 2..], $i[..$at + $j]);
            }
        }
    };
}

make_eater!(eat_if, is_else);

// TODO: terminated with memchr
named!(comment<Input, Node>, map!(
    alt!(
        delimited!(tag!("--"), take_until!("--!}}"), tag!("--!}}")) |
        terminated!(take_until!("!}}"), tag!("!}}"))
    ),
    |i| Node::Comment(from_utf8(i.0).unwrap())
));

static LET: &[u8] = b"let ";
macro_rules! try_eat_local {
    ($c:ident, $s:ident) => {
        if $s.0.starts_with(LET) {
            if let Ok(e) = eat_local($s) {
                return Ok(($c, Node::Let(e)));
            }
        }
    };
}

macro_rules! map_failure {
    ($i:expr, $e:ident, $($t:tt)+) => {
        ($($t)+).map_err(|_| nom::Err::Failure(error_position!($i, $e)))
    };
}

named!(partial<Input, Node>, do_parse!(
    lws: opt!(tag!("-"))
        >> take_while!(ws)
        >> ident: path
        >> take_while!(ws)
        >> rws: opt!(tag!("-"))
        >> tag!("}}")
        >> (Node::Partial((lws.is_some(), rws.is_some()), ident))
));

fn helper(i: Input) -> Result<(Input, Node), nom::Err<Input>> {
    let (i, (above_ws, ident, args)) = do_parse!(
        i,
        lws: opt!(tag!("-"))
            >> take_while!(ws)
            >> ident: identifier
            >> args: arguments
            >> rws: map!(take!(1), |x| x.0.starts_with(b"-"))
            >> alt!(tag!("}}") | terminated!(take!(1), tag!("}}")))
            >> (((lws.is_some(), rws), ident, args))
    )?;

    if ident.eq("if") {
        return if_else(above_ws, i, args);
    }

    let (c, (below_ws, block, c_ident)) = map_failure!(
        i,
        ERR_HELPER,
        do_parse!(
            i,
            block: eat
                >> lws: opt!(tag!("-"))
                >> take_while!(ws)
                >> c_ident: identifier
                >> take_while!(ws)
                >> rws: opt!(tag!("-"))
                >> tag!("}}")
                >> (((lws.is_some(), rws.is_some()), block, c_ident))
        )
    )?;

    if ident.eq(c_ident) {
        Ok((
            c,
            Node::Helper({
                match ident {
                    "each" => Helper::Each((above_ws, below_ws), args, block),
                    "with" => Helper::With((above_ws, below_ws), args, block),
                    "unless" => Helper::Unless((above_ws, below_ws), args, block),
                    defined => Helper::Defined((above_ws, below_ws), defined, args, block),
                }
            }),
        ))
    } else {
        Err(nom::Err::Failure(error_position!(i, ERR_HELPER)))
    }
}

#[inline]
fn if_else(abode_ws: Ws, i: Input, args: Expr) -> Result<(Input, Node), nom::Err<Input>> {
    let (i, first) = eat_if(i)?;

    let mut nodes = vec![];
    let mut tail = None;
    let (mut i, mut lws) = do_parse!(
        i,
        lws: opt!(tag!("-")) >> take_while!(ws) >> (lws.is_some())
    )?;

    loop {
        if i.0.starts_with(ELSE) {
            if let Ok((j, _)) = terminated!(Input(&i[4..]), take_while!(ws), tag!(IF)) {
                let (c, b) = map_failure!(
                    j,
                    ERR_IF,
                    do_parse!(
                        j,
                        args: arguments
                            >> rws: map!(take!(1), |x| x.0.starts_with(b"-"))
                            >> tag!("}}")
                            >> block: eat_if
                            >> (((lws, rws), args, block))
                    )
                )?;
                nodes.push(b);
                i = c;
            } else {
                let (c, b) = map_failure!(
                    i,
                    ERR_IF,
                    do_parse!(
                        Input(&i[4..]),
                        take_while!(ws)
                            >> rws: opt!(tag!("-"))
                            >> tag!("}}")
                            >> block: eat
                            >> (((lws, rws.is_some()), block))
                    )
                )?;
                tail = Some(b);
                i = c;
            }
        } else if i.0.starts_with(IF) {
            let (c, below_ws) = map_failure!(
                i,
                ERR_IF,
                do_parse!(
                    Input(&i[2..]),
                    take_while!(ws) >> rws: opt!(tag!("-")) >> tag!("}}") >> ((lws, rws.is_some()))
                )
            )?;
            break Ok((
                c,
                Node::Helper(Helper::If(((abode_ws, below_ws), args, first), nodes, tail)),
            ));
        } else {
            break Err(nom::Err::Failure(error_position!(i, ERR_IF)));
        }

        let c = do_parse!(
            i,
            lws: opt!(tag!("-")) >> take_while!(ws) >> (lws.is_some())
        )?;
        i = c.0;
        lws = c.1;
    }
}

fn arguments(i: Input) -> Result<(Input, Expr), nom::Err<Input>> {
    let mut at = 0;
    loop {
        if let Some(j) = memchr(b'}', &i[at..]) {
            let n = &i[at + j + 1..];
            if n.is_empty() {
                break Err(nom::Err::Error(error_position!(i, ERR_ARGS)));
            } else {
                if n[0] == b'}' {
                    break if 0 < at + j {
                        if i[at + j - 1] == b'-' {
                            eat_expr(Input(&i[..at + j - 1])).map(|e| (Input(&i[at + j - 1..]), e))
                        } else {
                            eat_expr(Input(&i[..at + j])).map(|e| (Input(&i[at + j - 1..]), e))
                        }
                    } else {
                        Err(nom::Err::Failure(error_position!(i, ERR_ARGS)))
                    };
                } else {
                    // next
                    at += j + 2;
                }
            }
        } else {
            break Err(nom::Err::Error(error_position!(i, ERR_ARGS)));
        }
    }
}

macro_rules! make_expr {
    ($i:ident, $sw:expr, $d:expr, $ret:ident) => {{
        let (mut at, lws) = if $i.0.starts_with(b"-") {
            (1, true)
        } else {
            (0, false)
        };

        let (c, rws, s) = loop {
            if let Some(j) = memchr(b'}', &$i[at..]) {
                let n = &$i[at + j + 1..];
                if n.starts_with($sw) {
                    if 0 < at + j {
                        let init = if lws { 1 } else { 0 };
                        break if $i[at + j - 1] == b'-' {
                            (
                                Input(&$i[at + j + $d..]),
                                true,
                                Input(&$i[init..at + j - 1]),
                            )
                        } else {
                            (Input(&$i[at + j + $d..]), false, Input(&$i[init..at + j]))
                        };
                    }
                }

                at += j + 2;
            } else {
                return Err(nom::Err::Error(error_position!($i, ERR_ARGS)));
            }
        };

        let (_, s, _) = trim(s);
        try_eat_local!(c, s);
        eat_expr(s).map(|e| (c, Node::$ret((lws, rws), e)))
    }};
}

fn safe(i: Input) -> Result<(Input, Node), nom::Err<Input>> {
    make_expr!(i, b"}}", 3, Safe)
}

fn expr(i: Input) -> Result<(Input, Node), nom::Err<Input>> {
    make_expr!(i, b"}", 2, Expr)
}

#[inline]
fn eat_expr(i: Input) -> Result<Expr, nom::Err<Input>> {
    map_failure!(i, ERR_EXPR, parse_str::<Expr>(from_utf8(i.0).unwrap()))
}

#[inline]
fn eat_local(i: Input) -> Result<Stmt, nom::Err<Input>> {
    map_failure!(
        i,
        ERR_LOCAL,
        parse_str::<Stmt>(&[from_utf8(i.0).unwrap(), ";"].join(""))
    )
}

fn identifier(i: Input) -> Result<(Input, &str), nom::Err<Input>> {
    if i.0.is_empty() || !nom::is_alphabetic(i[0]) && i[0] != b'_' {
        return Err(nom::Err::Error(error_position!(i, ERR_IDENT)));
    }

    for (j, c) in i[1..].iter().enumerate() {
        if !nom::is_alphanumeric(*c) && *c != b'_' {
            return Ok((Input(&i[j + 1..]), from_utf8(&i[..j + 1]).unwrap()));
        }
    }

    Ok((Input(&i[1..]), str::from_utf8(&i[..1]).unwrap()))
}

named!(path<Input, &str>, map!(take_while1!(is_path), |x| str::from_utf8(&x).unwrap()));

#[inline]
fn is_path(n: u8) -> bool {
    n.is_ascii_graphic()
}

#[inline]
fn ws(n: u8) -> bool {
    n.is_ascii_whitespace()
}

fn trim(i: Input) -> (Input, Input, Input) {
    if i.0.is_empty() {
        return (Input(&[]), Input(&[]), Input(&[]));
    }

    if let Some(ln) = i.iter().position(|x| !ws(*x)) {
        let rn = i.iter().rposition(|x| !ws(*x)).unwrap();
        (Input(&i[..ln]), Input(&i[ln..rn + 1]), Input(&i[rn + 1..]))
    } else {
        (i, Input(&[]), Input(&[]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    const WS: Ws = (false, false);

    #[test]
    fn test_empty() {
        let src = r#""#;
        assert_eq!(parse(src), vec![]);
        let src = r#"{{/"#;
        assert_eq!(parse(src), vec![]);
    }

    #[test]
    fn test_fallback() {
        let src = r#"{{"#;
        assert_eq!(parse(src), vec![Node::Lit("", "{{", "")]);
        let src = r#"{{{"#;
        assert_eq!(parse(src), vec![Node::Lit("", "{{{", "")]);
        let src = r#"{{#"#;
        assert_eq!(parse(src), vec![Node::Lit("", "{{#", "")]);
        let src = r#"{{>"#;
        assert_eq!(parse(src), vec![Node::Lit("", "{{>", "")]);
    }

    #[test]
    fn test_eat_comment() {
        let src = r#"{{! Commentary !}}"#;
        assert_eq!(parse(src), vec![Node::Comment(" Commentary ")]);
        let src = r#"{{!-- Commentary --!}}"#;
        assert_eq!(parse(src), vec![Node::Comment(" Commentary ")]);
    }

    #[test]
    fn test_eat_expr() {
        let src = r#"{{ var }}"#;
        assert_eq!(
            parse(src),
            vec![Node::Expr(WS, parse_str::<Expr>("var").unwrap())]
        );

        let src = r#"{{ fun() }}"#;
        assert_eq!(
            parse(src),
            vec![Node::Expr(WS, parse_str::<Expr>("fun()").unwrap())]
        );

        let src = r#"{{ fun(|a| a) }}"#;
        assert_eq!(
            parse(src),
            vec![Node::Expr(WS, parse_str::<Expr>("fun(|a| a)").unwrap())]
        );

        let src = r#"{{
            fun(|a| {
                { a }
            })
        }}"#;
        assert_eq!(
            parse(src),
            vec![Node::Expr(WS, parse_str::<Expr>("fun(|a| {{a}})").unwrap())]
        );
    }

    #[should_panic]
    #[test]
    fn test_eat_expr_panic_a() {
        let src = r#"{{ fn(|a| {{a}}) }}"#;
        parse(src);
    }

    #[should_panic]
    #[test]
    fn test_eat_expr_panic_b() {
        let src = r#"{{ let a = mut a  }}"#;
        parse(src);
    }

    #[test]
    fn test_eat_safe() {
        let src = r#"{{{ var }}}"#;
        assert_eq!(
            parse(src),
            vec![Node::Safe(WS, parse_str::<Expr>("var").unwrap())]
        );

        let src = r#"{{{ fun() }}}"#;
        assert_eq!(
            parse(src),
            vec![Node::Safe(WS, parse_str::<Expr>("fun()").unwrap())]
        );

        let src = r#"{{{ fun(|a| a) }}}"#;
        assert_eq!(
            parse(src),
            vec![Node::Safe(WS, parse_str::<Expr>("fun(|a| a)").unwrap())]
        );

        let src = r#"{{{
            fun(|a| {
                {{ a }}
            })
        }}}"#;
        assert_eq!(
            parse(src),
            vec![Node::Safe(
                WS,
                parse_str::<Expr>("fun(|a| {{{a}}})").unwrap(),
            )]
        );
    }

    #[should_panic]
    #[test]
    fn test_eat_safe_panic() {
        let src = r#"{{ fn(|a| {{{a}}}) }}"#;
        assert_eq!(
            parse(src),
            vec![Node::Safe(
                WS,
                parse_str::<Expr>("fn(|a| {{{a}}})").unwrap(),
            )]
        );
    }

    #[test]
    fn test_trim() {
        assert_eq!(trim(Input(b" a ")), (Input(b" "), Input(b"a"), Input(b" ")));
        assert_eq!(trim(Input(b" a")), (Input(b" "), Input(b"a"), Input(b"")));
        assert_eq!(trim(Input(b"a")), (Input(b""), Input(b"a"), Input(b"")));
        assert_eq!(trim(Input(b"")), (Input(b""), Input(b""), Input(b"")));
        assert_eq!(trim(Input(b"a ")), (Input(b""), Input(b"a"), Input(b" ")));
        assert_eq!(trim(Input(b"a a")), (Input(b""), Input(b"a a"), Input(b"")));
        assert_eq!(
            trim(Input(b"a a ")),
            (Input(b""), Input(b"a a"), Input(b" "))
        );
        assert_eq!(
            trim(Input(b" \n\t\ra a ")),
            (Input(b" \n\t\r"), Input(b"a a"), Input(b" "))
        );
        assert_eq!(
            trim(Input(b" \n\t\r ")),
            (Input(b" \n\t\r "), Input(b""), Input(b""))
        );
    }

    #[test]
    fn test_eat_if() {
        let src = Input(br#"foo{{ else }}"#);
        assert_eq!(
            eat_if(src).unwrap(),
            (Input(b" else }}"), vec![Node::Lit("", "foo", "")])
        );
        let src = Input(br#"{{foo}}{{else}}"#);
        assert_eq!(
            eat_if(src).unwrap(),
            (
                Input(b"else}}"),
                vec![Node::Expr(WS, parse_str::<Expr>("foo").unwrap())]
            )
        );
        let src = Input(br#"{{ let a = foo }}{{else if cond}}{{else}}"#);
        assert_eq!(
            eat_if(src).unwrap(),
            (
                Input(b"else if cond}}{{else}}"),
                vec![Node::Let(parse_str::<Stmt>("let a = foo;").unwrap())]
            )
        );
    }

    #[test]
    fn test_helpers() {
        let src = Input(b"each name }}{{first}} {{last}}{{/each}}");
        assert_eq!(
            helper(src).unwrap(),
            (
                Input(&[]),
                Node::Helper(Helper::Each(
                    (WS, WS),
                    parse_str::<Expr>("name").unwrap(),
                    vec![
                        Node::Expr(WS, parse_str::<Expr>("first").unwrap()),
                        Node::Lit(" ", "", ""),
                        Node::Expr(WS, parse_str::<Expr>("last").unwrap()),
                    ],
                ))
            )
        );
    }

    #[test]
    fn test_if_else() {
        let src = Input(b"foo{{/if}}");
        let arg = parse_str::<Expr>("bar").unwrap();

        assert_eq!(
            if_else(WS, src, arg).unwrap(),
            (
                Input(b""),
                Node::Helper(Helper::If(
                    (
                        (WS, WS),
                        parse_str::<Expr>("bar").unwrap(),
                        vec![Node::Lit("", "foo", "")]
                    ),
                    vec![],
                    None,
                ))
            )
        );

        let src = Input(b"foo{{else}}bar{{/if}}");
        let arg = parse_str::<Expr>("bar").unwrap();

        assert_eq!(
            if_else(WS, src, arg).unwrap(),
            (
                Input(b""),
                Node::Helper(Helper::If(
                    (
                        (WS, WS),
                        parse_str::<Expr>("bar").unwrap(),
                        vec![Node::Lit("", "foo", "")]
                    ),
                    vec![],
                    Some((WS, vec![Node::Lit("", "bar", "")])),
                ))
            )
        );
    }

    #[test]
    fn test_else_if() {
        let src = Input(b"foo{{else if cond }}bar{{else}}foO{{/if}}");
        let arg = parse_str::<Expr>("bar").unwrap();

        assert_eq!(
            if_else(WS, src, arg).unwrap(),
            (
                Input(b""),
                Node::Helper(Helper::If(
                    (
                        (WS, WS),
                        parse_str::<Expr>("bar").unwrap(),
                        vec![Node::Lit("", "foo", "")]
                    ),
                    vec![(
                        WS,
                        parse_str::<Expr>("cond").unwrap(),
                        vec![Node::Lit("", "bar", "")],
                    )],
                    Some((WS, vec![Node::Lit("", "foO", "")])),
                ))
            )
        );
    }

    #[test]
    fn test_defined() {
        let src = "{{#foo bar}}hello{{/foo}}";

        assert_eq!(
            parse(src),
            vec![Node::Helper(Helper::Defined(
                (WS, WS),
                "foo",
                parse_str::<Expr>("bar").unwrap(),
                vec![Node::Lit("", "hello", "")],
            ))]
        );
    }

    #[test]
    fn test_ws_expr() {
        let src = "{{-foo-}}";
        assert_eq!(
            parse(src),
            vec![Node::Expr((true, true), parse_str::<Expr>("foo").unwrap())]
        );
        let src = "{{- foo-}}";
        assert_eq!(
            parse(src),
            vec![Node::Expr((true, true), parse_str::<Expr>("foo").unwrap())]
        );
        let src = "{{- foo}}";
        assert_eq!(
            parse(src),
            vec![Node::Expr((true, false), parse_str::<Expr>("foo").unwrap())]
        );
        let src = "{{foo    -}}";
        assert_eq!(
            parse(src),
            vec![Node::Expr((false, true), parse_str::<Expr>("foo").unwrap())]
        );
        let src = "{{{-foo-}}}";
        assert_eq!(
            parse(src),
            vec![Node::Safe((true, true), parse_str::<Expr>("foo").unwrap())]
        );
        let src = "{{{-foo -}}}";
        assert_eq!(
            parse(src),
            vec![Node::Safe((true, true), parse_str::<Expr>("foo").unwrap())]
        );
    }

    #[test]
    fn test_ws_each() {
        let src = "{{#-each bar-}}{{/-each-}}";
        assert_eq!(
            parse(src),
            vec![Node::Helper(Helper::Each(
                ((true, true), (true, true)),
                parse_str::<Expr>("bar").unwrap(),
                vec![],
            ))]
        );
    }

    #[test]
    fn test_ws_if() {
        let src = "{{#-if bar-}}{{/-if-}}";
        assert_eq!(
            parse(src),
            vec![Node::Helper(Helper::If(
                (
                    ((true, true), (true, true)),
                    parse_str::<Expr>("bar").unwrap(),
                    vec![],
                ),
                vec![],
                None,
            ))]
        );
    }

    #[test]
    fn test_ws_if_else() {
        let src = "{{#-if bar-}}{{-else-}}{{/-if-}}";
        assert_eq!(
            parse(src),
            vec![Node::Helper(Helper::If(
                (
                    ((true, true), (true, true)),
                    parse_str::<Expr>("bar").unwrap(),
                    vec![],
                ),
                vec![],
                Some(((true, true), vec![])),
            ))]
        );
    }

    #[test]
    fn test_ws_if_else_if() {
        let src = "{{#-if bar-}}{{-else if bar-}}{{-else-}}{{/-if-}}";
        assert_eq!(
            parse(src),
            vec![Node::Helper(Helper::If(
                (
                    ((true, true), (true, true)),
                    parse_str::<Expr>("bar").unwrap(),
                    vec![],
                ),
                vec![((true, true), parse_str::<Expr>("bar").unwrap(), vec![],)],
                Some(((true, true), vec![])),
            ))]
        );
    }

    #[test]
    fn test_partial() {
        let src = "{{> partial }}";
        assert_eq!(parse(src), vec![Node::Partial((false, false), "partial")])
    }
}
