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
use super::instructions::{names, specs};
use super::{Prepared, StartError, on_mint};
use crate::cli_outbound::Cli;
use crate::opslog::{self, DETACHED_EXIT, OpEntry};
use lernie::mint::Rng;
use std::io;
use std::path::Path;

const PROMPT: &str = "prompt";
const NAME_FLAG: &str = "--name";
/// lernie's creation-time working-directory parameter (upstream bl-d0b4,
/// released 0.0.8): the §3.3 typed work-target binding's one channel.
const CWD_FLAG: &str = "--cwd";
/// lernie's caller-supplied pinned document (upstream bl-fb5c, released 0.0.4):
/// the §3.7 project-instruction freeze's one channel.
const PIN_FLAG: &str = "--pin";
const YOG_NAME: &str = "YOG_NAME";

/// `lernie prompt --name <minted> [--cwd <target>] <workspace> <goal>` fired
/// **detached** (§8.1): own process
/// group, stdin/stdout→null, stderr→the per-spawn sink ([`opslog::detached::sink`]),
/// `YOG_NAME=<workspace name>` layered (§8, §3.2 — the harness channel the goal
/// text no longer duplicates), the driver standing in the workspace it drives.
/// `prepared` carries all of it,
/// composed once by [`goal::compose_prepared`](super::goal).
///
/// **`--cwd` is the work target's one channel** (§3.3, bl-6654 consuming
/// bl-2b8c's ruling / VISION §4.10 item 2): the rung's typed binding — the ball
/// rung's claim-derived `work/<id>` worktree, the path rung's directory — seeds
/// the agent's working-directory mark at creation, so every tool step of every
/// later turn runs there. It rides only when the rung binds something
/// ([`Prepared::binding`](super::Prepared::binding)); an absent flag is lernie's
/// own default, the agent's worktree, which is exactly what the bare rung
/// means. It replaced a paragraph of goal prose naming the path, which reached
/// the model as content it had to notice and obey, and the initial process's
/// `current_dir`, which reached no tool step at all.
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
    rng: &dyn Rng,
) -> Result<String, StartError> {
    let conversation = on_mint(
        mint_conversation(occupied, rng),
        state_root,
        ts,
        &prepared.workspace,
        prepared.origin,
    )?;
    let ws_s = prepared.workspace.to_string_lossy();
    let bound = prepared.binding.as_ref().map(|p| p.to_string_lossy());
    // The §3.7 freeze: one `--pin` per instruction document the binding's
    // project declares, discovered from the binding's own authority root. No
    // binding, no discovery — the bare rung reads no policy and stats no file.
    let pins: Vec<String> = prepared.binding.as_deref().map_or_else(Vec::new, |target| {
        specs(target, &names::names(&prepared.workspace))
    });
    let named = lernie.and_env(vec![(YOG_NAME.to_owned(), prepared.name.clone())]);
    let sink = opslog::detached::sink(state_root, ts, &prepared.workspace);
    // One argv, built once and spawned *and* logged from it — so the flag that
    // rides conditionally cannot ride in only one of them. The goal stays LAST
    // in both: `opslog::clip_goal` trims exactly the final element (§4.2), so
    // every pin survives into the trail and *is* the provenance record (§3.7).
    let mut args = vec![PROMPT, NAME_FLAG, conversation.as_str()];
    if let Some(dir) = bound.as_deref() {
        args.extend([CWD_FLAG, dir]);
    }
    for pin in &pins {
        args.extend([PIN_FLAG, pin.as_str()]);
    }
    args.extend([ws_s.as_ref(), goal]);
    // The driver's own directory is the workspace it drives — the same for every
    // rung since bl-6654 retired the per-target `current_dir`. It is where the
    // process stands, not where the work is: the work target is `--cwd` above.
    let spawn = named.spawn_detached(Some(&prepared.workspace), &sink, &args);
    let argv: Vec<String> = std::iter::once(lernie.binary().display().to_string())
        .chain(args.iter().map(|a| (*a).to_owned()))
        .collect();
    let cwd = ws_s.clone().into_owned();
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
