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

use crate::boundary::help::HelpRow;

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
    },
];
