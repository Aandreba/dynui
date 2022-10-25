#![feature(is_some_and)]
#![feature(iterator_try_collect)]

use html::{Element, Elements, ElementEnd, ElementAttribute, Html};
use proc_macro2::{TokenStream};
use quote::{quote, ToTokens, quote_spanned};
use syn::{parse_macro_input, ItemFn, Signature, spanned::Spanned, Pat, FnArg, PatType};
mod html;

#[proc_macro]
pub fn html (items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut element = parse_macro_input!(items as Elements).0;

    if element.len() == 1 {
        return html_html(element.swap_remove(0)).into();
    }

    let element = element.into_iter().map(html_html);
    
    quote! {
        (|| {
            let mut __fragment__ = dynui::web_sys::DocumentFragment::new()?;
            #(
                dynui::web_sys::Node::append_child(
                    &__fragment__,
                    &dynui::component::Component::render(#element)?
                )?;
            )*
            return dynui::Result::Ok(__fragment__)
        })()
    }.into()
}

fn html_html (html: Html) -> TokenStream {
    return match html {
        Html::Element(x) => html_element(x),
        Html::Expr(x) => quote! {
            dynui::component::Component::render(#x)
        }
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
            let value = html_html(child);
            children.push(quote! {
                dynui::web_sys::Node::append_child(
                    &r#__element__,
                    &dynui::component::Component::render(#value)?
                )?;
            });
        }
    }

    let tokens = match path.get_ident() {
        Some(x) if x.to_string().starts_with(char::is_lowercase) => {
            let props = attrs.into_iter()
                .map(|ElementAttribute { pat, expr, .. }| {
                    let ident = match pat {
                        Pat::Ident(pat) => pat.ident,
                        other => return syn::Error::new_spanned(other, "only identity patterns are allowed as primitive props").to_compile_error()
                    };

                    quote! {
                        dynui::web_sys::Element::set_attribute_node(
                            &r#__element__,
                            &dynui::create_attribute(
                                stringify!(#ident),
                                #expr
                            )?
                        )?;
                    }
                });

            quote! {
                let mut r#__element__ = dynui::context().document.create_element(stringify!(#path))?;
                #(#props)*
            }
        },

        _ => {
            let props = attrs.into_iter()
                .map(|ElementAttribute { pat, expr, .. }| quote! { #pat: #expr });

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

    let props = match inputs.iter().map(|arg| match arg {
        FnArg::Typed(pat_ty @ PatType { attrs, pat, colon_token, ty  }) => Ok(quote_spanned! { pat_ty.span() => #(#attrs)* #vis #pat #colon_token #ty }),
        FnArg::Receiver(recv) => Err(syn::Error::new_spanned(recv, "only typed arguments are allowed as component props"))
    }).try_collect::<Vec<_>>() {
        Ok(x) => x,
        Err(e) => return e.to_compile_error().into()
    };

    let render_inputs = match inputs.iter().map(|arg| match arg {
        FnArg::Typed(ty @ PatType { attrs, pat, .. }) => Ok(quote_spanned! { ty.span() => #(#attrs)* #pat }),
        FnArg::Receiver(recv) => Err(syn::Error::new_spanned(recv, "only typed arguments are allowed as component props"))
    }).try_collect::<Vec<_>>() {
        Ok(x) => x,
        Err(e) => return e.to_compile_error().into()
    };

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
            #vis fn new (#inputs) -> Self {
                return Self {
                    #(#render_inputs),*
                }
            }
        }

        impl #impl_generics #constness dynui::component::Component for #ident #ty_generics #where_generics {
            fn render (self) -> dynui::Result<dynui::web_sys::Node> {
                let Self { #(#render_inputs),* } = self;
                return <#output>::render((move || #block)())
            }
        }
    }.into()
}
