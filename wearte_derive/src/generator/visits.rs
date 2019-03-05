use syn::visit::{self, Visit};
use syn::{self, punctuated::Punctuated, PathSegment};

use std::{fmt::Write, mem, str};

use super::{EWrite, Generator, On};

macro_rules! visit_attrs {
    ($_self:ident, $attrs:ident) => {
        for it in $attrs {
            $_self.visit_attribute(it)
        }
    };
}

macro_rules! visit_punctuated {
    ($_self:ident, $ele:expr, $method:ident) => {
        for el in Punctuated::pairs($ele) {
            let it = el.value();
            let punc = el.punct();
            $_self.$method(it);
            $_self.buf_t.write(&quote!(#punc));
        }
    };
}

impl<'a> Visit<'a> for Generator<'a> {
    fn visit_arg_captured(
        &mut self,
        syn::ArgCaptured {
            pat,
            colon_token,
            ty,
        }: &'a syn::ArgCaptured,
    ) {
        self.visit_pat(pat);
        self.buf_t.write(&quote!(#colon_token#ty));
    }

    fn visit_arm(
        &mut self,
        syn::Arm {
            attrs,
            leading_vert,
            pats,
            guard,
            fat_arrow_token,
            body,
            comma,
        }: &'a syn::Arm,
    ) {
        visit_attrs!(self, attrs);
        if let Some(_) = leading_vert {
            panic!("Not available")
        }
        if let Some(_) = guard {
            panic!("Not available")
        }

        self.scp.push(vec![]);
        visit_punctuated!(self, pats, visit_pat);
        self.buf_t.write(&quote!(#fat_arrow_token));
        self.visit_expr(body);
        self.buf_t.writeln(&quote!(#comma));
        self.scp.pop();
    }

    fn visit_attribute(&mut self, _i: &'a syn::Attribute) {
        panic!("Not available attributes in a template expression");
    }

    fn visit_bin_op(&mut self, i: &'a syn::BinOp) {
        write!(self.buf_t, " {} ", quote!(#i)).unwrap();
    }

    fn visit_block(&mut self, i: &'a syn::Block) {
        self.scp.push(vec![]);
        self.buf_t.write(&" { ");
        visit::visit_block(self, i);
        self.buf_t.write(&" }");
        self.scp.pop();
    }

    fn visit_expr_array(&mut self, syn::ExprArray { attrs, elems, .. }: &'a syn::ExprArray) {
        visit_attrs!(self, attrs);
        self.buf_t.push('[');
        for i in elems {
            self.visit_expr(i);
        }
        self.buf_t.push(']');
    }

    fn visit_expr_assign(
        &mut self,
        syn::ExprAssign {
            attrs,
            left,
            eq_token,
            right,
        }: &'a syn::ExprAssign,
    ) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&quote!(#left #eq_token));
        self.visit_expr(right);
    }

    fn visit_expr_assign_op(
        &mut self,
        syn::ExprAssignOp {
            attrs,
            left,
            op,
            right,
        }: &'a syn::ExprAssignOp,
    ) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&quote!(#left #op));
        self.visit_expr(right);
    }

    fn visit_expr_async(&mut self, _i: &'a syn::ExprAsync) {
        panic!("Not available async in a template expression");
    }

    fn visit_expr_block(&mut self, i: &'a syn::ExprBlock) {
        let last = mem::replace(&mut self.will_wrap, false);
        visit::visit_expr_block(self, i);
        self.will_wrap = last;
    }

    fn visit_expr_box(
        &mut self,
        syn::ExprBox {
            attrs,
            box_token,
            expr,
        }: &'a syn::ExprBox,
    ) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&quote!(#box_token));
        self.visit_expr(expr);
    }

    fn visit_expr_break(
        &mut self,
        syn::ExprBreak {
            attrs,
            break_token,
            label,
            expr,
        }: &'a syn::ExprBreak,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, "{} ", quote!(#break_token #label)).unwrap();
        if let Some(expr) = expr {
            self.visit_expr(expr)
        }
    }

    fn visit_expr_call(
        &mut self,
        syn::ExprCall {
            attrs, func, args, ..
        }: &'a syn::ExprCall,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, "{}(", quote!(#func)).unwrap();
        let last = mem::replace(&mut self.will_wrap, false);
        visit_punctuated!(self, args, visit_expr);
        self.will_wrap = last;
        self.buf_t.push(')');
    }

    fn visit_expr_cast(
        &mut self,
        syn::ExprCast {
            attrs,
            expr,
            as_token,
            ty,
        }: &'a syn::ExprCast,
    ) {
        visit_attrs!(self, attrs);
        let last = mem::replace(&mut self.will_wrap, false);
        self.visit_expr(expr);
        self.will_wrap = last;
        write!(self.buf_t, " {} ", quote!(#as_token #ty)).unwrap();
    }

    fn visit_expr_closure(
        &mut self,
        syn::ExprClosure {
            attrs,
            asyncness,
            movability,
            capture,
            inputs,
            output,
            body,
            ..
        }: &'a syn::ExprClosure,
    ) {
        visit_attrs!(self, attrs);

        write!(self.buf_t, "{} |", quote!(#asyncness #movability #capture)).unwrap();
        self.scp.push(vec![]);
        visit_punctuated!(self, inputs, visit_fn_arg);
        self.buf_t.write(&"| ");
        self.buf_t.write(&quote!(#output));
        self.visit_expr(body);
        self.scp.pop();
    }

    fn visit_expr_continue(&mut self, i: &'a syn::ExprContinue) {
        self.buf_t.write(&quote!(#i));
    }

    fn visit_expr_field(
        &mut self,
        syn::ExprField {
            attrs,
            base,
            member,
            ..
        }: &'a syn::ExprField,
    ) {
        visit_attrs!(self, attrs);

        self.visit_expr(base);
        write!(self.buf_t, ".{}", quote!(#member)).unwrap();
    }

    fn visit_expr_for_loop(
        &mut self,
        syn::ExprForLoop {
            attrs,
            label,
            for_token,
            pat,
            expr,
            body,
            ..
        }: &'a syn::ExprForLoop,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, " {} ", &quote!(#label #for_token)).unwrap();
        self.scp.push(vec![]);
        self.visit_pat(pat);
        let last = self.scp.pop().unwrap();
        self.buf_t.write(&" in ");
        self.visit_expr(expr);
        self.scp.push(last);
        self.visit_block(body);
        self.scp.pop();
    }

    fn visit_expr_if(
        &mut self,
        syn::ExprIf {
            attrs,
            cond,
            then_branch,
            else_branch,
            ..
        }: &'a syn::ExprIf,
    ) {
        visit_attrs!(self, attrs);

        self.buf_t.write(&" if ");
        self.scp.push(vec![]);

        let last = mem::replace(&mut self.will_wrap, false);
        self.visit_expr(cond);
        self.will_wrap = last;

        self.visit_block(then_branch);
        self.scp.pop();

        if let Some((_, it)) = else_branch {
            self.buf_t.write(&" else");
            self.visit_expr(it);
        };
    }

    fn visit_expr_in_place(&mut self, _i: &'a syn::ExprInPlace) {
        panic!("Not available in place in a template expression");
    }

    fn visit_expr_index(
        &mut self,
        syn::ExprIndex {
            attrs, expr, index, ..
        }: &'a syn::ExprIndex,
    ) {
        visit_attrs!(self, attrs);
        self.visit_expr(expr);
        self.buf_t.write(&quote!([#index]));
    }

    fn visit_expr_let(
        &mut self,
        syn::ExprLet {
            attrs, expr, pats, ..
        }: &'a syn::ExprLet,
    ) {
        visit_attrs!(self, attrs);
        let last_w = mem::replace(&mut self.will_wrap, false);

        self.buf_t.write_str("let ").unwrap();

        visit_punctuated!(self, pats, visit_pat);
        let last = self.scp.pop().unwrap();

        self.buf_t.push(' ');
        self.buf_t.push('=');

        self.visit_expr(expr);
        self.scp.push(last);

        self.will_wrap = last_w;
    }

    fn visit_expr_loop(
        &mut self,
        syn::ExprLoop {
            attrs,
            label,
            loop_token,
            body,
        }: &'a syn::ExprLoop,
    ) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&quote!(#label #loop_token));
        let last = mem::replace(&mut self.will_wrap, false);
        self.visit_block(body);
        self.will_wrap = last;
    }

    fn visit_expr_match(
        &mut self,
        syn::ExprMatch {
            attrs,
            match_token,
            expr,
            arms,
            ..
        }: &'a syn::ExprMatch,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, " {} ", quote!(#match_token)).unwrap();
        self.visit_expr(expr);
        self.buf_t.push('{');
        for i in arms {
            self.visit_arm(i);
        }
        self.buf_t.push('}');
    }

    fn visit_expr_method_call(
        &mut self,
        syn::ExprMethodCall {
            attrs,
            receiver,
            method,
            turbofish,
            args,
            ..
        }: &'a syn::ExprMethodCall,
    ) {
        visit_attrs!(self, attrs);
        self.visit_expr(receiver);
        write!(self.buf_t, ".{}(", quote!(#method#turbofish)).unwrap();
        let last = mem::replace(&mut self.will_wrap, false);
        visit_punctuated!(self, args, visit_expr);
        self.will_wrap = last;
        self.buf_t.push(')');
    }

    fn visit_expr_path(&mut self, syn::ExprPath { attrs, qself, path }: &'a syn::ExprPath) {
        debug_assert!(!self.scp.is_empty() && !self.scp[0].is_empty());
        visit_attrs!(self, attrs);
        if qself.is_some() {
            panic!("Not available QSelf in a template expression");
        }

        macro_rules! wrap_and_write {
            ($($t:tt)+) => {{
                if self.will_wrap {
                    self.wrapped = true;
                }
                return write!(self.buf_t, $($t)+).unwrap();
            }};
        }

        macro_rules! each_var {
            ($ident:expr, $j:expr) => {
                match $ident {
                    "index0" => wrap_and_write!("{}", self.scp[$j][1]),
                    "index" => wrap_and_write!("({} + 1)", self.scp[$j][1]),
                    "first" => wrap_and_write!("({} == 0)", self.scp[$j][1]),
                    "last" => wrap_and_write!(
                        "(({}).len() == ({} + 1))",
                        self.scp[$j][0],
                        self.scp[$j][1]
                    ),
                    "key" => return self.buf_t.write(&self.scp[$j][0]),
                    _ => (),
                }
            };
        }

        if path.segments.len() == 1 {
            let ident: &str = &path.segments[0].ident.to_string();

            if ident.chars().all(|x| x.is_ascii_uppercase() || x.eq(&'_')) {
                self.buf_t.write(&ident);
            } else if ident == "self" {
                self.buf_t.write(&self.scp[0][0]);
            } else if self.scp.iter().all(|v| v.iter().all(|e| ident.ne(e))) {
                if self.on.is_empty() {
                    write!(self.buf_t, "{}.{}", self.scp[0][0], ident).unwrap()
                } else {
                    if let Some(j) = self.on.iter().rev().find_map(|x| match x {
                        On::Each(j) => Some(j),
                        _ => None,
                    }) {
                        debug_assert!(self.scp.get(*j).is_some() && !self.scp[*j].is_empty());
                        each_var!(ident, *j);
                    }

                    match self.on.last() {
                        // self
                        None => write!(self.buf_t, "{}.{}", self.scp[0][0], ident).unwrap(),
                        Some(On::Each(j)) | Some(On::With(j)) => {
                            debug_assert!(self.scp.get(*j).is_some() && !self.scp[*j].is_empty());
                            return write!(self.buf_t, "{}.{}", self.scp[*j][0], ident).unwrap();
                        }
                    }
                }
            } else {
                self.buf_t.write(&ident);
            }
        } else {
            if let Some((j, ident)) = is_super(&path.segments) {
                if self.on.is_empty() {
                    panic!("use super at top");
                } else if self.on.len() == j {
                    write!(self.buf_t, "{}.{}", self.scp[0][0], ident).unwrap();
                } else if j < self.on.len() {
                    match self.on[self.on.len() - j - 1] {
                        On::With(j) => {
                            debug_assert!(self.scp.get(j).is_some() && !self.scp[j].is_empty());
                            write!(self.buf_t, "{}.{}", self.scp[j][0], ident).unwrap();
                        }
                        On::Each(j) => {
                            debug_assert!(self.scp.get(j).is_some() && !self.scp[j].is_empty());
                            each_var!(ident.as_ref(), j);
                            write!(self.buf_t, "{}.{}", self.scp[j][0], ident).unwrap();
                        }
                    }
                } else {
                    panic!("use super without parent")
                }
            } else {
                self.buf_t.write(&quote!(#path));
            }
        }
    }

    fn visit_expr_reference(&mut self, i: &'a syn::ExprReference) {
        let m = i.mutability;
        self.buf_t.write(&quote!(& #m));
        visit::visit_expr_reference(self, i);
    }

    fn visit_expr_repeat(&mut self, i: &'a syn::ExprRepeat) {
        self.buf_t.write(&quote!(#i));
    }

    fn visit_expr_return(&mut self, syn::ExprReturn { attrs, expr, .. }: &'a syn::ExprReturn) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&"return ");
        if let Some(expr) = expr {
            self.visit_expr(expr);
        }
    }

    fn visit_expr_struct(
        &mut self,
        syn::ExprStruct {
            attrs,
            path,
            fields,
            dot2_token,
            rest,
            ..
        }: &'a syn::ExprStruct,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, " {} {{", quote!(#path)).unwrap();
        visit_punctuated!(self, fields, visit_field_value);
        write!(self.buf_t, " {} }}", quote!(#dot2_token#rest)).unwrap();
    }

    fn visit_expr_try(&mut self, syn::ExprTry { attrs, expr, .. }: &'a syn::ExprTry) {
        visit_attrs!(self, attrs);
        self.visit_expr(expr);
        self.buf_t.push('?');
    }

    fn visit_expr_try_block(&mut self, _i: &'a syn::ExprTryBlock) {
        panic!("Not allowed try block expression in a template expression");
    }

    fn visit_expr_tuple(&mut self, syn::ExprTuple { attrs, elems, .. }: &'a syn::ExprTuple) {
        visit_attrs!(self, attrs);

        self.buf_t.push('(');
        let last = mem::replace(&mut self.will_wrap, false);
        visit_punctuated!(self, elems, visit_expr);
        self.will_wrap = last;
        self.buf_t.push(')');
    }

    fn visit_expr_unsafe(&mut self, syn::ExprUnsafe { attrs, block, .. }: &'a syn::ExprUnsafe) {
        visit_attrs!(self, attrs);
        self.buf_t.write(&"unsafe ");
        self.visit_block(block);
    }

    fn visit_expr_verbatim(&mut self, _i: &'a syn::ExprVerbatim) {
        panic!("Not allowed verbatim expression in a template expression");
    }

    fn visit_expr_while(
        &mut self,
        syn::ExprWhile {
            attrs,
            label,
            while_token,
            cond,
            body,
        }: &'a syn::ExprWhile,
    ) {
        visit_attrs!(self, attrs);
        let last = mem::replace(&mut self.will_wrap, false);
        write!(self.buf_t, " {} ", quote!(#label #while_token)).unwrap();
        self.visit_expr(cond);
        self.visit_block(body);
        self.will_wrap = last;
    }

    fn visit_expr_yield(&mut self, _i: &'a syn::ExprYield) {
        panic!("Not allowed yield expression in a template expression");
    }

    fn visit_field_value(
        &mut self,
        syn::FieldValue {
            attrs,
            member,
            colon_token,
            expr,
        }: &'a syn::FieldValue,
    ) {
        visit_attrs!(self, attrs);
        write!(self.buf_t, " {} ", quote!(#member #colon_token)).unwrap();
        self.visit_expr(expr)
    }

    fn visit_lit(&mut self, i: &'a syn::Lit) {
        use syn::Lit::*;
        match i {
            Int(_) | Float(_) | Bool(_) => {
                if self.will_wrap {
                    self.wrapped = true;
                }
            }
            _ => (),
        }
        self.buf_t.write(&quote!(#i));
    }

    fn visit_local(
        &mut self,
        syn::Local {
            attrs,
            pats,
            init,
            ty,
            ..
        }: &'a syn::Local,
    ) {
        visit_attrs!(self, attrs);

        let last = mem::replace(&mut self.will_wrap, false);
        self.scp.push(vec![]);

        self.buf_t.write(&"let ");

        for el in Punctuated::pairs(pats) {
            let it = el.value();
            self.visit_pat(it)
        }
        let scope = self.scp.pop().unwrap();

        if let Some((_, ty)) = ty {
            self.buf_t.write(&quote!(: #ty));
        }

        self.buf_t.push('=');

        if let Some((_, expr)) = init {
            self.visit_expr(expr);
        }
        self.buf_t.push(';');

        self.will_wrap = last;
        self.scp.last_mut().unwrap().extend(scope);
    }

    fn visit_macro(&mut self, i: &'a syn::Macro) {
        self.buf_t.write(&quote!(#i));
    }

    fn visit_pat_ident(
        &mut self,
        syn::PatIdent {
            by_ref,
            mutability,
            ident,
            subpat,
        }: &'a syn::PatIdent,
    ) {
        if let Some(_) = subpat {
            panic!("Subpat is not allowed");
        }

        self.buf_t.write(&quote!(#by_ref #mutability #ident));
        self.scp
            .last_mut()
            .expect("someone scope")
            .push(ident.to_string());
    }

    fn visit_pat_tuple(
        &mut self,
        syn::PatTuple {
            front,
            dot2_token,
            comma_token,
            back,
            ..
        }: &'a syn::PatTuple,
    ) {
        self.buf_t.push('(');
        visit_punctuated!(self, front, visit_pat);
        self.buf_t.write(&quote!( #dot2_token #comma_token));
        visit_punctuated!(self, back, visit_pat);
        self.buf_t.push(')');
    }

    fn visit_pat_tuple_struct(
        &mut self,
        syn::PatTupleStruct { path, pat }: &'a syn::PatTupleStruct,
    ) {
        self.buf_t.write(&quote!(#path));
        self.visit_pat_tuple(pat)
    }

    fn visit_pat_wild(&mut self, i: &'a syn::PatWild) {
        self.buf_t.write(&quote!(#i));
    }

    fn visit_range_limits(&mut self, i: &'a syn::RangeLimits) {
        use syn::RangeLimits::*;
        match i {
            HalfOpen(i) => {
                self.buf_t.write(&quote!(#i));
            }
            Closed(i) => {
                self.buf_t.write(&quote!(#i));
            }
        }
    }

    fn visit_stmt(&mut self, i: &'a syn::Stmt) {
        use syn::Stmt::*;
        let last = mem::replace(&mut self.will_wrap, false);
        match i {
            Local(i) => {
                self.visit_local(i);
            }
            Item(i) => {
                self.visit_item(i);
            }
            Expr(i) => {
                self.visit_expr(i);
            }
            Semi(i, semi) => {
                self.visit_expr(i);
                self.buf_t.write(&quote!(#semi));
            }
        }
        self.will_wrap = last;
    }

    fn visit_un_op(&mut self, i: &'a syn::UnOp) {
        self.buf_t.write(&quote!(#i));
    }
}

pub(super) fn is_super<S>(i: &Punctuated<PathSegment, S>) -> Option<(usize, String)> {
    let idents: Vec<String> = Punctuated::pairs(i)
        .map(|x| x.value().ident.to_string())
        .collect();
    let len = idents.len();
    let ident = idents[len - 1].clone();
    let idents: &[String] = &idents[0..len - 1];

    if idents.iter().all(|x| x.eq("super")) {
        Some((idents.len(), ident))
    } else {
        None
    }
}
