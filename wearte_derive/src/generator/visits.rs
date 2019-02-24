use syn::visit::{self, Visit};
use syn::{self, punctuated::Punctuated};

use std::{fmt::Write, str};

use super::Generator;

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
            write!($_self.buf_t, "{}", quote!(#punc)).unwrap();
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
        write!(self.buf_t, "{}", quote!(#colon_token#ty)).unwrap();
    }

    fn visit_attribute(&mut self, _i: &'a syn::Attribute) {
        panic!("Not available attributes in a template expression");
    }

    fn visit_bin_op(&mut self, i: &'a syn::BinOp) {
        write!(self.buf_t, " {} ", quote!(#i)).unwrap();
    }

    fn visit_block(&mut self, i: &'a syn::Block) {
        self.scp.push(vec![]);
        self.buf_t.write_str(" { ").unwrap();
        visit::visit_block(self, i);
        self.buf_t.write_str(" }").unwrap();
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

    fn visit_expr_assign(&mut self, _i: &'a syn::ExprAssign) {
        unimplemented!();
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
        self.will_wrap = false;
        visit_punctuated!(self, args, visit_expr);
        self.will_wrap = true;
        self.buf_t.push(')');
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
        write!(self.buf_t, "| ").unwrap();
        write!(self.buf_t, "{}", quote!(#output)).unwrap();
        self.visit_expr(body);
        self.scp.pop();
    }

    fn visit_expr_continue(&mut self, i: &'a syn::ExprContinue) {
        write!(self.buf_t, "{}", quote!(#i)).unwrap();
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

    fn visit_expr_for_loop(&mut self, _i: &'a syn::ExprForLoop) {
        panic!("Not available for loop in a template expression");
    }

    fn visit_expr_group(&mut self, _i: &'a syn::ExprGroup) {
        unimplemented!();
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

        self.buf_t.write_str(" if ").unwrap();
        self.scp.push(vec![]);

        self.will_wrap = false;
        self.visit_expr(cond);
        self.will_wrap = true;

        self.visit_block(then_branch);
        self.scp.pop();

        if let Some((_, it)) = else_branch {
            self.buf_t.write_str(" else").unwrap();
            self.visit_expr(it);
        };
    }

    fn visit_expr_index(
        &mut self,
        syn::ExprIndex {
            attrs, expr, index, ..
        }: &'a syn::ExprIndex,
    ) {
        visit_attrs!(self, attrs);
        self.visit_expr(expr);
        write!(self.buf_t, "[{}]", quote!(#index)).unwrap();
    }

    fn visit_expr_let(
        &mut self,
        syn::ExprLet {
            attrs, expr, pats, ..
        }: &'a syn::ExprLet,
    ) {
        visit_attrs!(self, attrs);
        self.will_wrap = false;

        self.buf_t.write_str("let ").unwrap();

        visit_punctuated!(self, pats, visit_pat);
        let last = self.scp.pop().unwrap();

        self.buf_t.push(' ');
        self.buf_t.push('=');

        self.visit_expr(expr);
        self.scp.push(last);

        self.will_wrap = true;
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
        write!(self.buf_t, "{}", quote!(#label #loop_token)).unwrap();
        self.will_wrap = false;
        self.visit_block(body);
        self.will_wrap = true;
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
        self.will_wrap = false;
        visit_punctuated!(self, args, visit_expr);
        self.will_wrap = true;
        self.buf_t.push(')');
    }

    fn visit_expr_path(&mut self, i: &'a syn::ExprPath) {
        // TODO: support any::thing::else
        if i.path.segments.len() == 1 {
            let ident: &str = &i.path.segments[0].ident.to_string();
            self.write_single_path(ident);
        } else {
            unimplemented!();
        }
    }

    fn visit_expr_reference(&mut self, i: &'a syn::ExprReference) {
        let m = i.mutability;
        write!(self.buf_t, "&{}", quote!(#m)).unwrap();
        visit::visit_expr_reference(self, i);
    }

    fn visit_expr_try(&mut self, syn::ExprTry { attrs, expr, .. }: &'a syn::ExprTry) {
        visit_attrs!(self, attrs);
        self.visit_expr(expr);
        self.buf_t.push('?');
    }

    fn visit_expr_tuple(&mut self, syn::ExprTuple { attrs, elems, .. }: &'a syn::ExprTuple) {
        visit_attrs!(self, attrs);

        self.buf_t.push('(');
        self.will_wrap = false;
        visit_punctuated!(self, elems, visit_expr);
        self.will_wrap = true;
        self.buf_t.push(')');
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
        write!(self.buf_t, "{} ", quote!(#i)).unwrap();
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

        self.will_wrap = false;
        self.scp.push(vec![]);

        self.buf_t.write_str("let ").unwrap();

        for el in Punctuated::pairs(pats) {
            let it = el.value();
            self.visit_pat(it)
        }
        let scope = self.scp.pop().unwrap();

        if let Some((_, ty)) = ty {
            write!(self.buf_t, "{} ", quote!(: #ty)).unwrap();
        }

        self.buf_t.push(' ');
        self.buf_t.push('=');

        if let Some((_, expr)) = init {
            self.visit_expr(expr);
        }
        self.buf_t.push(';');

        self.will_wrap = true;
        self.scp.last_mut().unwrap().extend(scope);
    }

    fn visit_macro(&mut self, i: &'a syn::Macro) {
        write!(self.buf_t, "{}", quote!(#i)).unwrap();
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

        write!(self.buf_t, "{}", quote!(#by_ref #mutability #ident)).unwrap();
        self.scp.last_mut().unwrap().push(ident.to_string());
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
        write!(self.buf_t, "{}", quote!(#dot2_token #comma_token)).unwrap();
        visit_punctuated!(self, back, visit_pat);
        self.buf_t.push(')');
    }

    fn visit_pat_tuple_struct(
        &mut self,
        syn::PatTupleStruct { path, pat }: &'a syn::PatTupleStruct,
    ) {
        write!(self.buf_t, "{}", quote!(#path)).unwrap();
        self.visit_pat_tuple(pat)
    }

    fn visit_pat_wild(&mut self, i: &'a syn::PatWild) {
        write!(self.buf_t, "{}", quote!(#i)).unwrap();
    }

    fn visit_range_limits(&mut self, i: &'a syn::RangeLimits) {
        use syn::RangeLimits::*;
        match i {
            HalfOpen(i) => {
                write!(self.buf_t, "{}", quote!(#i)).unwrap();
            }
            Closed(i) => {
                write!(self.buf_t, "{}", quote!(#i)).unwrap();
            }
        }
    }

    fn visit_stmt(&mut self, i: &'a syn::Stmt) {
        use syn::Stmt::*;
        self.will_wrap = false;
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
                write!(self.buf_t, "{}", quote!(#semi)).unwrap();
            }
        }
        self.will_wrap = true;
    }

    fn visit_un_op(&mut self, i: &'a syn::UnOp) {
        write!(self.buf_t, "{}", quote!(#i)).unwrap();
    }
}
