use derive_syn_parse::Parse;
use syn::{Token, Path, Pat, Expr, parse::ParseStream};
use syn::parse::Parse;

pub struct Elements (pub Vec<Element>);

impl Parse for Elements {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut v = Vec::new();
        while !input.is_empty() {
            v.push(Element::parse(input)?);
        }
        return Ok(Self(v))
    }
}

#[derive(Parse)]
pub struct OpenElement {
    pub left: Token![<],
    pub path: Path,
    #[call(parse_attrs)]
    pub attrs: Vec<ElementAttribute>,
    pub shift: Option<Token![/]>,
    pub right: Token![>],
}

#[derive(Parse)]
pub struct CloseElement {
    pub left: Token![<],
    pub shift: Token![/],
    pub path: Path,
    pub right: Token![>],
}

#[derive(Parse)]
pub enum ElementEnd {
    #[peek(Token![/], name = "Closed")]
    Closed {
        shift: Token![/],
        close: Token![>]
    },

    #[peek_with(true, name = "Open")]
    Open {
        #[call(parse_children)]
        children: Vec<Element>,
        close: CloseElement
    }
}

#[derive(Parse)]
pub struct Element {
    pub left: Token![<],
    pub path: Path,
    #[call(parse_attrs)]
    pub attrs: Vec<ElementAttribute>,


    #[call(parse_children)]
    pub children: Vec<Element>,
    pub close: CloseElement
}

#[derive(Parse)]
pub struct ElementAttribute {
    pub pat: Pat,
    pub eq_token: Token![=],
    pub expr: Expr
}

#[inline]
fn parse_attrs(input: ParseStream) -> syn::Result<Vec<ElementAttribute>> {
    let mut attrs = Vec::new();
    while !input.peek(Token![/]) && !input.peek(Token![>]) {
        attrs.push(ElementAttribute::parse(input)?);
    }
    return Ok(attrs)
}

#[inline]
fn parse_children(input: ParseStream) -> syn::Result<Vec<Element>> {
    let mut attrs = Vec::new();
    while !(input.peek(Token![<])  && input.peek2(Token![/])){
        attrs.push(Element::parse(input)?);
    }
    return Ok(attrs)
}