#![feature(is_some_and)]
#![feature(iterator_try_collect)]

use std::ops::Deref;

use html::{Element, Elements, ElementEnd, ElementAttribute, Html};
use proc_macro2::{TokenStream};
use quote::{quote, ToTokens, quote_spanned};
use syn::{parse_macro_input, ItemFn, Signature, spanned::Spanned, Pat, FnArg, PatType, punctuated::Punctuated, Token, Attribute, PatIdent, ext::IdentExt};
mod html;

#[proc_macro]
pub fn html (items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut element: Vec<Html> = parse_macro_input!(items as Elements).0;

    if element.len() == 1 {
        let (attrs, tokens) = html_html(element.swap_remove(0));
        return quote! { #(#attrs)* #tokens }.into()
    }

    let element = element.into_iter().map(|html| {
        let (attrs, tokens) = html_html(html);
        return quote! {
            #(#attrs)*
            dynui::component::Node::append_child(
                &__fragment__,
                dynui::component::Component::render(#tokens)?
            )?;
        }
    });
    
    quote! {
        (|| {
            let mut __fragment__ = dynui::web_sys::DocumentFragment::new()?;
            #(#element)*
            return unsafe {
                dynui::Result::Ok(dynui::component::Node::new(__fragment__))
            }
        })()
    }.into()
}

fn html_html (html: Html) -> (Vec<Attribute>, TokenStream) {
    return match html {
        Html::Element(attrs, x) => {
            let tokens = html_element(x);
            return (attrs, tokens)
        },
        Html::Expr(attrs, x) => (attrs, quote! {
            dynui::component::Component::render(#x)
        })
    }
}

fn html_element (Element { path, attrs, end, .. }: Element) -> TokenStream {
    let mut children = Vec::new();
    if let ElementEnd::Open(close) = end {
        children.reserve(close.children.len());

        if path != close.path {
            return syn::Error::new(
                path.span().join(close.path.span()).unwrap(), 
                format!(
                    "invalid tags. expected `{}`, found `{}`",
                    path.to_token_stream().to_string(),
                    close.path.to_token_stream().to_string()
                )).into_compile_error();
        }

        for child in close.children {
            let (attrs, value) = html_html(child);
            children.push(quote! {
                #(#attrs)*
                dynui::component::Node::append_child(
                    &r#__element__,
                    dynui::component::Component::render(#value)?
                )?;
            });
        }
    }

    let tokens = match path.get_ident() {
        Some(x) if x.to_string().starts_with(char::is_lowercase) => {
            let props = attrs.into_iter()
                .map(|ElementAttribute { attrs, pat, expr, .. }| {
                    let ident = match pat {
                        Pat::Ident(pat) => IdentExt::unraw(&pat.ident),
                        other => return syn::Error::new_spanned(other, "only identity patterns are allowed as primitive props").to_compile_error()
                    };

                    quote! {
                        #(#attrs)*
                        dynui::component::Element::set_attribute(
                            &r#__element__,
                            stringify!(#ident),
                            #expr
                        )?;
                    }
                });

            quote! {
                let mut r#__element__ = dynui::create_element(stringify!(#path))?;
                #(#props)*
            }
        },

        _ => {
            let props = attrs.into_iter()
                .map(|ElementAttribute { attrs, pat, expr, .. }| quote! { #(#attrs)* #pat: #expr });

            quote! {
                let mut r#__element__ = #path { #(#props),* };
            }
        }
    };

    quote! {(|| {
        #tokens
        #(#children)*
        return dynui::Result::Ok(r#__element__)
    })()}
}

#[proc_macro_attribute]
pub fn component (_attrs: proc_macro::TokenStream, items: proc_macro::TokenStream) -> proc_macro::TokenStream {    
    let ItemFn { attrs, vis, sig, block } = parse_macro_input!(items as ItemFn);
    let Signature { constness, asyncness, unsafety, ident, generics, inputs, output, .. } = sig;
    let (impl_generics, ty_generics, where_generics) = generics.split_for_impl();

    let inputs = match inputs.into_iter().map(|arg| match arg {
        FnArg::Typed(pat) => Ok(pat),
        FnArg::Receiver(recv) => Err(syn::Error::new_spanned(recv, "only typed arguments are allowed as component props"))
    }).try_collect::<Punctuated<_, Token![,]>>() {
        Ok(x) => x,
        Err(e) => return e.to_compile_error().into()
    };

    let props = inputs.iter()
        .map(|pat_ty @ PatType { attrs, pat, colon_token, ty }| {
            let ident = match pat.deref() {
                Pat::Ident(pat_ty @ PatIdent { attrs, ident, .. }) => quote_spanned! { pat_ty.span() => #(#attrs)* #ident },
                other => other.to_token_stream()
            };

            quote_spanned! { pat_ty.span() => #(#attrs)* #vis #ident #colon_token #ty }
        }).collect::<Vec<_>>();

    let def_inputs = inputs.iter()
        .map(|pat_ty @ PatType { attrs, pat, colon_token, ty }| {
            let ident = match pat.deref() {
                Pat::Ident(pat_ty @ PatIdent { attrs, ident, .. }) => quote_spanned! { pat_ty.span() => #(#attrs)* #ident },
                other => other.to_token_stream()
            };

            quote_spanned! { pat_ty.span() => #(#attrs)* #ident }
        }).collect::<Vec<_>>();

    let render_inputs = inputs.iter()
        .map(|ty @ PatType { attrs, pat, .. }| {
            quote_spanned! { ty.span() => #(#attrs)* #pat }
        })
        .collect::<Vec<_>>();

    let output = match output {
        out @ syn::ReturnType::Default => quote_spanned! { out.span() => dynui::component::Component },
        syn::ReturnType::Type(_, ty) => quote_spanned! { ty.span() => #ty as dynui::component::Component },
    };

    quote! {
        #(#attrs)*
        #vis struct #ident #impl_generics {
            #(#props),*
        }

        impl #impl_generics #ident #ty_generics #where_generics {
            #[inline]
            #[allow(unused_mut)]
            #vis fn new (#inputs) -> Self {
                return Self {
                    #(#def_inputs),*
                }
            }
        }

        impl #impl_generics #constness dynui::component::Component for #ident #ty_generics #where_generics {
            fn render (self) -> dynui::Result<dynui::component::Node> {
                let Self { #(#render_inputs),* } = self;
                return <#output>::render((move || #block)())
            }
        }
    }.into()
}