use yarte::Template;

#[derive(Template)] // this will generate the code...
#[template(path = "hello.html")] // using the template in this path, relative
                                 // to the templates dir in the crate root
struct HelloTemplate<'a> {
    // the name of the struct can be anything
    name: &'a str, // the field name should match the variable name
                   // in your template
}

#[test]
fn test_hello() {
    let hello = HelloTemplate { name: "world" }; // instantiate your struct
    assert_eq!("Hello, world!", hello.render().unwrap()); // then render it.
}

#[derive(Template)] // this will generate the code...
#[template(source = "{{}", ext = "txt")] // using the template in this path, relative
                                         // to the templates dir in the crate root
struct BracketsTemplate;

#[test]
fn test_brackets() {
    let hello = BracketsTemplate; // instantiate your struct
    assert_eq!("{{}", hello.render().unwrap()); // then render it.
}

#[derive(Template)] // this will generate the code...
#[template(source = "{{{}}", ext = "txt")] // using the template in this path, relative
                                           // to the templates dir in the crate root
struct Brackets2Template;

#[test]
fn test_brackets2() {
    let hello = Brackets2Template; // instantiate your struct
    assert_eq!("{{{}}", hello.render().unwrap()); // then render it.
}
