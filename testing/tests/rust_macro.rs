use yarte::Template;

macro_rules! hello {
    () => {
        "world"
    };
}

#[derive(Template)]
#[template(path = "rust-macros.html")]
struct RustMacrosTemplate {}

#[test]
fn main() {
    let template = RustMacrosTemplate {};
    assert_eq!("Hello, world!", template.render().unwrap());
}
