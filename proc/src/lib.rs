use html::{Element, OpenElement, Elements};
use proc_macro2::{TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn, Signature, spanned::Spanned};

mod html;

#[proc_macro]
pub fn html (items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let element = parse_macro_input!(items as Elements).0.into_iter().map(html_element);
    
    quote! {
        (|| {
            let mut fragment = ::web_sys::DocumentFragment::new()?;
            #(#element)*
        })()
    }.into()
}

fn html_element (Element { open, children, close }: Element) -> TokenStream {
    let OpenElement { left, path, attrs, shift, right } = open;

    if path != close.path {
        return syn::Error::new(
            path.span().join(close.path.span()).unwrap(), 
            format!(
                "invalid tags. expected `{}`, found `{}`",
                path.to_token_stream().to_string(),
                close.path.to_token_stream().to_string()
            )).into_compile_error();
    }

    quote! {

    }
}

#[proc_macro_attribute]
pub fn component (attrs: proc_macro::TokenStream, items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ItemFn { attrs, vis, sig, block } = parse_macro_input!(items as ItemFn);
    let Signature { constness, asyncness, unsafety, ident, generics, inputs, output, .. } = sig;
    let (impl_generics, ty_generics, where_generics) = generics.split_for_impl();

    quote! {
        #(#attrs)*
        #vis struct #ident #impl_generics {

        }
    }.into()
}
