use proc_macro::TokenStream;

use quote::quote;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::visit_mut::{VisitMut, visit_expr_try_mut, visit_impl_item_fn_mut};
use syn::{ExprTry, ImplItemFn, ItemFn, ItemImpl, Type, parse_macro_input, parse_quote_spanned};

#[proc_macro_attribute]
pub fn chain_err(_: TokenStream, input: TokenStream) -> TokenStream {
    match parse_macro_input!(input as Item) {
        Item::Fn(mut f) => {
            ChainErr::new(None, Some(f.sig.ident.to_string())).visit_item_fn_mut(&mut f);
            quote!(#f).into()
        }
        Item::Impl(mut i) => {
            ChainErr::new(Some(i.self_ty.clone()), None).visit_item_impl_mut(&mut i);
            quote!(#i).into()
        }
    }
}

enum Item {
    Fn(ItemFn),
    Impl(ItemImpl),
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ahead = input.fork();
        match ahead.parse::<ItemFn>() {
            Ok(v) => {
                input.advance_to(&ahead);
                Ok(Item::Fn(v))
            }
            Err(e) => match input.parse::<ItemImpl>() {
                Ok(v) => Ok(Item::Impl(v)),
                Err(mut e1) => {
                    e1.combine(e);
                    Err(e1)
                }
            },
        }
    }
}

struct ChainErr {
    self_ty: Option<Box<Type>>,
    ident: Option<String>,
}

impl ChainErr {
    fn new(self_ty: Option<Box<Type>>, ident: Option<String>) -> Self {
        Self { self_ty, ident }
    }
}

impl VisitMut for ChainErr {
    fn visit_expr_try_mut(&mut self, i: &mut ExprTry) {
        let ident = self.ident.as_ref().unwrap();
        let module = match self.self_ty {
            Some(ref v) => quote!(std::any::type_name::<#v>()),
            None => quote!(module_path!()),
        };
        let expr = &i.expr;
        *i.expr = parse_quote_spanned! {expr.span() =>
            #expr.map_err(|err| reveal::Error::chain(err, file!(), line!(), #ident, #module))
        };
        visit_expr_try_mut(self, i);
    }

    fn visit_impl_item_fn_mut(&mut self, i: &mut ImplItemFn) {
        let mut indices = vec![];
        for (i, attr) in i.attrs.iter().enumerate() {
            if attr.path().is_ident("chain_err") {
                indices.push(i);
            }
        }
        if indices.is_empty() {
            return;
        }
        for idx in indices {
            i.attrs.remove(idx);
        }
        self.ident = Some(i.sig.ident.to_string());
        visit_impl_item_fn_mut(self, i);
    }
}
