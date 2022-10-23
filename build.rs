use std::{process::Command, path::Path, fs::{read_dir, DirEntry, File}, collections::HashMap, io::Write};
use proc_macro2::{TokenStream, Ident, Span};
use quote::{quote, format_ident};
use serde::Deserialize;

fn main () -> std::io::Result<()> {
    if !Path::new("browser-compat-data").exists() {
        let clone = Command::new("git")
            .args(["clone", "https://github.com/mdn/browser-compat-data.git"])
            .spawn()?
            .wait()?
            .success();

        if !clone {
            panic!("Error cloning HTML data")
        }
    }

    let standard = reqwest::blocking::get("https://html.spec.whatwg.org/").unwrap()
        .text().unwrap();

    let standard = scraper::Html::parse_document(&standard);

    let mut elements = read_dir(Path::new("browser-compat-data/html/elements"))?;
    let mut err = Vec::new();
    let mut tokens = Vec::new();

    while let Some(element) = elements.next().transpose()? {
        match parse_elements(&element, &standard) {
            Ok(x) => tokens.push(x),
            Err(_) => err.push(element)
        }
        
    }

    let tokens = tokens.into_iter().collect::<TokenStream>();
    let mut file = File::create("src/html.rs")?;
    file.write_fmt(format_args!("{tokens}"))?;

    //panic!("{}", err.len());

    Ok(())
}

fn parse_elements (element: &DirEntry, standard: &scraper::Html) -> std::io::Result<TokenStream> {
    if !element.file_type()?.is_file() { return Ok(TokenStream::new()) }

    let path = element.path();
    if path.extension() != Some("json").map(AsRef::as_ref) {
        return Ok(TokenStream::new());
    }

    let Html { html } = serde_json::from_reader::<_, Html>(File::open(path)?)?;
    let Elements { elements } = html;
    let mut result = Vec::with_capacity(elements.len());

    for (name, element) in elements {
        result.push(parse_element(name, element, standard)?);
    }

    return Ok(result.into_iter().collect::<TokenStream>())
}

fn parse_element (name: String, Element { compat, attrs }: Element, standard: &scraper::Html) -> std::io::Result<TokenStream> {
    let select = 
    
    let ident = format_ident!("{}", to_pascal_case(&name));
    let deprecated = match compat.status.deprecated {
        true => Some(quote! { #[deprecated] }),
        false => None
    };

    let mut attr_def = Vec::with_capacity(attrs.len());
    for (name, Element { compat, attrs }) in attrs {
        let ident = Ident::new_raw(&name.replace('-', "_"), Span::call_site());
        attr_def.push(quote! { pub #ident: () });
    }

    Ok(quote! {
        #deprecated
        pub struct #ident {
            #(#attr_def),*
        }
    })
}

#[derive(Debug, Deserialize)]
pub struct Html {
    pub html: Elements
}

#[derive(Debug, Deserialize)]
pub struct Elements {
    pub elements: HashMap<String, Element>
}

#[derive(Debug, Clone, Deserialize)]
pub struct Element {
    #[serde(rename = "__compat")]
    compat: Compatibility,
    #[serde(flatten)]
    attrs: HashMap<String, Element>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Compatibility {
    #[serde(default)]
    mdn_url: Option<String>,
    #[serde(default)]
    spec_url: Option<String>,
    status: Status
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct Status {
    experimental: bool,
    standard_track: bool,
    deprecated: bool
}

#[inline]
fn to_pascal_case (s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut upper = true;

    for c in s.chars() {
        if c == '_' || c == '-' {
            upper = true;
            continue
        }

        if upper {
            result.extend(c.to_uppercase());
            upper = false;
            continue
        }

        result.push(c);
    }

    return result
}