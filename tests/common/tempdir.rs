// internal helper code
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

static PROCESS_TMPDIR: LazyLock<tempfile::TempDir> = LazyLock::new(|| {
    tempfile::TempDir::with_prefix("hegel-test-").unwrap()
});

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn process_tmpdir() -> &'static Path {
    PROCESS_TMPDIR.path()
}

pub fn mktemp_dir(prefix: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = format!("{prefix}.{}.{id}", std::process::id());
    let dir = process_tmpdir().join(name);
    std::fs::create_dir(&dir).unwrap();
    dir
}
