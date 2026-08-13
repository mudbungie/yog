//! STORIES **S8-T3** hatches: `yog env` prints exactly the export lines the
//! world composes, and `yog exec <cmd…>` runs its argv under them with the
//! requested cwd — both pure entrypoints of the yog binary, neither a substrate
//! spawn (STORIES S8.3, DESIGN §8.4, §16.2, §16.6 W6).
//!
//! Driven against the **real** `yog` binary (`CARGO_BIN_EXE_yog`, the
//! `editor_roundtrip` idiom) under a private `XDG_DATA_HOME`, so the world it
//! composes is this test's and never the operator's. That is the only way to
//! assert the second half honestly: the run itself lives in `main.rs`, and a
//! test of the parse alone would not prove a shell ends up *in* the world.

#![allow(clippy::unwrap_used)]

use std::path::Path;
use tempfile::tempdir;
use yog::world::hatch::{self, ExecError};

/// Run the real `yog` with a private data-root anchor, returning
/// `(stdout, stderr, exit)`.
fn yog(anchor: &Path, args: &[&str]) -> (String, String, i32) {
    // Through `git_env::command` like every other fork in this crate — a bare
    // `Command::new` under `tests/` is unaudited by `make rules-audit`.
    let out = yog::git_env::command(Path::new(env!("CARGO_BIN_EXE_yog")))
        .env("XDG_DATA_HOME", anchor)
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

/// STORIES **S8-T3** hatches.
#[test]
fn s8_t3_yog_env_prints_the_world_and_yog_exec_runs_inside_it() {
    let anchor = tempdir().unwrap();
    let world_root = anchor.path().join("yog/world");

    // --- `yog env`: one `export VAR='value'` line per override, quoted so a
    // value with spaces survives the operator's `eval`.
    let (stdout, _, exit) = yog(anchor.path(), &["env"]);
    assert_eq!(exit, 0, "the hatch is a pure entrypoint, not a spawn");
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines
            .iter()
            .all(|l| l.starts_with("export ") && l.contains("='")),
        "every line is a quoted export: {lines:?}"
    );
    let keys: Vec<&str> = lines
        .iter()
        .filter_map(|l| l.strip_prefix("export ")?.split('=').next())
        .collect();
    assert_eq!(
        keys,
        ["LERNIE_HOME", "XDG_STATE_HOME", "PATH"],
        "the world's own override set, in its own order"
    );
    assert!(
        stdout.contains(&*world_root.join("lernie").to_string_lossy()),
        "the lines name THIS anchor's world: {stdout}"
    );
    assert!(stdout.contains(&*world_root.join("state").to_string_lossy()));
    // The composed script is exactly what the shared serializer produces from
    // the shared override set — the hatch adds no second opinion.
    assert_eq!(
        stdout,
        hatch::env_script(&[
            (
                "LERNIE_HOME".to_owned(),
                world_root.join("lernie").to_string_lossy().into_owned()
            ),
            (
                "XDG_STATE_HOME".to_owned(),
                world_root.join("state").to_string_lossy().into_owned()
            ),
            (
                "PATH".to_owned(),
                extract(&stdout, "PATH").expect("a PATH line")
            ),
        ]),
        "one serializer, one answer"
    );

    // --- `yog exec`: the argv runs IN the world, at the requested cwd.
    let cwd = tempdir().unwrap();
    let (stdout, stderr, exit) = yog(
        anchor.path(),
        &[
            "exec",
            "--cwd",
            &cwd.path().to_string_lossy(),
            "sh",
            "-c",
            r#"pwd -P; printf '%s\n' "$LERNIE_HOME" "$XDG_STATE_HOME""#,
        ],
    );
    assert_eq!(exit, 0, "stderr: {stderr}");
    let out: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        out[0],
        std::fs::canonicalize(cwd.path()).unwrap().to_string_lossy(),
        "the child ran where it was told"
    );
    assert_eq!(out[1], world_root.join("lernie").to_string_lossy());
    assert_eq!(out[2], world_root.join("state").to_string_lossy());

    // --- The parse is total and refuses rather than guessing.
    let words = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let plan = hatch::parse_exec(&words(&["bl", "list", "--json"])).unwrap();
    assert_eq!(plan.cwd, None, "no --cwd ⇒ the caller's own directory");
    assert_eq!(plan.cmd, "bl");
    assert_eq!(plan.args, ["list", "--json"]);
    // Only a LEADING `--cwd` is yog's; one further along belongs to the command.
    let plan = hatch::parse_exec(&words(&["bl", "--cwd", "/x"])).unwrap();
    assert_eq!(plan.cwd, None);
    assert_eq!(plan.args, ["--cwd", "/x"]);
    assert_eq!(
        hatch::parse_exec(&words(&["--cwd"])),
        Err(ExecError::MissingCwdValue)
    );
    assert_eq!(
        hatch::parse_exec(&words(&["--cwd", "/x"])),
        Err(ExecError::MissingCommand)
    );
    assert_eq!(hatch::parse_exec(&[]), Err(ExecError::MissingCommand));

    // A value with a quote in it survives the round trip through `eval`.
    assert_eq!(hatch::shell_quote("it's here"), r"'it'\''s here'");
}

/// The value of `export <key>=…` in an env script, unquoted.
fn extract(script: &str, key: &str) -> Option<String> {
    let line = script
        .lines()
        .find(|l| l.starts_with(&format!("export {key}=")))?;
    let value = line.split_once('=')?.1;
    Some(value.trim_matches('\'').to_owned())
}
