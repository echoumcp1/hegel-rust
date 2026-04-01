---
name: coverage
description: "How to approach code coverage in this project. Use when coverage CI fails, when writing tests for new code, when deciding whether to add // nocov, or when you need to make untestable code testable. Also use proactively when writing new code to ensure it will be coverable."
---

# Code Coverage

This project requires 100% line coverage for new code. The coverage check (`scripts/check-coverage.py`) runs `cargo llvm-cov` and reports uncovered lines, with automatic exclusions for structural syntax, `#[cfg(test)]` modules, `todo!()`/`unreachable!()`/`assert!()` continuations, and `#[ignore]`d test bodies.

## The ratchet is not a budget

The nocov count in `.github/coverage-ratchet.json` tracks excluded lines and can only decrease. Just because previous work reduced the count does not mean you have implicit permission to add new uncovered lines. Think of the ratchet as immediately ratcheting down after any reduction — the slack is gone.

You may not add `// nocov` annotations without explicit human permission. If you think code is genuinely untestable, your first move should be to refactor it for testability, not to annotate it.

## Making code testable

The most common pattern: extract logic from functions that read environment/global state into parameterized functions that take those values as arguments.

```rust
// Hard to test — reads env vars directly
fn cache_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join("myapp");
    }
    // ...
}

// Testable — takes values as parameters
fn cache_dir_from(xdg: Option<String>, home: Option<PathBuf>) -> PathBuf {
    if let Some(xdg) = xdg {
        return PathBuf::from(xdg).join("myapp");
    }
    // ...
}

// Thin wrapper calls the testable version
fn cache_dir() -> PathBuf {
    cache_dir_from(std::env::var("XDG_CACHE_HOME").ok(), std::env::home_dir())
}
```

Other patterns:
- **Platform-specific match arms**: take arch/os as parameters so all branches are testable from any platform.
- **Command fallback chains**: take the command list as a parameter so tests can exercise the fallback without manipulating PATH.
- **Error paths in shell-outs**: restructure so the error message is on the same line as the call (for line-level coverage), or convert defensive error returns to panics when the function has only one caller that would panic anyway.

## Writing good tests

Tests should catch real bugs, not mirror the implementation.

- **Validate against external reality**: if your code maps to external identifiers (URLs, file names, API paths), hardcode the real external data in the test and validate against it. Don't just assert that each match arm produces a specific string — that's duplicating the code.
- **Test behavior, not structure**: a test that would still pass after introducing a bug is not testing anything.
- **Avoid network in unit tests**: use `file://` URLs with curl, create local tar.gz fixtures, use `tempfile::tempdir()` for isolation. Keep exactly one integration test that verifies the real network path works end-to-end.
- **Shell out to system tools** (sha256sum, tar) rather than reimplementing them. Makes the code simpler and the tests more realistic.

## Diagnosing coverage failures

When CI coverage fails:

1. Read the failure output — it lists each uncovered file:line and content.
2. Categorize each uncovered line:
   - **Thin wrapper**: a 1-2 line function that just delegates to a parameterized version. Is it called by any test (including integration tests via TempRustProject)? If subprocess coverage isn't picking it up, you may need a unit test that calls the wrapper directly.
   - **Platform-specific code**: refactor to take platform as a parameter.
   - **Error handling**: can you trigger the error in a test? (Bad file path, bad URL, invalid input.) If the error path is inside a panic/assert format string, the coverage checker excludes those continuation lines automatically.
   - **Dead code**: if it's truly unreachable, delete it.
3. Run `just check-coverage` locally if `cargo-llvm-cov` is available to iterate faster than CI.

## How the coverage script works

`scripts/check-coverage.py` runs `cargo llvm-cov --no-report --all-features` to collect coverage data, then generates an LCOV report. It tries to include TempRustProject subprocess binaries (found in the target directory as `temp_hegel_test_*`), though this depends on the binaries being compiled with coverage instrumentation.

The script then parses the LCOV data and checks each uncovered line against the automatic exclusion patterns. Lines that don't match any exclusion and don't have `// nocov` are reported as failures. It also automatically removes `// nocov` from lines that turn out to be covered, keeping the annotation count honest.
