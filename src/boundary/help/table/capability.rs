//! The **capability family's** pages (VISION §4.11): answering a park, and the
//! §4.9 fifth rung's two directions over the same fold. Split off
//! [`standing`](super::standing) at §12's budget, and joined back beside it —
//! no split here is a seam an operator can see.

use super::{HelpRow, Surface};

/// `/answer`, `/revoke` and `/restore`.
pub const CAPABILITY: &[HelpRow] = &[
    HelpRow {
        verb: "answer",
        usage: crate::boundary::line::ANSWER_USAGE,
        summary: "release, decline or keep parked the tool call held at this conversation",
        detail: "Answers the invocation the capability boundary parked before it ran. `pass` \
                 lets it through, `refuse` declines it — the model is told the decision stands, \
                 and told not to retry it, rephrase it or reach the same outcome another way — \
                 and `hold` keeps it parked even if the policy later would have passed it. The \
                 call is read from the conversation's own hold mark, so nothing is typed and \
                 nothing can be spent by a different call. \
                 `--scope` says how far the answer stands. Left off it settles that one call, \
                 which is the safe reading and the one you get without thinking about it. \
                 `--scope conversation` settles the whole CLASS of the held call — the same \
                 tool at the same reach — for this conversation and everything below it; \
                 `--scope workspace` settles that class for every conversation here. A wider \
                 answer is how you stop being asked eleven times about one narrow tool. \
                 Answer the same scope again to change it, and `/revoke` suspends every \
                 standing answer, which is how you take one back. A destructive or \
                 credential-reaching call takes the bare form only: those are decided for the \
                 call in front of you, never for calls to come. \
                 Passing or refusing then drives the conversation on (`litany advance`), which \
                 is what actually lifts the hold: the control is asked again and now finds your \
                 answer. Nothing here stops the agent. Refuses when nothing is held there.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "revoke",
        usage: "/revoke",
        summary: "take away this conversation's tool auto-approval, and its descendants'",
        detail: "Stops letting the selected conversation act on its own: from its next tool \
                 call, everything but a read waits for you — the same park a held call already \
                 makes, applied to all of them. **A read is what the call does, not which \
                 tool it is**: every invocation is classified on what it reaches, so `bash` \
                 running `ls`, `cat` or `echo` is a read and still runs, while the same \
                 `bash` writing outside its worktree, reaching the network or touching \
                 credentials parks. It keeps running, keeps its branch and keeps \
                 reading, so nothing is lost and nothing is killed. It covers the conversation \
                 and everything below it, including children it has not spawned yet. Anything \
                 the policy already refuses stays refused, and a call you pass with `/answer` \
                 still goes through. It also **suspends every standing answer** (bl-94a5): a \
                 class you released for this conversation or this workspace comes back to you \
                 one call at a time, which is both the point of a revoke and the way to take a \
                 standing release back. `/restore` gives the approval back.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "restore",
        usage: "/restore",
        summary: "give this conversation's tool auto-approval back",
        detail: "Lifts a floor `/revoke` put on the selected conversation: its calls are \
                 adjudicated by the ordinary policy again, from its next one. It drives nothing \
                 — a conversation parked at a held call is released by answering that call \
                 (`/answer pass`), which is the thing you are looking at when it is waiting. If \
                 an ancestor is still revoked, the conversation stays floored under it, and the \
                 reply says so rather than claiming a restore it did not make.",
        surface: Surface::Control,
    },
];
