#![forbid(unsafe_code)]

//! Zero-Dependency Rust Headless App Validation Telemetry Helper
//! Materialized from `crvscience-code-orchestrator`
//! (`templates/rust-headless-validation-telemetry`).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Maximum serialized telemetry summary artifact size (384 KB Search/Deploy cap).
pub const MAX_VALIDATION_BYTES: usize = 384 * 1024;

/// RAII temp directory scoped to PID and ThreadId, deleted on drop.
#[allow(dead_code)]
pub struct TempDirGuard {
    pub path: PathBuf,
}

#[allow(dead_code)]
impl TempDirGuard {
    pub fn new(prefix: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "{prefix}_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("Failed creating temp validation directory");
        Self { path: dir }
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Zero-dependency RFC 8259 control-character and quote JSON escaper (unquoted inner text).
#[allow(dead_code)]
pub fn json_escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 8);
    for ch in raw.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c.is_control() => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Standard RFC 4180 CSV cell escaper.
#[allow(dead_code)]
pub fn csv_escape(raw: &str) -> String {
    if raw.contains(',') || raw.contains('"') || raw.contains('\n') || raw.contains('\r') {
        format!("\"{}\"", raw.replace('"', "\"\""))
    } else {
        raw.to_string()
    }
}

/// Ensures `<repo_root>/validation/` exists and returns its path.
#[allow(dead_code)]
pub fn validation_dir(repo_root: &Path) -> PathBuf {
    let dir = repo_root.join("validation");
    fs::create_dir_all(&dir).expect("Failed creating validation directory");
    dir
}

/// Enforces `MAX_VALIDATION_BYTES`, writes the local JSON summary under `Mutex<()>`,
/// and optionally mirrors telemetry when `mirror_path` is provided.
#[allow(dead_code)]
pub fn write_and_mirror_summary_json(
    summary_lock: &Mutex<()>,
    local_path: &Path,
    mirror_path: Option<&Path>,
    payload: &str,
) {
    assert!(
        payload.len() < MAX_VALIDATION_BYTES,
        "Validation summary {} exceeded {} KB cap: {} bytes",
        local_path.display(),
        MAX_VALIDATION_BYTES / 1024,
        payload.len()
    );
    let _guard = summary_lock.lock().expect("Summary mutex poisoned");
    if let Some(parent) = local_path.parent() {
        fs::create_dir_all(parent).expect("Failed creating local summary parent directory");
    }
    fs::write(local_path, payload).expect("Failed writing local validation summary JSON");
    if let Some(mirror) = mirror_path {
        if let Some(parent) = mirror.parent() {
            fs::create_dir_all(parent).expect("Failed creating mirror parent directory");
        }
        fs::write(mirror, payload).expect("Failed mirroring validation telemetry JSON");
    }
}
