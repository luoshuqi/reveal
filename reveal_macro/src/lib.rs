use proc_macro::TokenStream;
use std::collections::HashMap;

use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    Expr, ExprTry, FnArg, Ident, ItemFn, LitStr, parse, parse_macro_input, parse_quote_spanned,
    parse_str, Pat, Token,
};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;

#[proc_macro_attribute]
pub fn error(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as ItemFn);
    let args_replace = parse_macro_input!(attr as ArgsReplace).0;
    match collect_args(&item, args_replace) {
        Ok(args) => {
            (PushContext {
                func: item.sig.ident.to_string(),
                args,
            })
                .visit_item_fn_mut(&mut item);
            quote!(#item).into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}

fn collect_args(
    item: &ItemFn,
    mut args_replace: HashMap<String, Option<Expr>>,
) -> syn::Result<Vec<(String, Expr)>> {
    let mut args = Vec::with_capacity(item.sig.inputs.len());
    for arg in &item.sig.inputs {
        match arg {
            FnArg::Receiver(arg) => match args_replace.remove("self") {
                Some(Some(v)) => args.push(("self".to_string(), v)),
                Some(None) => {}
                None => {
                    args.push((
                        "self".to_string(),
                        parse_quote_spanned! {arg.self_token.span()=> self},
                    ));
                }
            },
            FnArg::Typed(arg) => match *arg.pat {
                Pat::Ident(ref arg) => {
                    let name = arg.ident.to_string();
                    match args_replace.remove(&name) {
                        Some(Some(v)) => args.push((name, v)),
                        Some(None) => {}
                        None => args.push((name, parse(arg.ident.to_token_stream().into())?)),
                    }
                }
                _ => {}
            },
        }
    }

    for (k, v) in args_replace {
        if let Some(v) = v {
            args.push((k, v));
        }
    }

    Ok(args)
}

struct PushContext {
    func: String,
    args: Vec<(String, Expr)>,
}

impl VisitMut for PushContext {
    fn visit_expr_try_mut(&mut self, i: &mut ExprTry) {
        let expr = &i.expr;
        let source = expr.to_token_stream().to_string();
        let mut args = Vec::with_capacity(self.args.len());
        for (name, value) in &self.args {
            args.push(quote_spanned! {value.span()=>
                (#name, format!("{:?}", #value))
            })
        }

        let func = &self.func;
        i.expr = parse_quote_spanned! {expr.span()=>
            #expr.map_err(|__e| { let mut __e = Box::<::reveal::Error>::from(__e); __e.push_context(file!(), line!(), #func, vec![#(#args),*], #source); __e })
        }
    }
}

struct ArgsReplace(HashMap<String, Option<Expr>>);

impl Parse for ArgsReplace {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = HashMap::new();
        while !input.is_empty() {
            let name: Ident = input.call(Ident::parse_any)?;
            let value: Option<Expr> = if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                let s = value.value();
                if s == "_" {
                    None
                } else {
                    let ts = respan(parse_str(&s)?, value.span());
                    Some(parse(ts.into())?)
                }
            } else {
                Some(parse(name.to_token_stream().into())?)
            };
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            args.insert(name.to_string(), value);
        }
        Ok(Self(args))
    }
}

fn respan(ts: proc_macro2::TokenStream, span: Span) -> proc_macro2::TokenStream {
    ts.into_token_stream()
        .into_iter()
        .map(|tt| set_span(tt, span))
        .collect()
}

fn set_span(mut tt: proc_macro2::TokenTree, span: Span) -> proc_macro2::TokenTree {
    tt.set_span(span);
    if let proc_macro2::TokenTree::Group(ref mut g) = tt {
        *g = proc_macro2::Group::new(g.delimiter(), respan(g.stream(), span));
    }
    tt
}
