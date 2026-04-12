RELEASE_TYPE: minor

Fix `vecs(...).unique(true)` not actually enforcing element uniqueness in some cases.

Calling `.unique()` now requires the elements produced by the generator passed to `vecs()` to implement `PartialEq`. This is therefore technically a breaking change, though we expect that the only case where you will need to update your code is when it was previously not working anyway.