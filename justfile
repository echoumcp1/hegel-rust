check:
    cargo fmt --check
    cargo clippy --all-features --tests -- -D warnings
    RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
    cargo test
    cargo test --all-features

docs:
    cargo clean --doc && cargo doc --open --all-features --no-deps

test:
    cargo test

format:
    cargo fmt

lint:
    cargo fmt --check
    cargo clippy --all-features --tests -- -D warnings

coverage:
    # requires:
    # * cargo install cargo-llvm-cov
    # * rustup component add llvm-tools-preview
    cargo llvm-cov --all-features --fail-under-lines 30 --show-missing-lines
