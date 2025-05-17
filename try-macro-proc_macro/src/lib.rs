use std::mem::replace;

use proc_macro::{Delimiter::Brace, Group, Span, TokenStream, TokenTree};
use quote::{quote_spanned, ToTokens};
use syn::{
    spanned::Spanned,
    visit_mut::{visit_expr_mut, VisitMut},
    Block, Expr, ExprTry, Item,
};

fn default_expr_try() -> ExprTry {
    ExprTry {
        attrs: vec![],
        expr: Box::new(Expr::PLACEHOLDER),
        question_token: Default::default(),
    }
}

struct Visitor;

impl VisitMut for Visitor {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        let Expr::Try(expr_try) = node else {
            visit_expr_mut(self, node);
            return;
        };
        let ExprTry {
            attrs,
            mut expr,
            question_token,
        } = replace(expr_try, default_expr_try());

        self.visit_expr_mut(&mut expr);

        *node = syn::parse(
            quote_spanned! { question_token.span() =>
                #(#attrs)*
                match ::try_macro::Try::branch(#expr) {
                    ::core::ops::ControlFlow::Continue(value) => value,
                    ::core::ops::ControlFlow::Break(err) => {
                        return ::try_macro::FromResidual::from_residual(err);
                    }
                }
            }.into(),
        ).unwrap();
    }
}

/// Replace `?` for user `Try::branch`
///
/// Apply on Item, e.g `#[try_macro] fn foo() {}`
#[proc_macro_attribute]
pub fn try_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Some(attr) = attr.into_iter().next() {
        return syn::Error::new(
            attr.span().into(),
            "invalid attribute input",
        ).into_compile_error().into();
    }
    syn::parse::<Item>(item)
        .map(|mut item| {
            Visitor.visit_item_mut(&mut item);
            item.into_token_stream()
        })
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

/// Replace `?` for user `Try::branch` in block
///
/// try_macro_block! {}
#[proc_macro]
pub fn try_macro_block(stream: TokenStream) -> TokenStream {
    let braced = TokenStream::from_iter([
        TokenTree::from(Group::new(Brace, stream))
    ]);
    syn::parse::<Block>(braced)
        .map(|mut block| {
            Visitor.visit_block_mut(&mut block);
            block.into_token_stream()
        })
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}


/// ```
/// try_macro_proc_macro::__test!();
/// ```
#[doc(hidden)]
#[proc_macro]
pub fn __test(_: TokenStream) -> TokenStream {
    use quote::quote;

    let mut fails = vec![];
    macro_rules! eq {
        ($name:ident : {$($a:tt)*} => {$($b:tt)*}) => { #[allow(unused)] fn $name() {} {
            let name = stringify!($name);

            let (a, b) = (quote! { $($a)* }, quote! { $($b)* });
            let a = TokenStream::from(try_macro(TokenStream::new(), a.into()));
            let b = TokenStream::from(b);

            let a = a.to_string();
            let b = b.to_string();

            if a != b {
                fails.push(format!("`{name}` failed\n left: {a}\nright: {b}"))
            }
        }};
    }

    eq!(it_works: {
        fn foo() {
            x?
        }
    } => {
        fn foo() {
            match ::try_macro::Try::branch(x) {
                ::core::ops::ControlFlow::Continue(value) => value,
                ::core::ops::ControlFlow::Break(err) => {
                    return ::try_macro::FromResidual::from_residual(err);
                }
            }
        }
    });

    eq!(nesting: {
        fn foo() {
            x??
        }
    } => {
        fn foo() {
            match ::try_macro::Try::branch(match ::try_macro::Try::branch(x) {
                ::core::ops::ControlFlow::Continue(value) => value,
                ::core::ops::ControlFlow::Break(err) => {
                    return ::try_macro::FromResidual::from_residual(err);
                }
            }) {
                ::core::ops::ControlFlow::Continue(value) => value,
                ::core::ops::ControlFlow::Break(err) => {
                    return ::try_macro::FromResidual::from_residual(err);
                }
            }
        }
    });

    eq!(inner: {
        fn foo() {
            x?+2
        }
    } => {
        fn foo() {
            (match ::try_macro::Try::branch(x) {
                ::core::ops::ControlFlow::Continue(value) => value,
                ::core::ops::ControlFlow::Break(err) => {
                    return ::try_macro::FromResidual::from_residual(err);
                }
            })+2
        }
    });

    if fails.is_empty() {
        return TokenStream::new();
    }
    syn::Error::new(Span::call_site().into(), fails.join("\n\n"))
        .into_compile_error()
        .into()
}
