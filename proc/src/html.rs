use derive_syn_parse::Parse;
use syn::{braced, Attribute};
use syn::{Token, Path, Pat, Expr, parse::ParseStream};
use syn::parse::Parse;

#[derive(Debug)]
pub enum Html {
    Element (Vec<Attribute>, Element),
    Expr (Vec<Attribute>, Expr),
}

impl Parse for Html {
    fn parse(input: ParseStream) -> syn::Result<Self> {
       if input.peek(syn::token::Brace) {
            let attrs = Attribute::parse_outer(input)?;
            let content; braced!(content in input);
            let tokens = Expr::parse(&content)?;
            return Ok(Self::Expr(attrs, tokens))
       }

       let attrs = Attribute::parse_outer(input)?;
       let element = Element::parse(input)?;
       return Ok(Self::Element(attrs, element))
    }
}

pub struct Elements (pub Vec<Html>);

impl Parse for Elements {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut v = Vec::new();
        while !input.is_empty() {
            v.push(Html::parse(input)?);
        }
        return Ok(Self(v))
    }
}

#[derive(Debug, Parse)]
pub struct ClosedElement {
    shift: Token![/],
    close: Token![>]
}

#[derive(Debug, Parse)]
pub struct OpenElement {
    close: Token![>],
    #[call(parse_children)]
    pub children: Vec<Html>,
    left: Token![<],
    shift: Token![/],
    pub path: Path,
    right: Token![>],
}

#[derive(Debug)]
pub enum ElementEnd {
    Closed (ClosedElement),
    Open (OpenElement)
}

impl Parse for ElementEnd {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![/]) {
            return ClosedElement::parse(input).map(ElementEnd::Closed)
        }
        return OpenElement::parse(input).map(ElementEnd::Open)
    }
}

#[derive(Debug, Parse)]
pub struct Element {
    pub left: Token![<],
    #[call(Path::parse_mod_style)]
    pub path: Path,
    #[call(parse_attrs)]
    pub attrs: Vec<ElementAttribute>,
    pub end: ElementEnd
}

#[derive(Debug, Parse)]
pub struct ElementAttribute {
    #[call(Attribute::parse_outer)]
    pub attrs: Vec<Attribute>,
    pub pat: Pat,
    pub eq_token: Token![=],
    #[brace]
    pub brace_token: syn::token::Brace,
    #[inside(brace_token)]
    pub expr: Expr
}

#[inline]
fn parse_attrs(input: ParseStream) -> syn::Result<Vec<ElementAttribute>> {
    let mut attrs = Vec::new();
    while !input.is_empty() && !input.peek(Token![/]) && !input.peek(Token![>]) {
        attrs.push(ElementAttribute::parse(input)?);
    }

    return Ok(attrs)
}

#[inline]
fn parse_children(input: ParseStream) -> syn::Result<Vec<Html>> {
    let mut attrs = Vec::new();
    //panic!("{} --- {}", input.peek(Token![<]), input.peek(Token![/]));

    while !input.is_empty() && !(input.peek(Token![<]) && input.peek2(Token![/])) {
        attrs.push(Html::parse(input)?);
    }
    return Ok(attrs)
}