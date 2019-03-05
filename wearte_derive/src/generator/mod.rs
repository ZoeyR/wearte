use mime_guess::get_mime_type_str;
use syn::{self, visit::Visit};

use std::{
    collections::BTreeMap,
    fmt::{self, Write},
    mem,
    path::PathBuf,
    str,
};

use wearte_config::Config;

mod validator;
mod visit_derive;
mod visit_each;
mod visits;

pub(crate) use self::visit_derive::{visit_derive, Print, Struct};
use self::visit_each::find_loop_var;

use crate::parser::{Helper, Node, Ws};

pub(crate) fn generate(c: &Config, s: &Struct, ctx: Context) -> String {
    Generator::new(c, s, ctx).build()
}

pub(crate) trait EWrite: fmt::Write {
    fn write(&mut self, s: &dyn fmt::Display) {
        write!(self, "{}", s).unwrap()
    }

    fn writeln(&mut self, s: &dyn fmt::Display) {
        writeln!(self, "{}", s).unwrap()
    }
}

impl EWrite for String {}

pub(self) type Context<'a> = &'a BTreeMap<&'a PathBuf, Vec<Node<'a>>>;

#[derive(Debug, PartialEq)]
pub(self) enum On {
    Each(usize),
    With(usize),
}

enum Writable<'a> {
    Lit(&'a str),
    Expr(String, bool),
}

pub(self) struct Generator<'a> {
    pub(self) c: &'a Config<'a>,
    // ast of DeriveInput
    pub(self) s: &'a Struct<'a>,
    // wrapped expression flag
    pub(self) wrapped: bool,
    // will wrap expression Flag
    pub(self) will_wrap: bool,
    // buffer for tokens
    pub(self) buf_t: String,
    // Scope stack
    pub(self) scp: Vec<Vec<String>>,
    // On State stack
    pub(self) on: Vec<On>,
    // buffer for writable
    buf_w: Vec<Writable<'a>>,
    // path - nodes
    ctx: Context<'a>,
    // current file path
    on_path: PathBuf,
    // heuristic based on https://github.com/lfairy/maud
    size_hint: usize,
    // whitespace flag and buffer based on https://github.com/djc/askama
    next_ws: Option<&'a str>,
    skip_ws: bool,
}

impl<'a> Generator<'a> {
    fn new<'n>(c: &'n Config<'n>, s: &'n Struct<'n>, ctx: Context<'n>) -> Generator<'n> {
        Generator {
            c,
            s,
            ctx,
            buf_t: String::new(),
            buf_w: vec![],
            next_ws: None,
            on: vec![],
            on_path: s.path.clone(),
            scp: vec![vec!["self".to_string()]],
            skip_ws: false,
            will_wrap: true,
            wrapped: true,
            size_hint: 0,
        }
    }

    fn build(&mut self) -> String {
        let mut buf = String::new();

        let nodes: &[Node] = self.ctx.get(&self.on_path).unwrap();
        self.display(nodes, &mut buf);

        debug_assert_ne!(self.size_hint, 0);
        self.template(&mut buf);

        if cfg!(feature = "actix-web") {
            self.responder(&mut buf);
        }

        buf
    }

    fn get_mime(&mut self) -> &str {
        let ext = if self.s.wrapped {
            match self.s.path.extension() {
                Some(s) => s.to_str().unwrap(),
                None => "txt",
            }
        } else {
            "html"
        };

        get_mime_type_str(ext).expect("valid mime ext")
    }

    fn template(&mut self, buf: &mut String) {
        self.s.implement_head("::wearte::Template", buf);

        buf.writeln(&"fn mime() -> &'static str {");
        writeln!(buf, "{:?}", self.get_mime()).unwrap();
        buf.writeln(&"}");
        buf.writeln(&"fn size_hint() -> usize {");
        buf.writeln(&self.size_hint);
        buf.writeln(&"}");
        buf.writeln(&"}");
    }

    fn display(&mut self, nodes: &'a [Node], buf: &mut String) {
        self.s.implement_head("::std::fmt::Display", buf);

        buf.writeln(&"fn fmt(&self, _fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {");

        let last = buf.len();

        self.handle(nodes, buf);
        debug_assert_eq!(self.scp.len(), 1);
        debug_assert_eq!(self.scp[0][0], "self");
        debug_assert_eq!(self.on.len(), 0);
        debug_assert_eq!(self.on_path, self.s.path);
        debug_assert!(self.will_wrap);
        self.write_buf_writable(buf);
        self.size_hint = 1 + buf.len() - last;

        buf.writeln(&quote!(Ok(())));

        buf.writeln(&"}");
        buf.writeln(&"}");
    }

    fn responder(&mut self, buf: &mut String) {
        self.s.implement_head("::wearte::actix_web::Responder", buf);

        buf.writeln(&"type Item = ::wearte::actix_web::HttpResponse;");
        buf.writeln(&"type Error = ::wearte::actix_web::Error;");
        buf.writeln(
            &"fn respond_to<S>(self, _req: &::wearte::actix_web::HttpRequest<S>) \
              -> ::std::result::Result<Self::Item, Self::Error> {",
        );

        buf.writeln(
            &"self.call()
                .map(|s| Self::Item::Ok().content_type(Self::mime()).body(s))
                .map_err(|_| ::wearte::actix_web::ErrorInternalServerError(\"Template parsing error\"))"
        );

        buf.writeln(&"}");
        buf.writeln(&"}");
    }

    fn handle(&mut self, nodes: &'a [Node], buf: &mut String) {
        for n in nodes {
            match n {
                Node::Local(expr) => {
                    validator::statement(expr);

                    self.skip_ws();
                    self.write_buf_writable(buf);
                    self.visit_stmt(expr);
                    buf.writeln(&mem::replace(&mut self.buf_t, String::new()));
                }
                Node::Safe(ws, expr) => {
                    validator::expression(expr);

                    self.visit_expr(expr);
                    self.handle_ws(ws);
                    self.buf_w.push(Writable::Expr(
                        mem::replace(&mut self.buf_t, String::new()),
                        true,
                    ));
                }
                Node::Expr(ws, expr) => {
                    validator::expression(expr);

                    self.wrapped = false;
                    self.visit_expr(expr);
                    self.handle_ws(ws);
                    self.buf_w.push(Writable::Expr(
                        mem::replace(&mut self.buf_t, String::new()),
                        self.wrapped,
                    ))
                }
                Node::Lit(l, lit, r) => self.visit_lit(l, lit, r),
                Node::Helper(h) => self.visit_helper(buf, h),
                Node::Partial(ws, path, expr) => self.visit_partial(buf, ws, path, expr),
                Node::Comment(..) => self.skip_ws(),
                Node::Raw(ws, l, v, r) => {
                    self.handle_ws(&ws.0);
                    self.visit_lit(l, v, r);
                    self.handle_ws(&ws.1);
                }
            }
        }
    }

    fn visit_lit(&mut self, lws: &'a str, lit: &'a str, rws: &'a str) {
        debug_assert!(self.next_ws.is_none());
        if !lws.is_empty() {
            if self.skip_ws {
                self.skip_ws = false;
            } else if lit.is_empty() {
                debug_assert!(rws.is_empty());
                self.next_ws = Some(lws);
            } else {
                self.buf_w.push(Writable::Lit(lws));
            }
        }

        if !lit.is_empty() {
            self.buf_w.push(Writable::Lit(lit));
        }

        if !rws.is_empty() {
            self.next_ws = Some(rws);
        }
    }

    fn visit_helper(&mut self, buf: &mut String, h: &'a Helper<'a>) {
        use crate::parser::Helper::*;
        match h {
            Each(ws, e, b) => self.visit_each(buf, ws, e, b),
            If(ifs, elsif, els) => self.visit_if(buf, ifs, elsif, els),
            With(ws, e, b) => self.visit_with(buf, ws, e, b),
            Unless(ws, e, b) => self.visit_unless(buf, ws, e, b),
            Defined(..) => unimplemented!(),
        }
    }

    fn visit_unless(
        &mut self,
        buf: &mut String,
        ws: &'a (Ws, Ws),
        args: &'a syn::Expr,
        nodes: &'a [Node<'a>],
    ) {
        validator::unless(args);

        self.handle_ws(&ws.0);
        self.write_buf_writable(buf);

        self.visit_expr(args);
        writeln!(
            buf,
            "if !({}) {{",
            mem::replace(&mut self.buf_t, String::new())
        )
        .unwrap();

        self.scp.push(vec![]);
        self.handle(nodes, buf);
        self.scp.pop();

        self.handle_ws(&ws.1);
        self.write_buf_writable(buf);
        buf.writeln(&"}");
    }

    fn visit_with(
        &mut self,
        buf: &mut String,
        ws: &'a (Ws, Ws),
        args: &'a syn::Expr,
        nodes: &'a [Node<'a>],
    ) {
        validator::scope(args);

        self.handle_ws(&ws.0);
        self.visit_expr(args);
        self.on.push(On::With(self.scp.len()));
        self.scp
            .push(vec![mem::replace(&mut self.buf_t, String::new())]);

        self.handle(nodes, buf);

        self.scp.pop();
        self.on.pop();
        self.handle_ws(&ws.1);
    }

    fn visit_each(
        &mut self,
        buf: &mut String,
        ws: &'a (Ws, Ws),
        args: &'a syn::Expr,
        nodes: &'a [Node<'a>],
    ) {
        validator::each(args);

        self.handle_ws(&ws.0);
        self.write_buf_writable(buf);

        let loop_var = find_loop_var(self.c, self.ctx, self.on_path.clone(), nodes);
        self.visit_expr(args);
        let id = self.scp.len();
        let ctx = if loop_var {
            let ctx = vec![format!("_key_{}", id), format!("_index_{}", id)];
            if let syn::Expr::Range(..) = args {
                writeln!(
                    buf,
                    "for ({}, {}) in ({}).enumerate() {{",
                    ctx[1],
                    ctx[0],
                    &mem::replace(&mut self.buf_t, String::new())
                )
                .unwrap();
            } else {
                writeln!(
                    buf,
                    "for ({}, {}) in (&{}).into_iter().enumerate() {{",
                    ctx[1],
                    ctx[0],
                    &mem::replace(&mut self.buf_t, String::new())
                )
                .unwrap();
            }
            ctx
        } else {
            let ctx = vec![format!("_key_{}", id)];
            if let syn::Expr::Range(..) = args {
                writeln!(
                    buf,
                    "for {} in {} {{",
                    ctx[0],
                    &mem::replace(&mut self.buf_t, String::new())
                )
                .unwrap();
            } else {
                writeln!(
                    buf,
                    "for {} in (&{}).into_iter() {{",
                    ctx[0],
                    &mem::replace(&mut self.buf_t, String::new())
                )
                .unwrap();
            }
            ctx
        };
        self.on.push(On::Each(id));
        self.scp.push(ctx);

        self.handle(nodes, buf);
        self.handle_ws(&ws.1);
        self.write_buf_writable(buf);

        self.scp.pop();
        self.on.pop();
        buf.writeln(&"}");
    }

    fn visit_if(
        &mut self,
        buf: &mut String,
        (pws, cond, block): &'a ((Ws, Ws), syn::Expr, Vec<Node>),
        ifs: &'a [(Ws, syn::Expr, Vec<Node<'a>>)],
        els: &'a Option<(Ws, Vec<Node<'a>>)>,
    ) {
        validator::ifs(cond);

        self.handle_ws(&pws.0);
        self.write_buf_writable(buf);

        self.scp.push(vec![]);
        self.visit_expr(cond);
        writeln!(
            buf,
            "if {} {{",
            mem::replace(&mut self.buf_t, String::new())
        )
        .unwrap();

        self.handle(block, buf);
        self.scp.pop();

        for (ws, cond, block) in ifs {
            validator::ifs(cond);

            self.handle_ws(&ws);
            self.write_buf_writable(buf);

            self.scp.push(vec![]);
            self.visit_expr(cond);
            writeln!(
                buf,
                "}} else if {} {{",
                mem::replace(&mut self.buf_t, String::new())
            )
            .unwrap();

            self.handle(block, buf);
            self.scp.pop();
        }

        if let Some((ws, els)) = els {
            self.handle_ws(ws);
            self.write_buf_writable(buf);

            buf.writeln(&"} else {");

            self.scp.push(vec![]);
            self.handle(els, buf);
            self.scp.pop();
        }

        self.handle_ws(&pws.1);
        self.write_buf_writable(buf);
        buf.writeln(&"}");
    }

    fn visit_partial(&mut self, buf: &mut String, ws: &Ws, path: &str, exprs: &'a [syn::Expr]) {
        let p = self.c.resolve_partial(&self.on_path, path);
        let nodes = self.ctx.get(&p).unwrap();

        let p = mem::replace(&mut self.on_path, p);

        self.flush_ws(ws);

        if exprs.is_empty() {
            self.scp.push(vec![]);
            self.handle(nodes, buf);
            self.scp.pop();
        } else if exprs.len() == 1 {
            // TODO:
            let expr = &exprs[0];
            validator::scope(expr);

            self.visit_expr(expr);
            let parent = mem::replace(
                &mut self.scp,
                vec![vec![mem::replace(&mut self.buf_t, String::new())]],
            );
            self.handle(nodes, buf);
            self.scp = parent;
        }

        self.prepare_ws(ws);

        self.on_path = p;
    }

    fn write_buf_writable(&mut self, buf: &mut String) {
        if self.buf_w.is_empty() {
            return;
        }

        let mut buf_lit = String::new();
        if self.buf_w.iter().all(|w| match w {
            Writable::Lit(_) => true,
            _ => false,
        }) {
            for s in mem::replace(&mut self.buf_w, vec![]) {
                if let Writable::Lit(s) = s {
                    buf_lit.write_str(s).unwrap();
                };
            }
            writeln!(buf, "_fmt.write_str({:#?})?;", &buf_lit).unwrap();
            return;
        }

        for s in mem::replace(&mut self.buf_w, vec![]) {
            match s {
                Writable::Lit(s) => buf_lit.write_str(s).unwrap(),
                Writable::Expr(s, wrapped) => {
                    if !buf_lit.is_empty() {
                        writeln!(
                            buf,
                            "_fmt.write_str({:#?})?;",
                            &mem::replace(&mut buf_lit, String::new())
                        )
                        .unwrap();
                    }

                    buf.push('(');
                    if wrapped || self.s.wrapped {
                        buf.write(&s);
                    } else {
                        // wrap
                        write!(buf, "::wearte::MarkupAsStr::from(&{})", s).unwrap();
                    }
                    buf.writeln(&").fmt(_fmt)?;");
                }
            }
        }

        if !buf_lit.is_empty() {
            writeln!(buf, "_fmt.write_str({:#?})?;", buf_lit).unwrap();
        }
    }

    /* Helper methods for dealing with whitespace nodes */
    fn skip_ws(&mut self) {
        self.next_ws = None;
        self.skip_ws = true;
    }

    // Based on https://github.com/djc/askama
    // Combines `flush_ws()` and `prepare_ws()` to handle both trailing whitespace from the
    // preceding literal and leading whitespace from the succeeding literal.
    fn handle_ws(&mut self, ws: &Ws) {
        self.flush_ws(ws);
        self.prepare_ws(ws);
    }

    // If the previous literal left some trailing whitespace in `next_ws` and the
    // prefix whitespace suppressor from the given argument, flush that whitespace.
    // In either case, `next_ws` is reset to `None` (no trailing whitespace).
    fn flush_ws(&mut self, ws: &Ws) {
        if self.next_ws.is_some() && !ws.0 {
            let val = self.next_ws.unwrap();
            if !val.is_empty() {
                self.buf_w.push(Writable::Lit(val));
            }
        }
        self.next_ws = None;
    }

    // Sets `skip_ws` to match the suffix whitespace suppressor from the given
    // argument, to determine whether to suppress leading whitespace from the
    // next literal.
    fn prepare_ws(&mut self, ws: &Ws) {
        self.skip_ws = ws.1;
    }
}
