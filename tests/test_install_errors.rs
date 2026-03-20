mod common;

use common::project::TempRustProject;

#[test]
fn test_missing_uv_error_message() {
    // The test binary filters uv out of its own PATH at runtime,
    // so cargo can still compile it with the full PATH.
    let code = r#"
fn main() {
    // Filter uv out of PATH so ensure_hegel_installed can't find it
    let path = std::env::var("PATH").unwrap_or_default();
    let filtered: String = path
        .split(':')
        .filter(|dir| !std::path::Path::new(&format!("{dir}/uv")).exists())
        .collect::<Vec<_>>()
        .join(":");
    std::env::set_var("PATH", &filtered);

    // Also remove any cached install so it actually tries uv
    let _ = std::fs::remove_dir_all(".hegel");

    hegel::hegel(|tc| {
        let _ = tc.draw(hegel::generators::booleans());
    });
}
"#;

    let output = TempRustProject::new()
        .main_file(code)
        .env_remove("HEGEL_SERVER_COMMAND")
        .expect_failure("Could not find `uv` on your PATH")
        .cargo_run(&[]);

    assert!(
        output.stderr.contains("HEGEL_SERVER_COMMAND"),
        "Expected HEGEL_SERVER_COMMAND hint, got:\n{}",
        output.stderr
    );
    assert!(
        output.stderr.contains("docs/installation.md"),
        "Expected docs link, got:\n{}",
        output.stderr
    );
}
