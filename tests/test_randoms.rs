#![cfg(feature = "rand")]

use hegel::gen::{integers, randoms, vecs, Generate};
use rand::seq::SliceRandom;
use rand::Rng;

#[test]
fn test_randoms_gen_range() {
    hegel::hegel(|| {
        let mut rng = randoms().generate();
        let x: i32 = rng.gen_range(1..=100);
        assert!((1..=100).contains(&x));
    });
}

#[test]
fn test_randoms_gen_bool() {
    hegel::hegel(|| {
        let mut rng = randoms().generate();
        let _b: bool = rng.gen();
    });
}

#[test]
fn test_randoms_shuffle_preserves_elements() {
    hegel::hegel(|| {
        let mut rng = randoms().generate();

        let original = vec![1, 2, 3, 4, 5];
        let mut shuffled = original.clone();
        shuffled.shuffle(&mut rng);

        shuffled.sort();
        assert_eq!(original, shuffled);
    });
}

#[test]
fn test_randoms_choose() {
    hegel::hegel(|| {
        let mut rng = randoms().generate();
        let items: Vec<i32> = vecs(integers()).with_min_size(1).generate();
        let picked = items.choose(&mut rng).unwrap();
        assert!(items.contains(picked));
    });
}

#[test]
fn test_randoms_fill_bytes() {
    hegel::hegel(|| {
        let mut rng = randoms().generate();
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);
    });
}

#[test]
fn test_randoms_true_random() {
    hegel::hegel(|| {
        let mut rng = randoms().use_true_random().generate();
        let x: i32 = rng.gen_range(1..=100);
        assert!((1..=100).contains(&x));
    });
}
