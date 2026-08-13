//! The verb table's first half (§8.5): the mutating gestures — every command,
//! how it is typed, and what it does.
//!
//! **A detail says three things**: what the gesture does, what it takes from
//! the seat when the line does not say it, and how it refuses. That is what an
//! operator needs before pressing return on something that mutates a substrate.

use super::HelpRow;

pub mod queries;

/// Every **action** the boundary answers to, in the order help lists them:
/// the conversation verbs, the ball verbs, the start pair, the V2 attempt, the
/// §3.6 deletes, the §9 config family, the §4.9 monitor and the trail's own two.
/// The queries are [`queries::QUERIES`]; [`table`](super::table) reads the two
/// as one, so the split is §12's cap and never a seam an operator can see.
pub const ACTIONS: &[HelpRow] = &[
    HelpRow {
        verb: "message",
        usage: "/message <text…>",
        summary: "send the text to the selected conversation and wake its driver",
        detail: "Deposits the text in the selected conversation's inbox and wakes its driver so \
                 it reads it (`lernie message`). The text is the whole tail, verbatim — spacing \
                 and newlines reach the model unchanged, and no flag is read out of it. Takes \
                 the workspace and the agent from the seat; refuses when nothing is selected.",
    },
    HelpRow {
        verb: "stop",
        usage: "/stop [children]",
        summary: "kill the selected conversation's driver; `children` cascades",
        detail: "Kills the driver running the selected conversation (`lernie stop`). Everything \
                 it has already committed is kept, and it can be messaged again afterwards. Say \
                 `children` to stop the agents it spawned too, not only the one at its root.",
    },
    HelpRow {
        verb: "scan",
        usage: "/scan",
        summary: "flush the focused workspace's inboxes and deposit epitaphs",
        detail: "One workspace-wide sweep (`lernie scan`): delivers pending inbox mail and \
                 deposits an epitaph for any agent that died silently. Acts on the focused \
                 workspace, not on the selection.",
    },
    HelpRow {
        verb: "close",
        usage: "/close [id]",
        summary: "close a ball (the focused one by default) and deliver its work",
        detail: "Delivers a ball (`bl close`): folds `main` into its worktree, runs the \
                 project's pre-commit gate, squashes the work onto the target branch and removes \
                 the worktree. A failing gate aborts and leaves the ball claimed. With no id, \
                 the focused ball; stamped with its claimant, never your login name.",
    },
    HelpRow {
        verb: "assign",
        usage: "/assign [id]",
        summary: "claim a ready ball for this workspace",
        detail: "Claims a ready ball for the seat's workspace (`bl claim`), which is what binds \
                 the ball to it: a bound ball is one a workspace holds. With no id, the \
                 focused ball.",
    },
    HelpRow {
        verb: "release",
        usage: "/release [id]",
        summary: "unclaim a ball this workspace holds",
        detail: "Lets a ball go (`bl unclaim`): the workspace stops holding it and anyone can \
                 claim it again. Nothing already committed in its worktree is lost. With no id, \
                 the focused ball.",
    },
    HelpRow {
        verb: "move",
        usage: "/move [id] <to>",
        summary: "re-home a bound ball onto another workspace",
        detail: "Re-homes a bound ball: released here, claimed there, in one gesture. One word \
                 is the destination for the focused ball; two are the ball and its destination.",
    },
    HelpRow {
        verb: "create",
        usage: "/create <title…> [--body <text…>]",
        summary: "create a ball in the focused project",
        detail: "Creates a ball in the seat's project (`bl create`) and prints nothing but its \
                 id in the reply. The title is the words before any flag; `--body` carries the \
                 rest of the description. Both are whitespace-normalized — a line is a line.",
    },
    HelpRow {
        verb: "update",
        usage: "/update [id] [--title T] [--body B] [--note N]",
        summary: "amend a ball's title or body, or append a journal note",
        detail: "Amends a ball (`bl update`). At least one field is required, or the line asked \
                 for nothing; give any combination. `--note` appends to the ball's journal \
                 rather than replacing anything. With no id, the focused ball.",
    },
    HelpRow {
        verb: "prepare",
        usage: "/prepare | /prepare dir <path> | /prepare ball [--new <title…> [--body <text…>]]",
        summary: "the start flow's mutating half: seed, workspace, ball rung",
        detail: "Runs everything a new conversation needs before it is prompted: the mint, the \
                 workspace if it does not exist yet, and the ball rung's `bl` steps. The rung is \
                 said outright, never inferred — nothing is the bare rung, `dir` a work \
                 directory, `ball` the selected ball (or `--new` to mint one). Its reply is what \
                 `/prompt` then fires; an existing ball's spec comes from the seat's roster.",
    },
    HelpRow {
        verb: "prompt",
        usage: "/prompt <goal…>",
        summary: "fire the prepared start with this goal, verbatim",
        detail: "Fires the detached `lernie prompt` a `/prepare` made ready, with this goal as \
                 its whole tail, verbatim. Refuses when nothing is prepared. At a seat with no \
                 composer to hold it — a terminal, where each `yog gesture` is its own process — \
                 hand the `/prepare` reply's own `prepared` object back with `yog gesture \
                 --prepared '<that object>' '/prompt <goal…>'`. The conversation keeps running \
                 whatever yog does.",
    },
    HelpRow {
        verb: "fork",
        usage: "/fork --from <ref> --role <role> [--skills a,b] --goal <the goal…>",
        summary: "try this conversation again from a point in its history",
        detail: "Forks the selected conversation from `--from` — a commit of its own history \
                 (the mark you pinned) or a `config/<name>` head for a clean start — and gives \
                 the fork this goal (`lernie dispatch`). `--role` is what names the model: \
                 lernie reads the provider and model id from the role's entry in the config \
                 governing that ref, so a role the config does not declare is refused there. \
                 `--skills` pins each named skill's instructions into the fork's context. To \
                 compare candidates, fire this more than once from the same mark: they group \
                 themselves under that mark's rule in the chat, because siblings of one mark is a fact about \
                 the refs and not a list anything keeps. Everything after `--goal` is the goal, \
                 verbatim. Takes the workspace and the conversation from the seat.",
    },
    HelpRow {
        verb: "delete-workspace",
        usage: "/delete-workspace <typed name>",
        summary: "unmake the focused workspace; the typed name is the arming",
        detail: "Unmakes the focused workspace and releases the balls it held. Fail-closed at \
                 fire time wherever it is asked: refused unless the workspace is yog's own, \
                 nothing in it is live, and the name you type matches it exactly.",
    },
    HelpRow {
        verb: "delete-agent",
        usage: "/delete-agent [typed name]",
        summary: "delete the selected conversation; the typed name arms taking its children too",
        detail: "Removes the selected conversation and everything lernie holds for it — its ref, \
                 worktree, steps and inbox. Refused while it is live. A bare line deletes the one \
                 conversation; typing its name is what arms taking its descendants with it, and \
                 without that lernie declines a subtree nobody confirmed.",
    },
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
                 not there yet answers empty text; a lineage destination refuses, since browsing \
                 one stays the config pane's own read.",
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
