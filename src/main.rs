use std::vec;

use builder_concept::builder;

#[derive(Debug, Clone, Default)]
#[builder]
pub struct Person {
    name: String,
    age: u8,
    hobbies: Vec<String>,
}

fn main() {
    dbg!(Person::default());
    /*
    let p1 = Person::builder()
        .name("Test".into())
        .age(10)
        .hobbies(Vec::new())
        .build();

    dbg!(p1);

    let p2 = Person::builder()
        .name("P2".into())
        .hobby("H1".to_string())
        .hobby("h2".to_string())
        .hobby(&mut vec!["H3".to_string(), "h4".to_string()])
        .build_with_default();

    dbg!(p2);

    let p3 = Person::builder()
        .name("p3".to_string())
        .hobby("A".to_string())
        .age(0)
        .build();

    dbg!(p3);
    */
}
