#![feature(type_name_of_val)]
//! Test suite for the Web and headless browsers.

//#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use std::{time::Duration, ops::AddAssign};

use dynui::{*, macros::*, cell::{Cell, SharedCell, MutableCell, CellLike}};
use dynui::component::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() -> Result<()> {
    let mut cell = Cell::new(1u32);

    let html = html! {
        <div>
            <span>{&cell.display()}</span>
        </div>
    }?;

    context().body.append_child(&html.render()?)?;
    set_interval(Duration::from_secs(1), move || {
        cell.mutate(|x| x.add_assign(1));
    })?;

    Ok(())
}

#[wasm_bindgen_test]
fn html () -> Result<()> {
    use dynui::lib::button::Button;

    let value = SharedCell::new(0u128);
    let my_value = value.clone();
    let text = value.map(|x| format!("Clicked {x} times"));

    let alpha = html! {
        <div>
            <span>{&text}</span>
            <Button 
                text={"Click me!"}
                onclick={move || my_value.mutate(|x| x.add_assign(1))} 
            />
        </div>
    }?;

    let alpha = alpha.render()?;
    context().body.append_child(&alpha)?;
    Ok(())
}

#[wasm_bindgen_test]
fn list () -> Result<()> {
    use dynui::lib::{button::Button, list::List};
    let mut cell = Cell::new("hello");

    let children = vec![
        html! { <a href={"google.es"}>{&cell}</a> }?,
        html! { <a href={"facebook.com"}>{"Facebook"}</a> }?,
        html! { <a href={&cell}>{"YouTubve"}</a> }?
    ];

    let alpha = html! {
        <div>
            <List ordered={true} iter={children} />
            <Button text={"Click me!"} onclick={move || cell.set("world")} />
        </div>
    }?.render()?;

    context().body.append_child(&alpha)?;
    Ok(())
}

#[wasm_bindgen_test]
fn future () -> Result<()> {
    use dynui::lib::r#async::*;

    let future = async move {
        sleep(Duration::from_secs(1)).await?;
        return html! { <span>{"A second has passed!"}</span> }
    };

    let alpha = html! {
        <div>
            <Future fut={future} placeholder={html! { <span>{"Waiting"}</span> }} />
        </div>
    }?;

    context().body.append_child(&alpha)?;
    Ok(())
}