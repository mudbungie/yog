//! **The releasing drive** (§8.6, §8.2): `litany advance <ws> <agent>`, fired
//! detached. Two callers, one body (bl-9bef) — the capability answer's release
//! and the operator's own nudge — so a driver launch has one home.

use std::path::Path;

use crate::opslog::{self, DETACHED_EXIT, OpEntry, Origin};

use super::super::dispatch::Deps;

// litany's re-drive verb — `litany advance <ws> <agent>` (its ARCH §6): one
// hop of the workflow chain, which re-enters the tool window under the mark
// and re-consults the control. That re-consult *is* the release. The token is
// `opslog::launch`'s since bl-b95e — the launch writes it into `ops.jsonl` and
// the §8.1 verdict reads it back out, so the join has one home.
use opslog::launch::ADVANCE;

/// Fire `litany advance <ws> <agent>` detached, logging the launch exactly as
/// the §8.1 fire logs its own: [`DETACHED_EXIT`] for a handoff that happened, a
/// §4.2 synthetic-failure line for a fork that never landed. The row is the
/// receipt — the answer's own reply says only whether the launch was made.
///
/// **Two callers, one body** (bl-9bef): the release above, and the §8.2 nudge —
/// the operator's own "run it again from here", which is this launch and
/// nothing else, since litany derives what is due from the transcript tail
/// (ARCH §6). Shared rather than re-written, so a driver launch has one home.
///
/// **The spawn is workspace-bound** ([`Deps::bound`], bl-bf79): what this
/// launches is a *driver*, which makes model calls, so it owes its workspace
/// the §16.2 wall — without it the driver's first `bz` dies with `no workspace
/// in this environment` and the turn produces an empty reply. That is the same
/// fold every §8.2 litany verb takes, and it was missing here.
pub(crate) fn advance(deps: &Deps, ts: &str, workspace: &Path, agent: &str) -> Result<(), String> {
    let ws_s = workspace.to_string_lossy();
    let sink = opslog::detached::sink(&deps.state_root, ts, workspace);
    let bound = deps.bound(workspace);
    let spawn =
        bound
            .cli()
            .spawn_detached(Some(workspace), &sink, &[ADVANCE, ws_s.as_ref(), agent]);
    let argv = vec![
        deps.litany.binary().display().to_string(),
        ADVANCE.to_owned(),
        ws_s.into_owned(),
        agent.to_owned(),
    ];
    let cwd = crate::nav::ws_key(workspace);
    let entry = match spawn.as_ref().err() {
        Some(e) => OpEntry::synthetic_failure(
            ts.to_owned(),
            argv,
            cwd,
            e.to_string(),
            Origin::Conversation,
            deps.caller.client.clone(),
        ),
        None => OpEntry {
            ts: ts.to_owned(),
            argv,
            cwd,
            exit: DETACHED_EXIT,
            stdout: String::new(),
            stderr: String::new(),
            origin: Origin::Conversation,
            client: deps.caller.client.clone(),
        },
    };
    opslog::append(&deps.state_root, &entry).map_err(|e| e.to_string())?;
    spawn.map(|_pid| ()).map_err(|e| e.to_string())
}
