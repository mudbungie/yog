//! The **follow-class** reads' page (REMOTE §3, §10; bl-73e7) — the two verbs
//! whose answer is a *sequence* rather than a value, and whose hold is a
//! connection thread.
//!
//! Split from [`queries`](super::queries) at §12's 300-line cap on the seam
//! REMOTE §3 already draws: every other read answers what is true now and is
//! done, while these two stay open and keep saying it. [`table`](super::super::table)
//! reads both halves as one, so the split is not a seam an operator can see.
//!
//! What each page has to explain is the same fact from two sides: an intake
//! that can hold a connection answers many frames, and one that cannot answers
//! one — so the reply a `yog gesture` prints is a true answer of the same
//! question, not a degraded one.
//!
//! Three since bl-c285: the sign-in's output is written at a provider's pace
//! and a human's, which is the same criterion (REMOTE §10) at a third subject.

use crate::boundary::help::{HelpRow, Surface};

pub const FOLLOWING: &[HelpRow] = &[
    HelpRow {
        verb: "invocations",
        usage: "/invocations",
        summary: "a tool host's next work: wait for what this machine has been asked to run",
        detail: "The routing leg's follow-class read. It does not answer until this machine has \
                 an invocation or the engine's hold expires — thirty seconds — so a tool host \
                 waits here rather than polling, and asks again the moment it is answered. It \
                 names nothing: the queue it drains is the one addressed to the certificate this \
                 connection presented, which is why a caller inside the world (`yog gesture`, \
                 the deposit inbox, the window) is refused rather than handed somebody's work. \
                 An empty answer is the ordinary answer of a hold that ended quietly, not a \
                 failure. Each row is `{\"invocation\": …, \"tool\": …, \"input\": …}`; run \
                 it and post the result with `/complete`.",
        surface: Surface::Machine,
    },
    HelpRow {
        verb: "follow",
        usage: "/follow",
        summary: "the selected conversation's live answer, as it is written",
        detail: "The streaming tail of the model call in flight, delivered at the rate the \
                 answer is written rather than at the rate a seat asks. Over a connection that \
                 can be held it is one frame per growth of the open response, and the stream \
                 ends when the step commits — at which point the text is committed and \
                 `/transcript` carries it. At a seat that cannot hold one it is the tail as of \
                 now, in a single frame, which is the same fold answered once. Either way it is \
                 `/transcript`'s own tail and never a second reading of it, so two seats \
                 watching one conversation cannot describe one moment differently. Takes the \
                 workspace and the conversation from the seat; a conversation with nothing in \
                 flight has an empty tail rather than a refusal.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "login-tail",
        usage: "/login-tail <provider>",
        summary: "watch a sign-in: what `bz --login` has printed for this row, as it prints it",
        detail: "The output of the run `/login <provider>` started in the focused workspace —                  the authorize URL, whatever else bz says, and the exit when it finishes, which                  is the last thing this ever says. It starts from the beginning every time it                  is asked, so re-asking after a dropped connection replays rather than losing                  the URL: a seat holds nothing between asks and there is no offset to name.                  Over a connection that can be held it stays open and speaks as the run does;                  at a seat that cannot it answers everything said so far, in one frame, which                  is the same reading answered once. A row with no run answers nothing said,                  which is what never-signed-in looks like — not a refusal.",
        surface: Surface::Control,
    },
];
