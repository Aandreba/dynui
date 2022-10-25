use macros::{component, html};
use crate::component::{Component, RefComponent, MutComponent};
use crate::{dynui, CONTEXT, Result};

#[component]
pub fn List<I: IntoIterator> (ordered: bool, iter: I) -> Result<web_sys::Element> where I::Item: Component {
    let tag = if ordered { "ol" } else { "ul" };
    let list = CONTEXT.with(|ctx| ctx.document.create_element(tag))?;
    
    for item in iter {
        let node = html! { <li>{item}</li> }?;
        list.append_child(&node)?;
    }

    return Ok(list)
}

#[component]
pub fn RefList<'a, I: IntoIterator<Item = &'a T>, T: 'a + RefComponent> (ordered: bool, iter: I) -> Result<web_sys::Element> {
    let tag = if ordered { "ol" } else { "ul" };
    let list = CONTEXT.with(|ctx| ctx.document.create_element(tag))?;
    
    for item in iter {
        let node = html! { <li>{item}</li> }?;
        list.append_child(&node)?;
    }

    return Ok(list)
}

#[component]
pub fn MutList<'a, I: IntoIterator<Item = &'a mut T>, T: 'a + MutComponent> (ordered: bool, iter: I) -> Result<web_sys::Element> {
    let tag = if ordered { "ol" } else { "ul" };
    let list = CONTEXT.with(|ctx| ctx.document.create_element(tag))?;
    
    for item in iter {
        let node = html! { <li>{item}</li> }?;
        list.append_child(&node)?;
    }

    return Ok(list)
}