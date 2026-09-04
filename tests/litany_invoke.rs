//! **The re-exec target answers `invoke`** (bl-fe43; litany
//! `docs/DESIGN_CODE_EXECUTION.md` §2.8) — driven through the real world shim,
//! which is the one thing no in-process test can reach.
//!
//! litany's `python` built-in generates a `litany_tools` stub per tool the
//! injection declares, and every stub is a `subprocess.run([<driver target>,
//! "invoke"])` with a `{id, name, input}` block on stdin. The driver target is
//! the shim yog converges into `<world>/tools/litany` — a `/bin/sh` re-exec of
//! yog under the `litany` namespace — so a program's inner invocation reaches
//! the engine's front door only if that shim answers the verb. It does, because
//! the arm is litany's thin binding and builds one `Fx` for every verb: the
//! injection the router lives behind is installed for `invoke` exactly as it is
//! for `advance` and `tool`, with no verb table of yog's own to fall out of
//! step.
//!
//! The proof is two-directional against the SAME shim: an unknown verb is
//! clap's usage error, and `invoke` is not — it reaches litany's own door and
//! fails there, on the contract environment it was handed none of.
#![allow(clippy::unwrap_used)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Run the built `yog` with a hermetic world under `home`, so nothing here can
/// read or write the operator's own state.
fn yog(home: &Path, args: &[&str]) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_yog"))
        .args(args)
        .envs(hermetic(home))
        .status()
        .unwrap()
        .code()
        .unwrap_or(-1)
}

fn hermetic(home: &Path) -> Vec<(String, PathBuf)> {
    ["XDG_DATA_HOME", "XDG_STATE_HOME", "XDG_CONFIG_HOME", "HOME"]
        .into_iter()
        .map(|k| (k.to_owned(), home.join(k.to_lowercase())))
        .collect()
}

/// Spawn the world's `litany` shim exactly as a generated stub does — the verb
/// on argv, the block on stdin, nothing else — and answer `(exit, stderr)`.
fn shim(home: &Path, verb: &str, block: &str) -> (i32, String) {
    let target = home.join("xdg_data_home/yog/world/tools/litany");
    assert!(target.is_file(), "no driver target at {}", target.display());
    let mut child = Command::new(&target)
        .arg(verb)
        .envs(hermetic(home))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(block.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn the_re_exec_target_answers_the_door_verb() {
    let home = tempfile::TempDir::new().unwrap();
    // Any verb converges the shims on its way in; `prime` is the cheapest, and
    // its seed lands in the world rather than in any ambient harness root.
    assert_eq!(yog(home.path(), &["litany", "prime"]), 0);

    // The control direction: a verb the surface does not carry is clap's usage
    // error, so the assertion below is about `invoke` and not about a shim that
    // answers everything with the same code.
    let (code, said) = shim(home.path(), "no-such-verb", "");
    assert_eq!(code, 2, "{said}");
    assert!(said.contains("unrecognized subcommand"), "{said}");

    // `invoke` is a verb: the block parses, and the failure is litany's own
    // door failing to resolve whose invocation this is — the §3.3 contract
    // environment, which a stub inherits from the tool being executed and this
    // beat deliberately withholds. Reaching that sentence means the arm parsed
    // the verb, ran its preludes, built the `Fx` the injection stands on, and
    // handed litany the block.
    let (code, said) = shim(
        home.path(),
        "invoke",
        r#"{"id":"toolu_01-1","name":"read_file","input":{"path":"AGENTS.md"}}"#,
    );
    assert_eq!(code, 1, "{said}");
    assert!(said.starts_with("litany invoke: "), "{said}");
    assert!(said.contains("LITANY_CONV_REPO"), "{said}");
}
