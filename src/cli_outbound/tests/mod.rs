//! Tests for cli_outbound, split by concern to leave growth room under
//! the 300-line source cap:
//! - [`run`]: constructor, binary resolution, argv, happy-path streaming,
//!   `run_in` cwd propagation.
//! - [`stream`]: chunk iteration, `exit_info` classification, `pump_step`.
//! - [`spawn`]: spawn-error and drop→SIGTERM→SIGKILL cleanup.
//! - [`self_exe`]: which file yog itself is — the `stat` judgement over one
//!   reading and the process-lifetime memo over it (bl-f558).
//! - [`detach`]: `spawn_detached` — parent-survival, cwd propagation, the
//!   per-spawn stderr sink (and its null degradation), spawn error, and the
//!   reaping of an exited child (no zombie left parented to us, bl-3016).
//!
//! Shared fixtures (`write_script`, `collect`, `process_exists`) live here
//! so the submodules do not duplicate them. Binary resolution is tested
//! against an injected env lookup ([`Cli::resolve_with`]), so no test
//! mutates ambient env. Every script here is authored by
//! `crate::test_support::write_exec`, which is the whole of the ETXTBSY
//! discipline since bl-fd28 — no lock, no bracket, no per-test contract.

use super::*;
use std::fs;

// Spawn discipline rationale and the binary-wide lock live in
// crate::test_support — one static for every test module, because
// per-module locks do not exclude each other's threads.

mod detach;
mod exec;
mod run;
mod self_exe;
mod spawn;
mod stream;
mod streamed;
mod wrap;

fn write_script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    crate::test_support::write_exec(&path, body);
    path
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
