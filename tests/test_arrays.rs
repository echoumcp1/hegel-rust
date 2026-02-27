mod common;

use common::utils::find_any;
use hegel::generators::{self, Generate};

#[test]
fn test_array_of_integers() {
    hegel::hegel(|| {
        let arr: [i32; 5] = hegel::draw(&generators::arrays(generators::integers::<i32>()));
        assert_eq!(arr.len(), 5);
    });
}

#[test]
fn test_array_of_booleans() {
    hegel::hegel(|| {
        let arr: [bool; 3] = hegel::draw(&generators::arrays(generators::booleans()));
        assert_eq!(arr.len(), 3);
    });
}

#[test]
fn test_array_of_strings() {
    hegel::hegel(|| {
        let arr: [String; 2] = hegel::draw(&generators::arrays(generators::text()));
        assert_eq!(arr.len(), 2);
    });
}

#[test]
fn test_array_size_zero() {
    hegel::hegel(|| {
        let arr: [i32; 0] = hegel::draw(&generators::arrays(generators::integers::<i32>()));
        assert_eq!(arr.len(), 0);
    });
}

#[test]
fn test_array_size_one() {
    hegel::hegel(|| {
        let arr: [i32; 1] = hegel::draw(&generators::arrays(
            generators::integers().with_min(10).with_max(20),
        ));
        assert_eq!(arr.len(), 1);
        assert!((10..=20).contains(&arr[0]));
    });
}

#[test]
fn test_array_respects_element_bounds() {
    hegel::hegel(|| {
        let arr: [i32; 4] = hegel::draw(&generators::arrays(
            generators::integers().with_min(0).with_max(100),
        ));
        for &x in &arr {
            assert!((0..=100).contains(&x));
        }
    });
}

#[test]
fn test_array_with_mapped_elements() {
    hegel::hegel(|| {
        let arr: [i32; 3] = hegel::draw(&generators::arrays(
            generators::integers::<i32>()
                .with_min(i32::MIN / 2)
                .with_max(i32::MAX / 2)
                .map(|x| x * 2),
        ));
        for &x in &arr {
            assert!(x % 2 == 0);
        }
    });
}

#[test]
fn test_array_with_filtered_elements() {
    hegel::hegel(|| {
        let arr: [i32; 3] = hegel::draw(&generators::arrays(
            generators::integers::<i32>()
                .with_min(0)
                .with_max(100)
                .filter(|n| n % 2 == 0),
        ));
        for &x in &arr {
            assert!(x % 2 == 0);
        }
    });
}

#[test]
fn test_array_of_arrays() {
    hegel::hegel(|| {
        let arr: [[i32; 2]; 3] = hegel::draw(&generators::arrays(generators::arrays(
            generators::integers::<i32>().with_min(0).with_max(50),
        )));
        assert_eq!(arr.len(), 3);
        for inner in &arr {
            assert_eq!(inner.len(), 2);
            for &x in inner {
                assert!((0..=50).contains(&x));
            }
        }
    });
}

#[test]
fn test_array_generates_varying_values() {
    // An array of 5 integers from a wide range should not always be all the same
    find_any(
        generators::arrays::<_, i32, 5>(generators::integers()),
        |arr| arr.iter().collect::<std::collections::HashSet<_>>().len() > 1,
    );
}
