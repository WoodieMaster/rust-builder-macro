use std::fmt::Debug;
use wood_builder::builder;

#[builder(use_default, debug = simple, builder_fn)]
#[derive(Clone, Default, Debug)]
struct Person<H>
where
    H: IntoIterator<Item = String> + Default,
{
    name: String,
    age: u8,
    hobbies: H,
}

#[builder(builder_fn)]
#[derive(Debug)]
struct Vec2(f32, f32);

#[builder(builder_fn)]
struct RefMe<'a, T>
where
    T: ?Sized,
{
    _r: &'a T,
}

fn main() {
    let r: RefMe<'static, str> = RefMe::builder()._r("").build();
    let v = Vec2(0.0, 1.0);
    println!("{}, {}", v.0, v.1);
    let p: Person<Vec<String>> = PersonBuilder::new()
        .name("Hello".to_string())
        .age(10)
        .hobbies(vec!["A1".to_string()])
        .build();

    let p: Person<Vec<String>> = Person::builder().build_with_default();

    println!("{:?}", p.hobbies);

    PersonBuilder::<Vec<String>>::new()
        .name("Hello".into())
        .build_with_default();

    dbg!(PersonBuilder::<Vec<String>>::new().name("".to_string()));

    let p1 = Person::builder()
        .name("Test".into())
        .age(10)
        .hobbies(Vec::new())
        .build();

    dbg!(p1);

    let p2 = Person::<Vec<String>>::builder()
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
