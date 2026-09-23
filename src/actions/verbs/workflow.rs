//! `litany workflow <ws> <agent> --config <name> | --clear` — the §9.4
//! **workflow mark** (REMOTE §9.24, bl-b680; litany ARCH §6, upstream
//! bl-5c02), its own file beside the table for the reason the table's doc
//! gives: one verb, one argv, one origin.
//!
//! Piped, not detached, for [`retarget`](super::retarget)'s reason exactly:
//! every refusal litany makes — an unknown lineage, a head whose `version` or
//! `workflow.yaml` will not parse, an agent the workspace has not got — comes
//! back in litany's own words rather than as a click that did nothing, and
//! every one of them precedes the write, so a declined switch leaves no debris.
//! The verb's one product on stdout is nothing (litany §3.4); the confirmation
//! rides stderr, which the captured [`Outcome`] carries whole.
//!
//! **Bare would be a read**, and yog never spells it: `litany workflow <ws>
//! <agent>` with neither flag answers which workflow governs on stdout, and
//! that answer rides `reply/governing` as `workflow_mark`, derived by yog over
//! the same refs (`config_edit::branch::workflow_mark`) rather than scraped
//! from a sentence. So this verb always names its direction — `Some` is
//! `--config <name>`, `None` is `--clear` — and the one thing it cannot
//! spawn is the inspection that would silently have marked, which litany
//! itself retired in bl-5c02.

use std::io;
use std::path::Path;

use super::{Bound, Outcome, run_logged};
use crate::opslog::Origin;

/// The litany subcommand (pinned to `src/bin/litany.rs`, §8.2).
const WORKFLOW: &str = "workflow";

/// Set the mark at `config`'s head, or clear it (`None`).
pub fn workflow(
    litany: &Bound,
    state_root: &Path,
    ts: &str,
    agent: &str,
    config: Option<&str>,
) -> io::Result<Outcome> {
    let ws_s = litany.workspace_arg();
    let mut args = vec![WORKFLOW, &ws_s, agent];
    match config {
        Some(name) => args.extend(["--config", name]),
        None => args.push("--clear"),
    }
    run_logged(
        litany.cli(),
        state_root,
        ts,
        litany.workspace(),
        &args,
        Origin::Conversation,
    )
}
