//! The verb table's first half (§8.5): the mutating gestures — every command,
//! how it is typed, and what it does.
//!
//! **A detail says three things**: what the gesture does, what it takes from
//! the seat when the line does not say it, and how it refuses. That is what an
//! operator needs before pressing return on something that mutates a substrate.

use super::HelpRow;

pub mod queries;
pub mod standing;

/// Every **action on a conversation or a ball**, in the order help lists them:
/// the §8.2 conversation verbs, the ball verbs, the start pair, the V2 attempt
/// and the §3.6 deletes. The verbs whose subject is a setting, a standing
/// policy or a record are [`standing::STANDING`] and the queries are
/// [`queries::QUERIES`]; [`table`](super::table) reads all three as one, so
/// neither split is a seam an operator can see.
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
        verb: "retarget",
        usage: "/retarget",
        summary: "move the selected conversation onto the config this workspace runs now",
        detail: "A conversation is frozen on the config commit it forked off, so a model you \
                 picked afterwards governs the next conversation and not this one. This moves \
                 this one (`lernie retarget`): it marks the conversation, and the conversation's \
                 own driver re-forks it onto the current config at its next step, replaying \
                 everything it has already done on top — nothing is discarded and nothing is \
                 killed. It takes effect at that next step, never mid-step, which in practice is \
                 the message you send after it. Takes the workspace and the conversation from the \
                 seat; lernie declines it when the conversation is already on that config, or \
                 when the target config does not describe the role it runs as.",
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
];
