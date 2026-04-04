RELEASE_TYPE: patch

Pin `hegel-core` to 0.3.0 in conformance tests to prevent unrelated breakage from upstream changes. Move all unit tests from inline modules to `tests/embedded/`, keeping them accessible to private internals via `#[path]` includes.
