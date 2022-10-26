use macros::{component, html};
use crate::component::{Component, RefComponent, Element};
use crate::{dynui, Result, create_element};

#[component]
pub fn List<I: IntoIterator> (ordered: bool, iter: I) -> Result<Element> where I::Item: Component {
    let tag = if ordered { "ol" } else { "ul" };
    let list = create_element(tag)?;
    
    for item in iter {
        let node = html! { <li>{item}</li> }?;
        list.append_child(node)?;
    }

    return Ok(list)
}

#[component]
pub fn RefList<'a, I: IntoIterator<Item = &'a T>, T: 'a + RefComponent> (ordered: bool, iter: I) -> Result<Element> {
    let tag = if ordered { "ol" } else { "ul" };
    let list = create_element(tag)?;
    
    for item in iter {
        let node = html! { <li>{item}</li> }?;
        list.append_child(node)?;
    }

    return Ok(list)
}