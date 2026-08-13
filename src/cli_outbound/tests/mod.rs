//! Tests for cli_outbound, split by concern to leave growth room under
//! the 300-line source cap:
//! - [`run`]: constructor, binary resolution, argv, happy-path streaming,
//!   `run_in` cwd propagation.
//! - [`stream`]: chunk iteration, `exit_info` classification, `pump_step`.
//! - [`spawn`]: spawn-error and drop→SIGTERM→SIGKILL cleanup.
//! - [`detach`]: `spawn_detached` — parent-survival, cwd propagation, the
//!   per-spawn stderr sink (and its null degradation), spawn error, and the
//!   reaping of an exited child (no zombie left parented to us, bl-3016).
//!
//! Shared fixtures (`write_script`, `collect`, `process_exists`) live here
//! so the submodules do not duplicate them. Binary resolution is tested
//! against an injected env lookup ([`Cli::resolve_with`]), so no test
//! mutates ambient env — there is no `ENV_LOCK`. The binary-wide
//! `SPAWN_LOCK` is the one static from `crate::test_support` (per-module
//! locks do not exclude each other's threads).

use super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;

// Spawn discipline rationale and the binary-wide lock live in
// crate::test_support — one static for every test module, because
// per-module locks do not exclude each other's threads.
use crate::test_support::{SpawnGuard, spawn_guard};

mod detach;
mod exec;
mod run;
mod spawn;
mod stream;
mod streamed;

fn write_script(dir: &Path, name: &str, body: &str) -> (PathBuf, SpawnGuard) {
    let guard = spawn_guard();
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    (path, guard)
}

fn collect(stream: Stream) -> (Vec<u8>, Vec<u8>, ExitInfo) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut exit = ExitInfo::Unknown;
    for chunk in stream {
        match chunk {
            Chunk::Stdout(b) => out.extend(b),
            Chunk::Stderr(b) => err.extend(b),
            Chunk::Exited(e) => exit = e,
        }
    }
    (out, err, exit)
}

fn process_exists(pid: u32) -> bool {
    // After the child exits and we've wait()ed it, its /proc entry is
    // gone too — the drop tests poll this to see a killed child vanish.
    Path::new(&format!("/proc/{pid}")).exists()
}
