use memchr::memchr;

use std::str::from_utf8;

use crate::parser::{partial, Input, Node};

pub(crate) fn parse_partials(src: &str) -> Vec<Node> {
    match eat_partials(Input(src.as_bytes())) {
        Ok((l, res)) => {
            if l.0.is_empty() {
                return res;
            }
            panic!(
                "problems pre partials parsing template source: {:?}",
                from_utf8(l.0).unwrap()
            );
        }
        Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => panic!(
            "problems pre parsing partials at template source: {:?}",
            err
        ),
        Err(nom::Err::Incomplete(_)) => panic!("pre partials parsing incomplete"),
    }
}

fn eat_partials(mut i: Input) -> Result<(Input, Vec<Node>), nom::Err<Input>> {
    let mut nodes = vec![];

    loop {
        if let Some(j) = memchr(b'{', i.0) {
            let n = &i[j + 1..];

            i = if 1 < n.len() && n[0] == b'{' && n[1] == b'>' {
                let i = Input(&i[j + 3..]);
                match partial(i) {
                    Ok((i, n)) => {
                        nodes.push(n);
                        i
                    }
                    Err(nom::Err::Failure(err)) => break Err(nom::Err::Failure(err)),
                    Err(_) => i,
                }
            } else {
                // next
                Input(n)
            }
        } else {
            break Ok((Input(&[]), nodes));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let src = r#""#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{{/"#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{{"#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{"#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{{>"#;
        assert_eq!(parse_partials(src), vec![]);
        let src = r#"{{>}}"#;
        assert_eq!(parse_partials(src), vec![]);
    }
}
