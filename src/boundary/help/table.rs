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
        verb: "interrupt",
        usage: "/interrupt <text…>",
        summary: "cut the selected conversation off mid-work and send it this text",
        detail: "Stops whatever is running the selected conversation and then deposits the text \
                 (`lernie stop`, then `lernie message`), so the model reads it now instead of at \
                 the end of what it is doing. The deposit is what restarts the conversation — \
                 there is no separate resume — so this leaves it running on your new text. Work \
                 already committed is kept, and a tool call cut off mid-flight is reported to the \
                 model in band as having produced no result. With nothing running it is simply a \
                 send. Two lines on the trail, one for each half, because the stop can be \
                 declined while the text still lands. The text is the whole tail, verbatim; no \
                 flag is read out of it, `children` included — use `/stop children` for a \
                 subtree. Takes the workspace and the agent from the seat; refuses when nothing \
                 is selected. Ctrl+Enter in the composer is this gesture.",
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
        verb: "nudge",
        usage: "/nudge",
        summary: "prompt the selected conversation again from where it already stands",
        detail: "Runs the model on the selected conversation as it is, with nothing added \
                 (`lernie advance`): no new message, no goal retyped, the same conversation \
                 continued. This is the fix for a first turn that died before it reached the \
                 model — a missing sign-in, a provider row that was wrong — sign in, then nudge, \
                 and the turn is dispatched again in place. The driver runs detached, so it \
                 keeps going whatever yog does. Takes the workspace and the agent from the \
                 seat; refuses when nothing is selected, and does nothing while a driver is \
                 already running the conversation.",
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
                 whatever yog does. The receipt's `conversation` is the minted name, and a name \
                 is an address: hand it back at `--agent` to any conversation verb or read.",
    },
    HelpRow {
        verb: "fan",
        usage: "/fan <n>",
        summary: "spread the prepared start over n isolated candidate attempts",
        detail: "Asks balls for `n` private attempt worktrees off one pinned tip of the focused \
                 ball's delivery target, and answers with the prepared start once per candidate, \
                 each bound to its own worktree — fire them with `/prompt`, one per candidate, \
                 with whatever variation you want between them. `n` of 1 or 0 materializes \
                 nothing and hands back the ordinary claim binding, which is the same path with \
                 one candidate. Nothing records that the candidates belong together: they share \
                 a target and a base commit, and that is what makes them siblings.",
    },
    HelpRow {
        verb: "retire",
        usage: "/retire <handle>",
        summary: "release a candidate's worktree; keep its source ref unless retention says not to",
        detail: "Releases the named candidate's worktree. Its source ref — and so its whole diff \
                 — stays addressable, because a rejected candidate is one that was never \
                 delivered and yog deletes nothing on an opinion. The ref goes too only when \
                 `cadence.yaml`'s `retention:` block declares a `keep_min` for this project and \
                 the candidate has outlived it. Retiring changes no delivery target, ever.",
    },
    HelpRow {
        verb: "deliver",
        usage: "/deliver <handle> <summary…>",
        summary: "accept one candidate: the ordinary source-to-target delivery of its attempt",
        detail: "Delivers the named candidate onto the focused ball's own `work/<id>` ref — the \
                 same recursive delivery `bl close` later performs one level up, so accepting \
                 neither closes the ball nor changes what its close delivers. The summary is \
                 the whole tail, verbatim: it becomes the delivery subject, which balls tags \
                 with the handle — the only acceptance mark there is, derived from the target's \
                 history rather than stored. A stale candidate refuses before anything merges: \
                 message its agent to incorporate the current target in its own worktree and \
                 deliver again. Rejection has no verb — a loser is simply never delivered.",
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
        verb: "advertise",
        usage: "/advertise <json array>",
        summary: "present this machine's tool set into the workspaces it is registered in",
        detail: "A tool host says what it can do. The whole tail is the set, as a JSON array \
                 whose elements are `{\"name\": …, \"description\": …, \"input_schema\": …}` — \
                 the name a single path component, the description one string, the schema a \
                 JSON Schema carried through untouched. The set replaces whatever this client \
                 advertised before, and it is stored only when it differs, so re-presenting an \
                 unchanged set on every reconnect writes nothing. Two elements sharing a name \
                 is refused outright; two machines both offering `Bash` is ordinary. It names \
                 no client: the identity it lands under is the certificate the connection \
                 presented, so a caller inside the world — the deposit inbox, `yog gesture`, \
                 the window — has none and is refused.",
    },
    HelpRow {
        verb: "invoke",
        usage: "/invoke <client> <tool> <json input>",
        summary: "route one tool call to the machine that advertised it",
        detail: "Queues one call in a tool host's mailbox and answers the handle it is known \
                 by. It does NOT wait: the engine's intake is one thread for the whole world, \
                 so what a tool takes is waited out by the caller, with `/capture <handle>` \
                 until an answer lands. The client must advertise that tool right now, or this \
                 refuses saying so — the same correction a machine that dropped a tool earns. \
                 The tool name is the one the host advertised, never the prefixed name a model \
                 sees. Whether the machine is connected is deliberately not checked: a tool \
                 host holds its connection only while it is waiting, so a busy one looks \
                 absent — what makes a vanished machine visible is your own deadline.",
    },
    HelpRow {
        verb: "complete",
        usage: "/complete <invocation> <json capture>",
        summary: "answer one routed invocation with what running it captured",
        detail: "The tool host's half of the routing leg. The capture is \
                 `{\"stdout\": …, \"stderr\": …, \"exit_code\": …}` — lernie's own tool \
                 contract, one for one. Only the machine the invocation was addressed to may \
                 answer it; a handle addressed to anyone else reads as absent, which is the \
                 same sentence a handle nobody minted earns. A caller inside the world has no \
                 certificate and therefore no invocations, and is refused.",
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
