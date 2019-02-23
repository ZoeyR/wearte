use syn;
use syn::visit::Visit;

use std::{mem, path::PathBuf};

use super::{Context, Struct};

use crate::{
    append_extension,
    parser::{Helper, Node},
};

pub(super) fn find_loop_var(s: &Struct, ctx: Context, path: PathBuf, nodes: &[Node]) -> bool {
    FindEach::new(s, ctx, path).find(nodes)
}

// Find {{ index }} {{ index1 }} {{ first }} {{ last }} {{ _index_[0-9] }}
struct FindEach<'a> {
    loop_var: bool,
    s: &'a Struct<'a>,
    ctx: Context<'a>,
    on_path: PathBuf,
}

impl<'a> FindEach<'a> {
    fn new<'n>(s: &'n Struct<'n>, ctx: Context<'n>, on_path: PathBuf) -> FindEach<'n> {
        FindEach {
            s,
            ctx,
            on_path,
            loop_var: false,
        }
    }

    pub fn find(&mut self, nodes: &'a [Node]) -> bool {
        for n in nodes {
            match n {
                Node::Let(expr) => self.visit_stmt(expr),
                Node::Expr(_, expr) | Node::Safe(_, expr) => self.visit_expr(expr),
                Node::Helper(h) => match h {
                    Helper::If((_, first, block), else_if, els) => {
                        // TODO: super deep
                        // TODO: super or only when loop
                        self.visit_expr(first);
                        if self.loop_var {
                            break;
                        }
                        self.find(block);
                        for (_, e, b) in else_if {
                            if self.loop_var {
                                break;
                            }

                            self.visit_expr(e);
                            self.find(b);
                        }
                        if self.loop_var {
                            break;
                        }
                        if let Some((_, els)) = els {
                            self.find(els);
                        }
                    }
                    Helper::With(_, e, b) => {
                        self.visit_expr(e);
                        self.find(b);
                    }
                    _ => (),
                },
                Node::Partial(_, path) => {
                    let mut p = self.on_path.clone();
                    p.pop();
                    p.push(append_extension(&self.s.path, path));
                    let nodes = self.ctx.get(&p).unwrap();

                    let parent = mem::replace(&mut self.on_path, p);

                    self.find(nodes);

                    self.on_path = parent;
                }
                Node::Lit(..) | Node::Comment(_) => (),
            }
            if self.loop_var {
                break;
            }
        }
        self.loop_var
    }
}

impl<'a> Visit<'a> for FindEach<'a> {
    fn visit_expr_path(&mut self, i: &'a syn::ExprPath) {
        if !self.loop_var {
            if i.path.segments.len() == 1 {
                let ident: &str = &i.path.segments[0].ident.to_string();
                match ident {
                    "index" | "index0" | "first" | "last" => self.loop_var = true,
                    ident => {
                        let ident = ident.as_bytes();
                        if 7 < ident.len()
                            && &ident[0..7] == b"_index_"
                            && ident[7].is_ascii_digit()
                        {
                            self.loop_var = true;
                        }
                    }
                }
            }
        }
    }
}

// TODO: coverage
