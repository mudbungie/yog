//! The fire (DESIGN §3.3, §8.1 step 2): mint the conversation's name and spawn
//! `lernie prompt` **detached** — the goal exactly as the operator edited it.
//!
//! Split from [`exec`](super::exec) at §12's 300-line budget, along the seam the
//! design already draws: everything else in the start flow is a piped, gated
//! substrate step, while this is the one irreversible launch — and the one place
//! the §3.3 conversation mint runs for real. The mint is re-derived *here*, not
//! carried from the composer's preview: another instance may have taken the
//! predicted name between preview and Enter, so the preview is a prediction and
//! this mint is the truth.

use super::identity::mint_conversation;
use super::{Prepared, StartError, on_mint};
use crate::cli_outbound::Cli;
use crate::names::Rng;
use crate::opslog::{self, DETACHED_EXIT, OpEntry};
use std::io;
use std::path::Path;

const PROMPT: &str = "prompt";
const NAME_FLAG: &str = "--name";
const YOG_NAME: &str = "YOG_NAME";

/// `lernie prompt --name <minted> <workspace> <goal>` fired **detached** (§8.1):
/// own process
/// group, stdin/stdout→null, stderr→the per-spawn sink ([`opslog::detached::sink`]),
/// `YOG_NAME=<workspace name>` layered (§8, §3.2 — the harness channel the goal
/// text no longer duplicates), cwd the §3.4 driver dir. `prepared` carries all
/// three, composed once by [`goal::compose_prepared`](super::goal).
///
/// The conversation name mints first ([`mint_conversation`] over `occupied` + the
/// injected `rng`); an exhausted pool leaves a `["yog-step","mint"]` row and
/// aborts before anything spawns (§4.2). The minted name rides `--name` (§3.3 as
/// ruled by bl-50f3): lernie commits it beside `goal.md`, the one durable home
/// the display ladder reads back — and on a lost-race re-mint the re-derived
/// name is what passes, since the mint here *is* the truth. What is sent is
/// `goal` **verbatim** — the operator's edited payload, unmutated (operator
/// ruling bl-6920): identity is `--name`'s alone, and lernie states the stored
/// name fact in its assembled context (lernie bl-d55f, released 0.0.4 —
/// `compose_system`'s `Your name is <name>.` in the system slot); yog prepends
/// nothing. The *logged* argv rides
/// the full goal through [`opslog::clip_goal`], which trims it so the serialized
/// line stays ≤ CAP/PIPE_BUF (§4.2 atomicity) — the *spawned* one is full. Only
/// the spawn is logged, and which line it writes is the outcome (bl-afa9): a
/// handoff logs [`DETACHED_EXIT`] and nothing else, a fork that never landed logs
/// the §4.2 synthetic-failure line with the error in `stderr`. A child that dies
/// *after* launching speaks through the sink, folded into the `-2` row at read
/// time (§13.3 amended).
///
/// Returns the **minted conversation name** — the one thing the fire learns that
/// no caller could have known beforehand (the preview only predicted it). It is
/// the handle the §3.4 focus claim is held by: the started root has no agent id
/// until the detached driver writes its branch, so the name is what identifies
/// the conversation until then ([`AppModel::await_conversation`](crate::AppModel::await_conversation)).
pub fn execute_prompt(
    lernie: &Cli,
    state_root: &Path,
    ts: &str,
    prepared: &Prepared,
    goal: &str,
    occupied: &[String],
    rng: &mut dyn Rng,
) -> Result<String, StartError> {
    let conversation = on_mint(
        mint_conversation(occupied, rng),
        state_root,
        ts,
        &prepared.cwd,
        prepared.origin,
    )?;
    let ws_s = prepared.workspace.to_string_lossy();
    let named = lernie.and_env(vec![(YOG_NAME.to_owned(), prepared.name.clone())]);
    let sink = opslog::detached::sink(state_root, ts, &prepared.workspace);
    // The goal stays LAST in both the spawned and the logged argv:
    // `opslog::clip_goal` trims exactly the final element (§4.2).
    let spawn = named.spawn_detached(
        Some(&prepared.cwd),
        &sink,
        &[PROMPT, NAME_FLAG, &conversation, ws_s.as_ref(), goal],
    );
    let argv = vec![
        lernie.binary().display().to_string(),
        PROMPT.to_owned(),
        NAME_FLAG.to_owned(),
        conversation.clone(),
        ws_s.into_owned(),
        goal.to_owned(),
    ];
    let cwd = prepared.cwd.display().to_string();
    // Two outcomes, two lines — never one sentinel for both (bl-afa9). A fork
    // that never landed is the ordinary §4.2 synthetic-failure line every other
    // never-launched spawn writes; `DETACHED_EXIT` is reserved for a handoff
    // that actually happened, and so records nothing but the launch.
    let entry = match spawn.as_ref().err() {
        Some(e) => {
            OpEntry::synthetic_failure(ts.to_owned(), argv, cwd, e.to_string(), prepared.origin)
        }
        None => OpEntry {
            ts: ts.to_owned(),
            argv,
            cwd,
            exit: DETACHED_EXIT,
            stdout: String::new(),
            stderr: String::new(),
            origin: prepared.origin,
        },
    };
    opslog::append(state_root, &opslog::clip_goal(&entry))?;
    spawn
        .map(|_pid| conversation)
        .map_err(|e| StartError::Io(io::Error::other(e)))
}
