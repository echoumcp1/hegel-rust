// internal helper code
#![allow(dead_code)]

use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use tempfile::TempDir;

pub struct TempRustProject {
    _temp_dir: TempDir,
    project_path: PathBuf,
    env_vars: Vec<(String, String)>,
    features: Vec<String>,
    has_main: bool,
    has_tests: bool,
}

pub struct RunOutput {
    pub status: ExitStatus,
    #[allow(dead_code)]
    pub stdout: String,
    pub stderr: String,
}

impl TempRustProject {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Copy the main project's Cargo.lock so the temp project uses the same
        // pinned dependency versions. Without this, cargo resolves fresh and may
        // pull in crates (e.g. getrandom 0.4+) that require a newer Rust edition
        // than our MSRV supports.
        let hegel_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let lock_src = hegel_path.join("Cargo.lock");
        if lock_src.exists() {
            std::fs::copy(&lock_src, project_path.join("Cargo.lock")).unwrap();
        }

        Self {
            _temp_dir: temp_dir,
            project_path,
            env_vars: Vec::new(),
            features: Vec::new(),
            has_main: false,
            has_tests: false,
        }
    }

    pub fn main_file(mut self, code: &str) -> Self {
        assert!(!self.has_main, "main_file can only be called once");
        let src_dir = self.project_path.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.rs"), code).unwrap();
        self.has_main = true;
        self
    }

    pub fn test_file(mut self, code: &str) -> Self {
        let tests_dir = self.project_path.join("tests");
        std::fs::create_dir_all(&tests_dir).unwrap();
        std::fs::write(tests_dir.join("test.rs"), code).unwrap();
        self.has_tests = true;
        self
    }

    pub fn feature(mut self, feature: &str) -> Self {
        self.features.push(feature.to_string());
        self
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.push((key.to_string(), value.to_string()));
        self
    }

    pub fn run(self) -> RunOutput {
        assert!(
            self.has_main || self.has_tests,
            "TempRustProject needs at least a main_file or test_file"
        );

        let hegel_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let features = if self.features.is_empty() {
            String::new()
        } else {
            format!(
                ", features = [{}]",
                self.features
                    .iter()
                    .map(|f| format!("\"{}\"", f))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let cargo_toml = format!(
            r#"[package]
name = "temp_hegel_test"
version = "0.1.0"
edition = "2021"

[dependencies]
hegeltest = {{ path = "{path}"{features} }}
"#,
            path = hegel_path.display(),
            features = features,
        );
        std::fs::write(self.project_path.join("Cargo.toml"), cargo_toml).unwrap();

        let mut cmd = Command::new(env!("CARGO"));

        if self.has_tests {
            cmd.args(["test", "--quiet"]);
        } else {
            cmd.args(["run", "--quiet"]);
        }

        cmd.current_dir(&self.project_path);

        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        let output = cmd.output().unwrap();

        RunOutput {
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        }
    }
}
