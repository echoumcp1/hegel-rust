mod common;

use common::project::TempRustProject;

#[test]
fn test_filter_too_much_fails() {
    let code = r#"
use hegel::generators;

fn main() {
    hegel::Hegel::new(|tc: hegel::TestCase| {
        let _: i32 = tc.draw(generators::integers().min_value(0).max_value(100));
        tc.assume(false);
    })
    .test_cases(100)
    .run();
}
"#;
    let output = TempRustProject::new(code).run();
    assert!(
        !output.status.success(),
        "Expected failure from filter_too_much health check"
    );
    assert!(
        output.stderr.contains("Health check failure"),
        "Expected health check failure message in stderr, got: {}",
        output.stderr
    );
}

#[test]
fn test_filter_too_much_suppressed() {
    let code = r#"
use hegel::generators;

fn main() {
    hegel::Hegel::new(|tc: hegel::TestCase| {
        let _: i32 = tc.draw(generators::integers().min_value(0).max_value(100));
        tc.assume(false);
    })
    .test_cases(100)
    .suppress_health_check(hegel::HealthCheck::FilterTooMuch)
    .run();
}
"#;
    let output = TempRustProject::new(code).run();
    assert!(
        output.status.success(),
        "Expected success with suppressed health check, got stderr: {}",
        output.stderr
    );
}

#[test]
fn test_multiple_health_checks_suppressed() {
    let code = r#"
use hegel::generators;

fn main() {
    hegel::Hegel::new(|tc: hegel::TestCase| {
        let _: i32 = tc.draw(generators::integers().min_value(0).max_value(100));
        tc.assume(false);
    })
    .test_cases(100)
    .suppress_health_check(hegel::HealthCheck::FilterTooMuch)
    .suppress_health_check(hegel::HealthCheck::TooSlow)
    .run();
}
"#;
    let output = TempRustProject::new(code).run();
    assert!(
        output.status.success(),
        "Expected success with multiple suppressed health checks, got stderr: {}",
        output.stderr
    );
}
