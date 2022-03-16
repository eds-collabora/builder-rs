#![allow(dead_code)]
#![allow(unused_imports)]

use boulder::{Buildable, Builder, Generatable, Generator};

fn foo(a: i16) -> i16 {
    a + 6
}

#[derive(Debug, Buildable)]
pub struct Womble {
    #[boulder(default = "hullo")]
    a: String,
    #[boulder(default=foo(1))]
    b: i32,
}

#[derive(Debug, Buildable)]
pub struct Badger {
    #[boulder(buildable(a="hallo", b=foo(5)))]
    w: Womble,
    #[boulder(buildable)]
    v: Womble,
}

#[test]
fn test_simple() {
    let w = Womble::builder().a("hello").b(4i16).build();
    println!("W: {:?}", w);

    let w = Womble::builder().build();
    println!("W: {:?}", w);

    let b = Badger::builder().build();
    println!("B: {:?}", b);
}

#[derive(Debug, Buildable)]
pub struct Bodger<U: Buildable>
where
    U: core::fmt::Debug,
{
    #[boulder(buildable(a="hallo", b=foo(5)))]
    w: Womble,
    #[boulder(buildable)]
    v: U,
}

#[test]
fn test_generic() {
    let w = Bodger::<Womble>::builder()
        .w(Womble::builder().build())
        .v(Womble::builder().build())
        .build();
    println!("B: {:?}", w);
}

#[derive(Debug, Generatable)]
pub struct Wizard {
    #[boulder(default = "hello")]
    a: String,
    #[boulder(generator=boulder::gen::Inc(5))]
    b: i32,
}

#[test]
fn test_generator() {
    let mut g = Wizard::generator();

    let w = g.generate();
    let w2 = g.generate();

    assert_eq!(w.a, "hello".to_string());
    assert_eq!(w.b, 5);
    assert_eq!(w2.a, "hello".to_string());
    assert_eq!(w2.b, 6);
}

#[test]
fn test_string_pattern() {
    let mut g = boulder::gen::Pattern!("example-{}", boulder::gen::Inc(2));
    for i in 0..5 {
        assert_eq!(g.generate(), format!("example-{}", i + 2));
    }
}

#[derive(Debug, Generatable)]
pub struct Sorceress {
    #[boulder(generator=boulder::gen::Pattern!("an-example-{}", boulder::gen::Inc(1)))]
    a: String,
    #[boulder(generator=boulder::gen::Inc(5))]
    b: i32,
}

#[test]
fn test_generator2() {
    let mut g = Sorceress::generator();

    let w = g.generate();
    let w2 = g.generate();

    assert_eq!(w.a, "an-example-1".to_string());
    assert_eq!(w.b, 5);
    assert_eq!(w2.a, "an-example-2".to_string());
    assert_eq!(w2.b, 6);
}

#[derive(Debug, Generatable)]
pub struct Sorceress2 {
    #[boulder(generator=boulder::gen::Pattern!("{}-an-example-{}", boulder::gen::Inc(1), boulder::gen::Inc(5)))]
    a: String,
    #[boulder(generator=boulder::gen::Inc(5))]
    b: i32,
}

#[test]
fn test_generator3() {
    let mut g = Sorceress2::generator();

    let w = g.generate();
    let w2 = g.generate();

    assert_eq!(w.a, "1-an-example-5".to_string());
    assert_eq!(w.b, 5);
    assert_eq!(w2.a, "2-an-example-6".to_string());
    assert_eq!(w2.b, 6);
}

#[derive(Debug, Generatable)]
pub struct Elephant<T>
where
    T: Generatable + 'static,
    // <T as Generatable>::Generator: 'static
{
    #[boulder(generatable)]
    foo: T,
    #[boulder(default = 5)]
    ival: i64,
}

#[test]
fn test_iterator() {
    let g = Elephant::<Sorceress2>::generator();

    for (count, elt) in g.into_iter().take(5).enumerate() {
        assert_eq!(elt.foo.a, format!("{}-an-example-{}", count + 1, count + 5));
        assert_eq!(elt.foo.b, (count + 5) as i32);
        assert_eq!(elt.ival, 5i64);
    }
}

#[derive(Debug, Generatable)]
pub struct Giraffe {
    a: i32,
    b: String,
}

#[test]
fn test_closure() {
    let mut z = 5;
    let mut s = String::new();
    let mut gen = Giraffe::generator();
    gen.a(move || {
        z += 1;
        z
    })
    .b(move || {
        s.push('+');
        s.clone()
    });
    let g1 = gen.generate();
    let g2 = gen.generate();
    assert_eq!(g1.a, 6);
    assert_eq!(g1.b, "+".to_string());
    assert_eq!(g2.a, 7);
    assert_eq!(g2.b, "++".to_string());
}

#[derive(Debug, Buildable)]
pub struct Zebra1 {
    a: i32,
    #[boulder(sequence = 2)]
    b: Vec<String>,
}

#[derive(Debug, Buildable)]
pub struct Zebra2 {
    a: i32,
    #[boulder(sequence = 3, default = "hello")]
    b: Vec<String>,
}

#[derive(Debug, Buildable)]
pub struct Zebra3 {
    a: i32,
    #[boulder(sequence=4, generator=boulder::gen::Pattern!("a-{}", boulder::gen::Inc(0)))]
    b: Vec<String>,
}

#[derive(Clone, Debug, Buildable, Generatable)]
struct Nested {
    a: i32,
    b: String,
}

#[derive(Debug, Buildable)]
struct Zebra4 {
    a: i32,
    #[boulder(sequence=5, generatable(a=boulder::gen::Inc(5),
                                      b=boulder::gen::Pattern!("x{}", boulder::gen::Inc(2))))]
    b: Vec<Nested>,
}

#[derive(Debug, Buildable)]
struct Zebra5 {
    a: i32,
    #[boulder(sequence = 6, buildable(a = 10, b = "hello"))]
    b: Vec<Nested>,
}

#[test]
fn test_build_vector() {
    let z1 = Zebra1::builder().build();
    assert_eq!(z1.a, 0);
    assert_eq!(z1.b.len(), 2);
    assert_eq!(z1.b[0], String::new());
    assert_eq!(z1.b[1], String::new());

    let z2 = Zebra2::builder().build();
    assert_eq!(z2.a, 0);
    assert_eq!(z2.b.len(), 3);
    assert_eq!(z2.b[0], "hello".to_string());
    assert_eq!(z2.b[1], "hello".to_string());
    assert_eq!(z2.b[2], "hello".to_string());

    let z3 = Zebra3::builder().build();
    assert_eq!(z3.a, 0);
    assert_eq!(z3.b.len(), 4);
    assert_eq!(z3.b[0], "a-0".to_string());
    assert_eq!(z3.b[1], "a-1".to_string());
    assert_eq!(z3.b[2], "a-2".to_string());
    assert_eq!(z3.b[3], "a-3".to_string());

    let z4 = Zebra4::builder().build();
    assert_eq!(z4.a, 0);
    assert_eq!(z4.b.len(), 5);
    assert_eq!(z4.b[0].a, 5);
    assert_eq!(z4.b[0].b, "x2".to_string());
    assert_eq!(z4.b[1].a, 6);
    assert_eq!(z4.b[1].b, "x3".to_string());
    assert_eq!(z4.b[2].a, 7);
    assert_eq!(z4.b[2].b, "x4".to_string());
    assert_eq!(z4.b[3].a, 8);
    assert_eq!(z4.b[3].b, "x5".to_string());
    assert_eq!(z4.b[4].a, 9);
    assert_eq!(z4.b[4].b, "x6".to_string());

    let z5 = Zebra5::builder().build();
    assert_eq!(z5.a, 0);
    assert_eq!(z5.b.len(), 6);
    assert_eq!(z5.b[0].a, 10);
    assert_eq!(z5.b[0].b, "hello".to_string());
    assert_eq!(z5.b[1].a, 10);
    assert_eq!(z5.b[1].b, "hello".to_string());
    assert_eq!(z5.b[2].a, 10);
    assert_eq!(z5.b[2].b, "hello".to_string());
    assert_eq!(z5.b[3].a, 10);
    assert_eq!(z5.b[3].b, "hello".to_string());
    assert_eq!(z5.b[4].a, 10);
    assert_eq!(z5.b[4].b, "hello".to_string());
    assert_eq!(z5.b[5].a, 10);
    assert_eq!(z5.b[5].b, "hello".to_string());
}

#[derive(Debug, Generatable)]
pub struct Kangaroo1 {
    a: i32,
    #[boulder(sequence_generator = boulder::gen::Inc(2usize))]
    b: Vec<String>,
}

#[derive(Debug, Generatable)]
pub struct Kangaroo2 {
    a: i32,
    #[boulder(sequence_generator = boulder::gen::Inc(3usize), default = "hello")]
    b: Vec<String>,
}

#[derive(Debug, Generatable)]
pub struct Kangaroo3 {
    a: i32,
    #[boulder(sequence_generator= boulder::gen::Inc(4usize),
              generator=boulder::gen::Pattern!("a-{}", boulder::gen::Inc(0)))]
    b: Vec<String>,
}

#[derive(Debug, Generatable)]
struct Kangaroo4 {
    a: i32,
    #[boulder(sequence_generator= boulder::gen::Inc(5usize),
              generatable(a=boulder::gen::Inc(5),
                          b=boulder::gen::Pattern!("x{}", boulder::gen::Inc(2))))]
    b: Vec<Nested>,
}

#[derive(Debug, Generatable)]
struct Kangaroo5 {
    a: i32,
    #[boulder(sequence_generator = boulder::gen::Inc(6usize),
              buildable(a = 10, b = "hello"))]
    b: Vec<Nested>,
}

#[derive(Debug, Generatable)]
struct Kangaroo6 {
    a: i32,
    #[boulder(sequence = 3,
              generatable(a=boulder::gen::Inc(5),
                          b=boulder::gen::Pattern!("x{}", boulder::gen::Inc(2))))]
    b: Vec<Nested>,
}

#[test]
fn test_generate_vector() {
    let mut g = Kangaroo1::generator();
    let k11 = g.generate();
    let k12 = g.generate();
    assert_eq!(k11.a, 0);
    assert_eq!(k11.b.len(), 2);
    assert_eq!(k11.b[0], String::new());
    assert_eq!(k11.b[1], String::new());
    assert_eq!(k12.a, 0);
    assert_eq!(k12.b.len(), 3);
    assert_eq!(k12.b[0], String::new());
    assert_eq!(k12.b[1], String::new());
    assert_eq!(k12.b[2], String::new());

    let mut g = Kangaroo2::generator();
    let k21 = g.generate();
    let k22 = g.generate();
    assert_eq!(k21.a, 0);
    assert_eq!(k21.b.len(), 3);
    assert_eq!(k21.b[0], "hello".to_string());
    assert_eq!(k21.b[1], "hello".to_string());
    assert_eq!(k21.b[2], "hello".to_string());
    assert_eq!(k22.a, 0);
    assert_eq!(k22.b.len(), 4);
    assert_eq!(k22.b[0], "hello".to_string());
    assert_eq!(k22.b[1], "hello".to_string());
    assert_eq!(k22.b[2], "hello".to_string());
    assert_eq!(k22.b[3], "hello".to_string());

    let mut g = Kangaroo3::generator();
    let k31 = g.generate();
    let k32 = g.generate();
    assert_eq!(k31.a, 0);
    assert_eq!(k31.b.len(), 4);
    assert_eq!(k31.b[0], "a-0".to_string());
    assert_eq!(k31.b[1], "a-1".to_string());
    assert_eq!(k31.b[2], "a-2".to_string());
    assert_eq!(k31.b[3], "a-3".to_string());
    assert_eq!(k32.a, 0);
    assert_eq!(k32.b.len(), 5);
    assert_eq!(k32.b[0], "a-0".to_string());
    assert_eq!(k32.b[1], "a-1".to_string());
    assert_eq!(k32.b[2], "a-2".to_string());
    assert_eq!(k32.b[3], "a-3".to_string());
    assert_eq!(k32.b[4], "a-4".to_string());

    let mut g = Kangaroo4::generator();
    let k41 = g.generate();
    let k42 = g.generate();
    assert_eq!(k41.a, 0);
    assert_eq!(k41.b.len(), 5);
    assert_eq!(k41.b[0].a, 5);
    assert_eq!(k41.b[0].b, "x2".to_string());
    assert_eq!(k41.b[1].a, 6);
    assert_eq!(k41.b[1].b, "x3".to_string());
    assert_eq!(k41.b[2].a, 7);
    assert_eq!(k41.b[2].b, "x4".to_string());
    assert_eq!(k41.b[3].a, 8);
    assert_eq!(k41.b[3].b, "x5".to_string());
    assert_eq!(k41.b[4].a, 9);
    assert_eq!(k41.b[4].b, "x6".to_string());
    assert_eq!(k42.a, 0);
    assert_eq!(k42.b.len(), 6);
    assert_eq!(k42.b[0].a, 5);
    assert_eq!(k42.b[0].b, "x2".to_string());
    assert_eq!(k42.b[1].a, 6);
    assert_eq!(k42.b[1].b, "x3".to_string());
    assert_eq!(k42.b[2].a, 7);
    assert_eq!(k42.b[2].b, "x4".to_string());
    assert_eq!(k42.b[3].a, 8);
    assert_eq!(k42.b[3].b, "x5".to_string());
    assert_eq!(k42.b[4].a, 9);
    assert_eq!(k42.b[4].b, "x6".to_string());
    assert_eq!(k42.b[5].a, 10);
    assert_eq!(k42.b[5].b, "x7".to_string());

    let mut g = Kangaroo5::generator();
    let k51 = g.generate();
    let k52 = g.generate();
    assert_eq!(k51.a, 0);
    assert_eq!(k51.b.len(), 6);
    assert_eq!(k51.b[0].a, 10);
    assert_eq!(k51.b[0].b, "hello".to_string());
    assert_eq!(k51.b[1].a, 10);
    assert_eq!(k51.b[1].b, "hello".to_string());
    assert_eq!(k51.b[2].a, 10);
    assert_eq!(k51.b[2].b, "hello".to_string());
    assert_eq!(k51.b[3].a, 10);
    assert_eq!(k51.b[3].b, "hello".to_string());
    assert_eq!(k51.b[4].a, 10);
    assert_eq!(k51.b[4].b, "hello".to_string());
    assert_eq!(k51.b[5].a, 10);
    assert_eq!(k51.b[5].b, "hello".to_string());
    assert_eq!(k52.a, 0);
    assert_eq!(k52.b.len(), 7);
    assert_eq!(k52.b[0].a, 10);
    assert_eq!(k52.b[0].b, "hello".to_string());
    assert_eq!(k52.b[1].a, 10);
    assert_eq!(k52.b[1].b, "hello".to_string());
    assert_eq!(k52.b[2].a, 10);
    assert_eq!(k52.b[2].b, "hello".to_string());
    assert_eq!(k52.b[3].a, 10);
    assert_eq!(k52.b[3].b, "hello".to_string());
    assert_eq!(k52.b[4].a, 10);
    assert_eq!(k52.b[4].b, "hello".to_string());
    assert_eq!(k52.b[5].a, 10);
    assert_eq!(k52.b[5].b, "hello".to_string());
    assert_eq!(k52.b[6].a, 10);
    assert_eq!(k52.b[6].b, "hello".to_string());

    let mut g = Kangaroo6::generator();
    let k61 = g.generate();
    let k62 = g.generate();
    assert_eq!(k61.a, 0);
    assert_eq!(k61.b.len(), 3);
    assert_eq!(k61.b[0].a, 5);
    assert_eq!(k61.b[0].b, "x2".to_string());
    assert_eq!(k61.b[1].a, 6);
    assert_eq!(k61.b[1].b, "x3".to_string());
    assert_eq!(k61.b[2].a, 7);
    assert_eq!(k61.b[2].b, "x4".to_string());
    assert_eq!(k62.a, 0);
    assert_eq!(k62.b.len(), 3);
    assert_eq!(k62.b[0].a, 5);
    assert_eq!(k62.b[0].b, "x2".to_string());
    assert_eq!(k62.b[1].a, 6);
    assert_eq!(k62.b[1].b, "x3".to_string());
    assert_eq!(k62.b[2].a, 7);
    assert_eq!(k62.b[2].b, "x4".to_string());
}
