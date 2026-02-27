mod common;

use common::utils::{assert_all_examples, find_any};
use hegel::generators::{self, Generate};

// tuples2

#[test]
fn test_tuple2_basic() {
    hegel::hegel(|| {
        let (a, b): (i32, bool) = hegel::draw(&generators::tuples2(
            generators::integers(),
            generators::booleans(),
        ));
        let _ = (a, b);
    });
}

#[test]
fn test_tuple2_respects_bounds() {
    hegel::hegel(|| {
        let (a, b): (i32, i32) = hegel::draw(&generators::tuples2(
            generators::integers().with_min(0).with_max(10),
            generators::integers().with_min(100).with_max(200),
        ));
        assert!((0..=10).contains(&a));
        assert!((100..=200).contains(&b));
    });
}

// tuples3

#[test]
fn test_tuple3_basic() {
    hegel::hegel(|| {
        let (a, b, c): (i32, String, bool) = hegel::draw(&generators::tuples3(
            generators::integers(),
            generators::text(),
            generators::booleans(),
        ));
        let _ = (a, b, c);
    });
}

#[test]
fn test_tuple3_respects_bounds() {
    hegel::hegel(|| {
        let (a, b, c): (i32, i32, i32) = hegel::draw(&generators::tuples3(
            generators::integers().with_min(0).with_max(10),
            generators::integers().with_min(20).with_max(30),
            generators::integers().with_min(40).with_max(50),
        ));
        assert!((0..=10).contains(&a));
        assert!((20..=30).contains(&b));
        assert!((40..=50).contains(&c));
    });
}

// tuples4

#[test]
fn test_tuple4_basic() {
    hegel::hegel(|| {
        let (a, b, c, d): (i32, i32, i32, i32) = hegel::draw(&generators::tuples4(
            generators::integers().with_min(0).with_max(10),
            generators::integers().with_min(0).with_max(10),
            generators::integers().with_min(0).with_max(10),
            generators::integers().with_min(0).with_max(10),
        ));
        assert!((0..=10).contains(&a));
        assert!((0..=10).contains(&b));
        assert!((0..=10).contains(&c));
        assert!((0..=10).contains(&d));
    });
}

// tuples5

#[test]
fn test_tuple5_basic() {
    hegel::hegel(|| {
        let t: (i32, i32, i32, i32, i32) = hegel::draw(&generators::tuples5(
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
        ));
        let _ = t;
    });
}

// larger arities compile and run

#[test]
fn test_tuple6_through_12() {
    hegel::hegel(|| {
        let _: (i32, i32, i32, i32, i32, i32) = hegel::draw(&generators::tuples6(
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
        ));
    });
    hegel::hegel(|| {
        let _: (i32, i32, i32, i32, i32, i32, i32) = hegel::draw(&generators::tuples7(
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
        ));
    });
    hegel::hegel(|| {
        let _: (i32, i32, i32, i32, i32, i32, i32, i32) = hegel::draw(&generators::tuples8(
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
        ));
    });
    hegel::hegel(|| {
        let _: (i32, i32, i32, i32, i32, i32, i32, i32, i32) = hegel::draw(&generators::tuples9(
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
            generators::integers(),
        ));
    });
    hegel::hegel(|| {
        let _: (i32, i32, i32, i32, i32, i32, i32, i32, i32, i32) =
            hegel::draw(&generators::tuples10(
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
            ));
    });
    hegel::hegel(|| {
        let _: (i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32) =
            hegel::draw(&generators::tuples11(
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
            ));
    });
    hegel::hegel(|| {
        let _: (i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32) =
            hegel::draw(&generators::tuples12(
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
                generators::integers(),
            ));
    });
}

// mapped tuples

#[test]
fn test_tuple2_with_mapped_elements() {
    hegel::hegel(|| {
        let (a, b): (i32, i32) = hegel::draw(&generators::tuples2(
            generators::integers::<i32>()
                .with_min(i32::MIN / 2)
                .with_max(i32::MAX / 2)
                .map(|x| x * 2),
            generators::integers::<i32>()
                .with_min(0)
                .with_max(100)
                .map(|x| x + 1),
        ));
        assert!(a % 2 == 0);
        assert!((1..=101).contains(&b));
    });
}

// mixed types

#[test]
fn test_tuple_mixed_types() {
    hegel::hegel(|| {
        let (n, s, b, f): (i32, String, bool, f64) = hegel::draw(&generators::tuples4(
            generators::integers().with_min(0).with_max(100),
            generators::text().with_max_size(10),
            generators::booleans(),
            generators::floats(),
        ));
        assert!((0..=100).contains(&n));
        assert!(s.len() <= 40); // max_size is in chars, UTF-8 can expand
        let _ = (b, f);
    });
}

// tuples in collections

#[test]
fn test_vec_of_tuples() {
    hegel::hegel(|| {
        let vec: Vec<(i32, bool)> = hegel::draw(
            &generators::vecs(generators::tuples2(
                generators::integers::<i32>().with_min(0).with_max(100),
                generators::booleans(),
            ))
            .with_max_size(10),
        );
        for &(n, _b) in &vec {
            assert!((0..=100).contains(&n));
        }
    });
}

// tuple can find specific values

#[test]
fn test_tuple2_can_find_both_true_and_false() {
    find_any(
        generators::tuples2(generators::booleans(), generators::booleans()),
        |(a, b)| *a && !*b,
    );
    find_any(
        generators::tuples2(generators::booleans(), generators::booleans()),
        |(a, b)| !*a && *b,
    );
}

// assert_all_examples with tuples

#[test]
fn test_tuple2_all_examples_in_bounds() {
    assert_all_examples(
        generators::tuples2(
            generators::integers::<i32>().with_min(0).with_max(10),
            generators::integers::<i32>().with_min(0).with_max(10),
        ),
        |(a, b)| (0..=10).contains(a) && (0..=10).contains(b),
    );
}

#[test]
fn test_tuple3_all_examples_in_bounds() {
    assert_all_examples(
        generators::tuples3(
            generators::integers::<i32>().with_min(-5).with_max(5),
            generators::integers::<i32>().with_min(10).with_max(20),
            generators::integers::<i32>().with_min(100).with_max(200),
        ),
        |(a, b, c)| (-5..=5).contains(a) && (10..=20).contains(b) && (100..=200).contains(c),
    );
}
