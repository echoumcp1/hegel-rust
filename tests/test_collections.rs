use hegel::gen::{self, Generate};
use std::collections::HashSet;

#[test]
fn test_vec_with_max_size() {
    hegel::hegel(|| {
        let max_size: usize = gen::integers().with_min(0).with_max(20).generate();
        let vec: Vec<i32> = gen::vecs(gen::integers::<i32>())
            .with_max_size(max_size)
            .generate();
        assert!(vec.len() <= max_size);
    });
}

#[test]
fn test_vec_with_min_size() {
    hegel::hegel(|| {
        let min_size: usize = gen::integers().with_min(0).with_max(20).generate();
        let vec: Vec<i32> = gen::vecs(gen::integers::<i32>())
            .with_min_size(min_size)
            .generate();
        assert!(vec.len() >= min_size);
    });
}

#[test]
fn test_vec_with_min_and_max_size() {
    hegel::hegel(|| {
        let min_size: usize = gen::integers().with_min(0).with_max(10).generate();
        let max_size = min_size + 10;
        let vec: Vec<i32> = gen::vecs(gen::integers::<i32>())
            .with_min_size(min_size)
            .with_max_size(max_size)
            .generate();
        assert!(vec.len() >= min_size && vec.len() <= max_size);
    });
}

#[test]
fn test_vec_unique() {
    hegel::hegel(|| {
        let max_size: usize = gen::integers().with_min(0).with_max(20).generate();
        let vec: Vec<i32> = gen::vecs(gen::integers::<i32>())
            .with_max_size(max_size)
            .unique()
            .generate();

        let set: HashSet<_> = vec.iter().collect();
        assert_eq!(set.len(), vec.len());
    });
}

#[test]
fn test_vec_unique_with_min_size() {
    hegel::hegel(|| {
        let min_size: usize = gen::integers().with_min(1).with_max(10).generate();
        let vec: Vec<i32> = gen::vecs(gen::integers::<i32>())
            .with_min_size(min_size)
            .unique()
            .generate();

        assert!(vec.len() >= min_size);

        let set: HashSet<_> = vec.iter().collect();
        assert_eq!(set.len(), vec.len());
    });
}
