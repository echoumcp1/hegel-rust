RELEASE_TYPE: minor

This patch fixes `gs::vecs(gs::sampled_from(...)).unique(true)` sometimes producing duplicate elements.

Rename `Variables::empty()` to `Variables::is_empty()` to follow Rust naming conventions, and add `Variables::len() -> usize`. The old `empty()` method is removed; callers should use `is_empty()` instead.
