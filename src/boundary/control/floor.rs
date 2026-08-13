//! The capability family's **floor writer** (VISION §4.9's fifth rung, §4.11
//! item 7, DESIGN §8.6): revoke one conversation's tool auto-approval, and
//! restore it.
//!
//! One `ops.jsonl` row, and nothing else:
//!
//! ```text
//! ["yog-control","floor",<conversation id>,"raise"|"lower"]
//! ```
//!
//! which is at once the audit and the memory the control folds on its next
//! consult ([`crate::control::judge::Answers`] reads it back). The reader
//! landed with the shim; this is the writer, and between them the floor is a
//! *fold over rows* rather than a field anywhere — latest row wins, so the two
//! directions are one gesture and there is no order to get wrong.
//!
//! **Three things it deliberately does not do.**
//!
//! 1. *It launches nothing.* Answering a park drives the branch on because an
//!    answer is about one invocation that is waiting; a floor is standing
//!    policy, and §4.11 item 6 binds policy at the **next consult**. A restore
//!    that also drove would spend a process on a conversation that may not be
//!    parked at all, and the branch a floor *did* park is released by the
//!    answer gesture that already exists — the one the operator is looking at.
//! 2. *It carries no reason.* The row's reason is the row before it: the
//!    monitor's own `["yog-monitor",<verdict>,…]` line with its sentence, or a
//!    `["yog-flag",…]`. Re-typing it here would give one fact two homes, and
//!    the trail is read in order.
//! 3. *It checks nothing exists.* The floor matches by descent prefix, so
//!    naming a conversation whose children are not born yet is the mechanism
//!    working, not a mistake — refusing an absent id would refuse exactly the
//!    pre-emptive floor the subtree match is for.
//!
//! **The receipt is re-derived, never echoed** (the `marks` precedent): the
//! reply says whether a floor *stands* over the conversation now, read back off
//! the trail this call just appended to. Those differ in the case that matters
//! — restoring a child whose parent is still floored leaves the child floored,
//! and a receipt that answered "restored" there would be a lie.

use std::path::Path;

use crate::boundary::dispatch::Deps;
use crate::boundary::reply::Reply;
use crate::control::judge::Answers;
use crate::opslog::{self, OpEntry, Origin, YOG_CONTROL};

/// The ops-row verb naming a per-conversation floor, and its two states.
/// Mirrored from the fold that reads them, exactly as the once-answer's word
/// is: the reader owns its grammar, and a test holds the spellings equal.
const FLOOR: &str = "floor";
const RAISE: &str = "raise";
const LOWER: &str = "lower";

/// Write `agent`'s floor row on `workspace`'s trail, and answer with the floor
/// that stands over it afterwards.
pub(crate) fn set_floor(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    agent: &str,
    raised: bool,
) -> Result<Reply, String> {
    let state = if raised { RAISE } else { LOWER };
    let row = OpEntry {
        ts: ts.to_owned(),
        argv: vec![
            YOG_CONTROL.to_owned(),
            FLOOR.to_owned(),
            agent.to_owned(),
            state.to_owned(),
        ],
        cwd: crate::nav::ws_key(workspace),
        exit: 0,
        stdout: String::new(),
        stderr: String::new(),
        // The subject is a conversation, which is what §7.3 attribution names.
        origin: Origin::Conversation,
    };
    opslog::append(&deps.state_root, &row).map_err(|e| e.to_string())?;
    let standing = Answers::fold(&opslog::tail(&deps.state_root, usize::MAX)).floored(agent);
    Ok(Reply::Floored { standing })
}
