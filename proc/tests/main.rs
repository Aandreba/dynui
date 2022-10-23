use proc::html;

#[test]
fn a () {
    let alpha = html! {
        <a href="google.es" />
    };
}