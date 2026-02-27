mod common;

use common::project::TempRustProject;
use hegel::generators;

#[hegel::test]
#[test]
fn test_basic_usage() {
    let _: bool = hegel::draw(&generators::booleans());
}

#[hegel::test(test_cases = 10)]
#[test]
fn test_with_settings() {
    let _: bool = hegel::draw(&generators::booleans());
}

#[test]
#[should_panic(expected = "draw() cannot be called outside of a Hegel test")]
fn test_draw_outside_test_panics() {
    hegel::draw(&generators::booleans());
}

#[test]
fn test_params_compile_error() {
    let code = r#"
use hegel::generators;

#[hegel::test]
fn main(x: i32) {
    let _ = x;
}
"#;
    let output = TempRustProject::new(code).run();
    assert!(!output.status.success());
    assert!(
        output.stderr.contains("must not have parameters"),
        "Expected parameter error, got: {}",
        output.stderr
    );
}
