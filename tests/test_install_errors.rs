mod common;

use common::project::TempRustProject;

#[test]
fn test_missing_uv_error_message() {
    // When uv is not on PATH, not cached, and curl is also missing,
    // the user should get a helpful error about installing uv manually.
    let code = r#"
fn main() {
    // Filter uv and curl out of PATH so download can't proceed
    let path = std::env::var("PATH").unwrap_or_default();
    let filtered: String = path
        .split(':')
        .filter(|dir| {
            !std::path::Path::new(&format!("{dir}/uv")).exists()
            && !std::path::Path::new(&format!("{dir}/curl")).exists()
        })
        .collect::<Vec<_>>()
        .join(":");
    std::env::set_var("PATH", &filtered);

    hegel::hegel(|tc| {
        let _ = tc.draw(hegel::generators::booleans());
    });
}
"#;

    TempRustProject::new()
        .main_file(code)
        .env_remove("HEGEL_SERVER_COMMAND")
        // Point XDG_CACHE_HOME to a temp dir so no cached uv is found
        .env("XDG_CACHE_HOME", "/tmp/hegel-test-no-cache")
        .expect_failure("Install uv manually")
        .cargo_run(&[]);
}

#[test]
fn test_downloads_uv_when_not_on_path() {
    // When uv is not on PATH and not cached, hegel should download uv
    // and use it to run hegel-core successfully.
    let code = r#"
fn main() {
    let path = std::env::var("PATH").unwrap_or_default();
    let filtered: String = path
        .split(':')
        .filter(|dir| !std::path::Path::new(&format!("{dir}/uv")).exists())
        .collect::<Vec<_>>()
        .join(":");
    std::env::set_var("PATH", &filtered);

    hegel::hegel(|tc| {
        let _ = tc.draw(hegel::generators::booleans());
    });

    // Verify uv was downloaded to the cache
    let cache_home = std::env::var("XDG_CACHE_HOME").unwrap();
    let cached_uv = format!("{cache_home}/hegel/uv");
    assert!(std::path::Path::new(&cached_uv).is_file(), "uv should be cached at {cached_uv}");
}
"#;

    TempRustProject::new()
        .main_file(code)
        .env_remove("HEGEL_SERVER_COMMAND")
        .env("XDG_CACHE_HOME", "/tmp/hegel-test-uv-download")
        .cargo_run(&[]);

    // Clean up the downloaded uv
    let _ = std::fs::remove_dir_all("/tmp/hegel-test-uv-download");
}
