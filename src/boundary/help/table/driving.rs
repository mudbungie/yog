//! The six §8.2 verbs whose subject is a conversation **already running**
//! (§8.5) — split from [`super`](super) at §12's budget (bl-c088) on the seam
//! the family itself draws: none of these creates or destroys anything, they
//! act on what is live, and each is spelled by a `litany` run. Everything left
//! in [`ACTIONS`](super::ACTIONS) mutates a ball, starts or ends a
//! conversation, spreads an attempt or routes a call.
//!
//! It is a prefix of the roster, not a second list: [`table`](crate::boundary::help::table)
//! joins it back ahead of `ACTIONS`, so an operator meets one list in one order.

use crate::boundary::help::{HelpRow, Surface};

/// The §8.2 conversation verbs: send, cut off, kill, sweep, prompt again, and
/// move onto the config the workspace runs now.
pub const DRIVING: &[HelpRow] = &[
    HelpRow {
        verb: "message",
        usage: "/message <text…>",
        summary: "send the text to the selected conversation and wake its driver",
        detail: "Deposits the text in the selected conversation's inbox and wakes its driver so \
                 it reads it (`litany message`). The text is the whole tail, verbatim — spacing \
                 and newlines reach the model unchanged, and no flag is read out of it. Takes \
                 the workspace and the agent from the seat; refuses when nothing is selected.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "interrupt",
        usage: "/interrupt <text…>",
        summary: "cut the selected conversation off mid-work and send it this text",
        detail: "Stops whatever is running the selected conversation and then deposits the text \
                 (`litany stop`, then `litany message`), so the model reads it now instead of at \
                 the end of what it is doing. The deposit is what restarts the conversation — \
                 there is no separate resume — so this leaves it running on your new text. Work \
                 already committed is kept, and a tool call cut off mid-flight is reported to the \
                 model in band as having produced no result. With nothing running it is simply a \
                 send. Two lines on the trail, one for each half, because the stop can be \
                 declined while the text still lands. The text is the whole tail, verbatim; no \
                 flag is read out of it, `children` included — use `/stop children` for a \
                 subtree. Takes the workspace and the agent from the seat; refuses when nothing \
                 is selected. Ctrl+Enter in the composer is this gesture.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "stop",
        usage: "/stop [children]",
        summary: "kill the selected conversation's driver; `children` cascades",
        detail: "Kills the driver running the selected conversation (`litany stop`). Everything \
                 it has already committed is kept, and it can be messaged again afterwards. Say \
                 `children` to stop the agents it spawned too, not only the one at its root.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "scan",
        usage: "/scan",
        summary: "flush the focused workspace's inboxes and deposit epitaphs",
        detail: "One workspace-wide sweep (`litany scan`): delivers pending inbox mail and \
                 deposits an epitaph for any agent that died silently. Acts on the focused \
                 workspace, not on the selection.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "nudge",
        usage: "/nudge",
        summary: "prompt the selected conversation again from where it already stands",
        detail: "Runs the model on the selected conversation as it is, with nothing added \
                 (`litany advance`): no new message, no goal retyped, the same conversation \
                 continued. This is the fix for a first turn that died before it reached the \
                 model — a missing sign-in, a provider row that was wrong — sign in, then nudge, \
                 and the turn is dispatched again in place. The driver runs detached, so it \
                 keeps going whatever yog does. Takes the workspace and the agent from the \
                 seat; refuses when nothing is selected, and does nothing while a driver is \
                 already running the conversation.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "retarget",
        usage: "/retarget",
        summary: "settle the selected conversation onto this workspace's config lineage",
        detail: "You do not need this to make a config edit reach a running conversation: a \
                 conversation follows its lineage's head at every step boundary, so a model you \
                 pick now governs this one at its next step by itself. What this changes is \
                 which *lineage* it follows — and it is the way out of the one state that has \
                 no head to follow, where two or more lineages have diverged over the \
                 conversation's fork point and it is held there until somebody says which. This \
                 runs `litany retarget`: it marks the conversation, and the conversation's own \
                 driver re-forks it onto this workspace's lineage at its next step, replaying \
                 everything it has already done on top — nothing is discarded and nothing is \
                 killed. It takes effect at that next step, never mid-step, which in practice is \
                 the message you send after it. Takes the workspace and the conversation from the \
                 seat; a conversation already on that lineage is a clean no-op litany reports for \
                 itself, and litany declines the move when the target does not describe the role \
                 it runs as.",
        surface: Surface::Control,
    },
];
