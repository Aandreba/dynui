#![feature(type_name_of_val)]
use proc::html;

#[test]
fn a () {
    let alpha = html! {
        <a href={"hello world"} />
    };

    println!("{}", core::any::type_name_of_val(&alpha))
}