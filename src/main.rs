use std::fmt::Debug;

use builder_concept::builder;

#[builder]
#[derive(Clone, Default)]
struct Person {
    name: String,
    age: u8,
    hobbies: Vec<String>,
}

impl Debug for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Person")
            .field("name", &self.name)
            .field("age", &self.age)
            .field("hobbies", &self.hobbies)
            .finish()
    }
}

fn main() {
    let p = PersonBuilder::new()
        .name("Hello".to_string())
        .age(10)
        .hobbies(vec!["h1".to_string()])
        .build();
    dbg!(p);

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
