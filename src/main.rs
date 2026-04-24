use std::fmt::Debug;

use builder_concept::builder;

#[builder(use_default, debug = simple, builder_fn)]
#[derive(Clone, Default, Debug)]
struct Person {
    name: String,
    age: u8,
    hobbies: Vec<String>,
}

fn main() {
    let p = PersonBuilder::new()
        .name("Hello".to_string())
        .age(10)
        .hobbies(vec!["h1".to_string()])
        .build();

    let p = Person::builder().build_with_default();

    PersonBuilder::new()
        .name("Hello".into())
        .build_with_default();

    dbg!(PersonBuilder::new().name("".to_string()));

    let p1 = Person::builder()
        .name("Test".into())
        .age(10)
        .hobbies(Vec::new())
        .build();

    dbg!(p1);

    let p2 = Person::builder()
        .name("P2".into())
        /* .hobby("H1".to_string())
        .hobby("h2".to_string())
        .hobby(&mut vec!["H3".to_string(), "h4".to_string()]) */
        .build_with_default();

    dbg!(p2);

    let p3 = Person::builder()
        .name("p3".to_string())
        //.hobby("A".to_string())
        .hobbies(vec![])
        .age(0)
        .build();

    dbg!(p3);
}
