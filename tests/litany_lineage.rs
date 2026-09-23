//! **The escalation bl-baed measured, driven end to end through the real
//! world shim** — the one chain no in-process test can reach.
//!
//! The defect: a worker with the shipped `bash` grant and no foot enrolled
//! takes REMOTE §5.4's third rung, the engine's own front door. It writes a
//! script, exports it as `$EDITOR`, and types `litany config <workspace>` —
//! which the world's `PATH` resolves to yog's own shim (DESIGN §16.4). The
//! lineage advanced: `souls/*.md`, `providers.yaml`'s model and grant rows,
//! `facts.md` and the workspace skills, all rewritten from inside the
//! conversation those very files govern, walking the learning loop's operator
//! veto (litany `docs/DESIGN_LEARNING_LOOP.md` §3).
//!
//! **The door that refuses is litany's, and yog's whole part is to hand it the
//! truth.** `litany config` and `litany proposal --accept` — the only two acts
//! that advance a `config/*` branch — refuse when `LITANY_TOOL_ID` is set
//! (upstream litany bl-d273). yog owes the marker at three places and every
//! one of them is a link in this chain:
//!
//! 1. the front-door spawn puts `LITANY_TOOL_ID` on the child
//!    (`src/tool_host/engine_act.rs`, `const TOOL_ID`);
//! 2. the shim is a bare `/bin/sh` re-exec that strips nothing
//!    (`src/world/tools.rs`, `shim_script`);
//! 3. the multiplex arm reads it back at the binding and fills `Fx::tool_id`
//!    unchanged (`src/multiplex/litany.rs`).
//!
//! Each link has its own beat. None of them proves the *chain*, and the chain
//! is what the ball measured: a step's shell inherits the marker, so the shim
//! it types is already marked before litany ever parses a verb. That is what
//! this file drives, both ways, against the built binary — a refusal that
//! materializes nothing and leaves `config/default` byte-identical, and the
//! same call with the marker absent landing, so the gate cannot pass by
//! refusing everything.
// clippy's allow-*-in-tests reaches `#[test]` fns, not the free fixture
// helpers of an integration-test crate (the `tests/support` precedent).
#![allow(clippy::unwrap_used)]

// The executable-fixture writer, shared by every integration binary that
// writes one (bl-fd28): an `fs::write` here would hand a live write fd to any
// peer thread that forks, and an exec of the script inside that window is
// ETXTBSY. `#[path]` because this file IS the test target's crate root.
#[path = "support/write_exec.rs"]
mod write_exec;

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The workspace's one lineage branch — what the escalation advanced.
const LINEAGE: &str = "config/default";

/// The marker litany refuses under, spelled as the shell sees it.
const TOOL_ID: &str = "LITANY_TOOL_ID";

/// A hermetic world under `home`, so nothing here reads or writes the
/// operator's own state: the roots yog folds from, plus the git identity
/// litany's minted repos commit through (a runner has none) and a wall against
/// every other ambient global setting.
fn hermetic(home: &Path) -> Vec<(String, String)> {
    let mut env: Vec<(String, String)> = ["XDG_DATA_HOME", "XDG_STATE_HOME", "XDG_CONFIG_HOME"]
        .into_iter()
        .map(|k| (k.to_owned(), path(&home.join(k.to_lowercase()))))
        .collect();
    env.push(("HOME".to_owned(), path(home)));
    env.push(("GIT_CONFIG_GLOBAL".to_owned(), path(&gitconfig(home))));
    env.push(("GIT_CONFIG_SYSTEM".to_owned(), "/dev/null".to_owned()));
    env
}

fn path(p: &Path) -> String {
    p.display().to_string()
}

/// The scratch global gitconfig, written once per home.
fn gitconfig(home: &Path) -> PathBuf {
    let file = home.join("gitconfig");
    if !file.is_file() {
        std::fs::write(
            &file,
            "[user]\n\tname = Tester\n\temail = t@test.invalid\n[commit]\n\tgpgsign = false\n",
        )
        .unwrap();
    }
    file
}

/// A child of this test, hermetic and with the git vars a hook-invoked run may
/// inherit scrubbed — `git` exports `GIT_DIR`/`GIT_INDEX_FILE` into every
/// process it starts and they outrank `-C`, so an unscrubbed child would fork
/// its own git against the hook's repo (AGENTS.md, bl-916a).
fn child(program: &Path, home: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.envs(hermetic(home));
    for var in yog::git_env::INHERITED {
        cmd.env_remove(var);
    }
    cmd
}

/// Run the built `yog` and answer its exit code.
fn yog(home: &Path, args: &[&str]) -> i32 {
    child(Path::new(env!("CARGO_BIN_EXE_yog")), home)
        .args(args)
        .status()
        .unwrap()
        .code()
        .unwrap_or(-1)
}

/// The world's `litany` shim — the file an agent's `PATH` resolves.
fn shim(home: &Path) -> PathBuf {
    let target = home.join("xdg_data_home/yog/world/tools/litany");
    assert!(target.is_file(), "no shim at {}", target.display());
    target
}

/// **One `bash` step, exactly as REMOTE §5.4's third rung performs it**:
/// `<driver target> tool bash` with the `tool_use.input` block on stdin and
/// litany's §3.3 contract on the environment — the caller identity and, when
/// `marked`, the invocation's own id. Answers the step's exit code and
/// everything the model would read back.
fn bash_step(home: &Path, ws: &Path, editor: &Path, command: &str, marked: bool) -> (i32, String) {
    let mut cmd = child(&shim(home), home);
    cmd.args(["tool", "bash"])
        .env("EDITOR", editor)
        .env("LITANY_CONV_REPO", ws)
        .env("LITANY_CONV_BRANCH", "agents/dulcet-mongoose")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if marked {
        cmd.env(TOOL_ID, "toolu_step_1");
    } else {
        cmd.env_remove(TOOL_ID);
    }
    let mut running = cmd.spawn().unwrap();
    let block = serde_json::json!({ "command": command }).to_string();
    running
        .stdin
        .take()
        .unwrap()
        .write_all(block.as_bytes())
        .unwrap();
    let out = running.wait_with_output().unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code().unwrap_or(-1), said)
}

/// The lineage's tip, read out of the workspace's bare repository — the fact
/// the escalation moved and a refusal must leave exactly where it stood.
fn tip(home: &Path, ws: &Path) -> String {
    let out = child(Path::new("git"), home)
        .args(["-C", &path(&ws.join("repo.git")), "rev-parse", LINEAGE])
        .output()
        .unwrap();
    assert!(out.status.success(), "rev-parse {LINEAGE}");
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// The agent's own script, handed to `litany config` as `$EDITOR`: it writes a
/// durable fact into the checkout it is given and leaves a sentinel saying it
/// ran. An editor that WOULD write, so a refusal arriving after the hand-off
/// would still show its hand.
fn scripted_editor(home: &Path, sentinel: &Path) -> PathBuf {
    let script = home.join("editor.sh");
    write_exec::write_exec(
        &script,
        &format!(
            "#!/bin/sh\ntouch '{}'\nprintf '%s\\n' \
             '- the retention figure is 45 days' > \"$1/facts.md\"\n",
            sentinel.display()
        ),
    );
    script
}

#[test]
fn a_bash_step_cannot_advance_its_own_config_lineage_through_the_world_shim() {
    let scratch = tempfile::TempDir::new().unwrap();
    let home = scratch.path();
    // Any verb converges the shims on its way in; `prime` is the cheapest.
    assert_eq!(yog(home, &["litany", "prime"]), 0);
    let ws = home.join("ws");
    assert_eq!(yog(home, &["litany", "new", &path(&ws)]), 0);

    let sentinel = home.join("editor-ran");
    let editor = scripted_editor(home, &sentinel);
    let before = tip(home, &ws);
    let advance = format!("litany config {}", path(&ws));

    // **The escalation itself.** The step's shell inherits the marker from the
    // front-door spawn, finds `litany` on the world's PATH, and is refused at
    // litany's own door — ahead of the root resolution and the transient
    // checkout, so nothing is materialized and no ref is read.
    let (code, said) = bash_step(home, &ws, &editor, &advance, true);
    assert_ne!(code, 0, "a step advanced its own config lineage: {said}");
    assert!(said.contains(TOOL_ID), "{said}");
    assert!(said.contains("litany proposal <workspace>"), "{said}");
    assert!(
        !sentinel.exists(),
        "the refusal stands ahead of the checkout: no editor was handed one"
    );
    assert_eq!(tip(home, &ws), before, "{LINEAGE} moved under a refusal");

    // `litany proposal --accept` is the lineage's other door and is refused on
    // the same marker, ahead of resolving the id it was given.
    let accept = format!("litany proposal {} no-such-reviewer --accept", path(&ws));
    let (code, said) = bash_step(home, &ws, &editor, &accept, true);
    assert_ne!(code, 0, "{said}");
    assert!(said.contains(TOOL_ID), "{said}");

    // **The other direction, through the same mechanism**: unmarked, the very
    // same command is the operator's and lands. Without this the beat would
    // pass just as well against a shim that refuses everything.
    let (code, said) = bash_step(home, &ws, &editor, &advance, false);
    assert_eq!(code, 0, "{said}");
    assert!(sentinel.exists(), "the operator's own edit still runs");
    assert_ne!(tip(home, &ws), before, "{LINEAGE} did not advance");
}
