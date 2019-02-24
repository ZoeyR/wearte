use wearte::Template;

struct Holder {
    foo: usize,
    bar: usize,
}

#[derive(Template)] // this will generate the code...
#[template(path = "with.html")] // using the template in this path, relative
                                // to the templates dir in the crate root
struct WithTemplate {
    // the name of the struct can be anything
    hold: Holder, // the field name should match the variable name
                  // in your template
}

#[test]
fn test_with() {
    let hello = WithTemplate {
        hold: Holder { foo: 0, bar: 1 },
    }; // instantiate your struct
    assert_eq!("0 1", hello.call().unwrap()); // then call it.
}

struct DeepHold {
    deep: Holder,
}

#[derive(Template)] // this will generate the code...
#[template(
    source = "{{#each hold}}{{#with deep}}{{ foo }} {{ bar }}{{/with}}{{/each}}",
    ext = "txt"
)] // using the template in this path, relative
   // to the templates dir in the crate root
struct WithEachTemplate {
    // the name of the struct can be anything
    hold: Vec<DeepHold>, // the field name should match the variable name
                         // in your template
}

#[test]
fn test_with_each() {
    let hello = WithEachTemplate {
        hold: vec![DeepHold {
            deep: Holder { foo: 0, bar: 1 },
        }],
    }; // instantiate your struct
    assert_eq!("0 1", hello.call().unwrap()); // then call it.
}
