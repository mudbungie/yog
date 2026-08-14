//! The verb table's second half (§8.5): the gestures whose subject is a
//! **setting, a standing policy, or a record** — not a conversation or a ball.
//!
//! The cut is where the subject changes, and it is a real seam rather than a
//! line budget's arbitrary point: everything in [`super::ACTIONS`] spends a
//! substrate verb on one conversation or one ball, and everything here writes a
//! policy file, a control row or yog's own trail. `arm` is the first row on
//! this side because arming *is* the mechanism (VISION §4.3/§4.9) — nothing is
//! done to anything when it fires; a setting starts being true.
//!
//! [`table`](crate::boundary::help::table) reads the two as one, so the split
//! is invisible to an operator: no seat ever renders half a roster.

use super::super::HelpRow;

/// The standing families and the settings and record verbs, in the order help
/// lists them: the §4.9 monitor's three, the §4.3 loop's two, the §9 config
/// family, the trail's own two, the §6 queue's answer, and the §4.11
/// capability family.
pub const STANDING: &[HelpRow] = &[
    HelpRow {
        verb: "arm",
        usage: "/arm <model>",
        summary: "watch this workspace's agents with a cheap model, and say when work drifts",
        detail: "Arms the alignment monitor on the focused workspace, pinned to the model you \
                 name. From then on, whenever an agent commits, one small tool-less call reads \
                 its goal and the work since the last check and answers aligned, drifting or \
                 diverged with one sentence — recorded on the ops trail, which is the only thing \
                 it does by default. The question it asks lives in an editable policy file, \
                 seeded beside the settings the first time you arm. Costs money per check; \
                 `/disarm` ends it.",
    },
    HelpRow {
        verb: "disarm",
        usage: "/disarm",
        summary: "stop watching this workspace",
        detail: "Removes the focused workspace's monitor setting. No further checks are made and \
                 nothing further is charged; every verdict already recorded stays on the trail.",
    },
    HelpRow {
        verb: "flag",
        usage: "/flag <why…>",
        summary: "raise an attention item on the selected conversation, with a reason",
        detail: "Records that this conversation wants a human look, and why, as its own row on \
                 the ops trail. It changes nothing else — it does not stop, message or touch the \
                 conversation. It is also the one verb an alignment responder is granted by \
                 default, so that signalling out is a call with a shape rather than a sentence \
                 someone has to read.",
    },
    HelpRow {
        verb: "fleet",
        usage: "/fleet <cap>",
        summary: "run the focused project's ready balls here, up to this many at once",
        detail: "Arms the loop on the focused workspace: from then on, whenever it holds fewer \
                 than <cap> balls of the focused project, it claims the top ready one and starts \
                 a drone on it — the same start a ▶ Start click makes, through the same spend \
                 ceiling. It renders as facts on the board (how full, how often it looks, when \
                 it last acted) and every spawn and reap is a row on the trail. It never stops \
                 anything that is running. Add a `lease_min` to its cadence.yaml entry and it \
                 will also release a claim whose conversations have gone quiet for that long, \
                 recording the comparison that decided it. `/disband` ends it.",
    },
    HelpRow {
        verb: "disband",
        usage: "/disband",
        summary: "stop running a fleet in this workspace",
        detail: "Removes the focused workspace's fleet setting. Nothing further is claimed, \
                 started or released; everything already running is untouched and keeps its \
                 ball. Its own verb rather than a cap of zero, which is an armed loop that \
                 spawns nothing and still reaps.",
    },
    HelpRow {
        verb: "config",
        usage: crate::boundary::line::CONFIG_USAGE,
        summary: "read or write a config file: the destination's words, then the text (or nothing)",
        detail: "With text after the destination, writes one configuration file, carrying its \
                 entire new contents. The destination decides how it lands: `brazen` is \
                 validated by bz before it is written, into the seat's own workspace (its \
                 providers, sign-ins and model cache belong to that workspace and nowhere \
                 else); `models`, `cadence` and a `workflow` are refused if they name a \
                 provider row brazen does not have; a lineage (`branch` advances one, `fork` \
                 starts one from another, `orphan` starts a fresh one) is committed by \
                 `lernie config` on the seat's workspace. The text is everything \
                 after the destination's words, verbatim — whitespace is part of a config file. \
                 With nothing after the destination, reads its current bytes instead — a file \
                 not there yet answers empty text, a lineage answers what its tip holds at that \
                 path, and `/lineages` lists the lineages and the paths to ask for.",
    },
    HelpRow {
        verb: "marks",
        usage: "/marks | /marks <branch>",
        summary: "read, or amend, the branch this agent tracks its tasks on",
        detail: "Each agent tracks on a balls branch of its own, in a task space of its own — \
                 so two agents' task churn never collides. With a branch name, points the \
                 focused workspace's space at that branch; `balls/tasks` is the project's \
                 shared board, which is the branch an agent is pointed at when it is raised to \
                 work an existing project. Subagents inherit the space they were dispatched \
                 from, so a descent works one set of tasks. The answer is the branch re-read \
                 afterwards, beside the space it is a branch of, never an echo of what was \
                 asked. With no branch, reads the current one instead of changing it.",
    },
    HelpRow {
        verb: "model",
        usage: "/model <role> <provider> <model-id>",
        summary: "give a role this model, on the focused workspace's config lineage",
        detail: "Assigns a model to a role for the whole focused workspace: declares the model \
                 in lernie's global models.yaml if it is not there yet, then rewrites \
                 providers.yaml on the workspace's default config lineage. Both halves or \
                 neither — a provider row brazen does not have is refused before anything is \
                 written. Conversations already running keep the policy they were born on; the \
                 next one started here gets this.",
    },
    HelpRow {
        verb: "ack",
        usage: "/ack",
        summary: "acknowledge every alarm on the ops trail",
        detail: "Appends the acknowledgement line every failure-derived alarm reads past, so a \
                 failure you have understood and chosen to leave alone stops bannering. A new \
                 failure lands after the watermark and banners again.",
    },
    HelpRow {
        verb: "seen",
        usage: "/seen",
        summary: "answer the selected conversation's place in the attention queue",
        detail: "Records what the selected conversation is currently asking about as seen, which \
                 is what takes it off the attention queue — the same watermarks the window writes \
                 simply by having that conversation open, so a headless answer and a windowed one \
                 are the same thing. The answer is the queue that remains. It quiets what the \
                 conversation has *said*; undelivered mail is not a watermark and clears only \
                 when a driver reads it. New evidence re-raises it. Refuses when the conversation \
                 is not one yog can see.",
    },
    HelpRow {
        verb: "answer",
        usage: crate::boundary::line::ANSWER_USAGE,
        summary: "release, decline or keep parked the tool call held at this conversation",
        detail: "Answers the invocation the capability boundary parked before it ran. `pass` \
                 lets that one call through, `refuse` declines it in band — the model reads why \
                 and carries on — and `hold` keeps it parked even if the policy later would have \
                 passed it. The answer is scoped to the exact call that is held, which is read \
                 from the conversation's own hold mark, so nothing is typed and nothing can be \
                 spent by a different call. Passing or refusing then drives the conversation on \
                 (`lernie advance`), which is what actually lifts the hold: the control is asked \
                 again and now finds your answer. Nothing here stops the agent. Refuses when \
                 nothing is held there.",
    },
    HelpRow {
        verb: "revoke",
        usage: "/revoke",
        summary: "take away this conversation's tool auto-approval, and its descendants'",
        detail: "Stops letting the selected conversation act on its own: from its next tool \
                 call, everything but a read waits for you — the same park a held call already \
                 makes, applied to all of them. It keeps running, keeps its branch and keeps \
                 reading, so nothing is lost and nothing is killed. It covers the conversation \
                 and everything below it, including children it has not spawned yet. Anything \
                 the policy already refuses stays refused, and a call you pass with `/answer` \
                 still goes through. `/restore` gives the approval back.",
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
    },
    HelpRow {
        verb: "clear-trail",
        usage: "/clear-trail",
        summary: "truncate the ops trail; the clear is the new trail's first row",
        detail: "Starts a fresh `ops.jsonl`. The clear itself is logged as the new trail's first \
                 row, so the trail never lies about having been cut.",
    },
];
