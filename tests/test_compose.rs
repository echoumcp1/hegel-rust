mod common;

use common::utils::assert_all_examples;
use hegel::generators::{self, Generate};
use hegel::TestCase;

#[hegel::test]
fn test_compose_basic(tc: TestCase) {
    let value = tc.draw(&hegel::compose!(|draw| {
        draw(&generators::integers::<i32>().min_value(0).max_value(100))
    }));
    assert!((0..=100).contains(&value));
}

#[hegel::test]
fn test_compose_dependent_generation(tc: TestCase) {
    let (x, y) = tc.draw(&hegel::compose!(|draw| {
        let x = draw(&generators::integers::<i32>().min_value(0).max_value(50));
        let y = draw(&generators::integers::<i32>().min_value(x).max_value(100));
        (x, y)
    }));
    assert!(y >= x);
    assert!((0..=50).contains(&x));
    assert!((0..=100).contains(&y));
}

#[hegel::test]
fn test_compose_with_map(tc: TestCase) {
    let value = tc.draw(
        &hegel::compose!(|draw| {
            draw(&generators::integers::<i32>().min_value(0).max_value(10))
        })
        .map(|n| n * 2),
    );
    assert!(value % 2 == 0);
    assert!((0..=20).contains(&value));
}

#[hegel::test]
fn test_compose_with_filter(tc: TestCase) {
    let value = tc.draw(
        &hegel::compose!(|draw| {
            draw(&generators::integers::<i32>().min_value(0).max_value(100))
        })
        .filter(|n| n % 2 == 0),
    );
    assert!(value % 2 == 0);
}

#[hegel::test]
fn test_compose_with_boxed(tc: TestCase) {
    let gen =
        hegel::compose!(|draw| { draw(&generators::integers::<i32>().min_value(0).max_value(50)) })
            .boxed();
    let value = tc.draw(&gen);
    assert!((0..=50).contains(&value));
}

#[test]
fn test_compose_assert_all_examples() {
    assert_all_examples(
        hegel::compose!(|draw| {
            let x = draw(&generators::integers::<i32>().min_value(0).max_value(100));
            let y = draw(&generators::integers::<i32>().min_value(0).max_value(100));
            (x, y)
        }),
        |&(x, y)| (0..=100).contains(&x) && (0..=100).contains(&y),
    );
}

#[hegel::test]
fn test_compose_inside_one_of(tc: TestCase) {
    let value: i32 = tc.draw(&hegel::one_of!(
        hegel::compose!(|draw| { draw(&generators::integers::<i32>().min_value(0).max_value(10)) }),
        generators::integers::<i32>().min_value(100).max_value(110),
    ));
    assert!((0..=10).contains(&value) || (100..=110).contains(&value));
}

#[hegel::test]
fn test_compose_list_with_index(tc: TestCase) {
    let (list, index) = tc.draw(&hegel::compose!(|draw| {
        let list = draw(
            &generators::vecs(generators::integers::<i32>())
                .min_size(1)
                .max_size(20),
        );
        let index = draw(
            &generators::integers::<usize>()
                .min_value(0)
                .max_value(list.len() - 1),
        );
        (list, index)
    }));
    assert!(!list.is_empty());
    assert!(index < list.len());
}

#[hegel::test]
fn test_compose_nested(_tc: TestCase) {
    // we expect hegel::draw() inside compose! after nested compose to panic
    let result = std::panic::catch_unwind(|| {
        hegel::draw(&hegel::compose!(|draw| {
            draw(&hegel::compose!(|draw| {}));
            // expected to panic
            hegel::draw(&generators::integers::<i32>())
        }));
    });
    assert!(result.is_err());
}

#[hegel::test]
fn test_compose_string_building(tc: TestCase) {
    let s = tc.draw(&hegel::compose!(|draw| {
        let prefix = draw(&generators::sampled_from(vec!["hello", "world"]));
        let n = draw(&generators::integers::<i32>().min_value(0).max_value(99));
        format!("{}-{}", prefix, n)
    }));
    assert!(s.starts_with("hello-") || s.starts_with("world-"));
}
