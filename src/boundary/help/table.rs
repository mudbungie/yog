//! The verb table's first half (§8.5): the mutating gestures — every command,
//! how it is typed, and what it does.
//!
//! **A detail says three things**: what the gesture does, what it takes from
//! the seat when the line does not say it, and how it refuses. That is what an
//! operator needs before pressing return on something that mutates a substrate.

use super::HelpRow;

/// The six §8.2 verbs whose subject is a conversation already running — split
/// off at §12's budget (bl-c088), and joined back *ahead* of [`ACTIONS`].
pub mod driving;
pub mod following;
pub mod queries;
pub mod standing;
/// The eleven reads aimed at a workspace or the world above it — split off at
/// §12's budget (bl-c088), and joined back *ahead* of [`queries::QUERIES`].
pub mod world;

/// Every **action on a conversation or a ball** that is not one of the six
/// [`driving::DRIVING`] states, in the order help lists them: the ball verbs,
/// the start pair, the V2 attempt, REMOTE §5's routing leg and the §3.6
/// deletes. The verbs whose subject is a setting, a standing
/// policy or a record are [`standing::STANDING`], the world's own reads are
/// [`world::WORLD`], the rest of the queries are [`queries::QUERIES`] and the follow-class reads are
/// [`following::FOLLOWING`]; [`table`](super::table) reads all four as one, so
/// no split is a seam an operator can see.
pub const ACTIONS: &[HelpRow] = &[
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
        verb: "create",
        usage: crate::boundary::line::CREATE_USAGE,
        summary: "create a ball in the focused project",
        detail: "Creates a ball in the seat's project (`bl create`) and prints nothing but its \
                 id in the reply. The title is the words before any flag; `--body` carries the \
                 rest of the description. Both are whitespace-normalized — a line is a line. \
                 The four scheduling facts the board orders on ride here too: `--priority` (a \
                 number, higher first), `--tag` (repeatable), `--parent` and `--needs` (a \
                 blocker `ID[:OP]`, `OP` defaulting to claim). A subtask is `--parent E` plus \
                 `/update E --needs <new>:close`; balls judges them and its refusal rides back.",
    },
    HelpRow {
        verb: "update",
        usage: crate::boundary::line::UPDATE_USAGE,
        summary: "amend a ball's title, body, schedule, or append a journal note",
        detail: "Amends a ball (`bl update`). At least one field is required, or the line asked \
                 for nothing; give any combination. `--note` appends to the ball's journal \
                 rather than replacing anything. The scheduling facts each have a clearing \
                 form beside them — `--no-priority`, `--no-parent`, and `--no-tag T` / \
                 `--no-needs ID` which drop one named entry. Repeat `--tag` to add several. \
                 With no id, the focused ball.",
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
        usage: "/prompt [<goal…>]",
        summary: "fire the prepared start with this goal, verbatim",
        detail: "Fires the detached `litany prompt` a `/prepare` made ready, with this goal as \
                 the whole payload, verbatim — nothing is prepended to it. Say no goal and the \
                 `/prepare` reply's own prefill fires instead, whole: the ball rung's `Ball \
                 <id>: <title>` header and body, or the path rung's `Working directory: <dir>` \
                 preamble. A bare start prepares no prefill, so there the goal is required. To \
                 fire a prefill with words of your own, send the two joined as one goal — that \
                 text is the reply's `prepared.goal`, and editing it is exactly what a seat with \
                 a composer does. Refuses when nothing is prepared. At a seat with no composer \
                 to hold it — a terminal, where each `yog gesture` is its own process — \
                 hand the `/prepare` reply's own `prepared` object back with `yog gesture \
                 --prepared '<that object>' '/prompt [<goal…>]'`. The conversation keeps running \
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
                 the fork this goal (`litany dispatch`). `--role` is what names the model: \
                 litany reads the provider and model id from the role's entry in the config \
                 lineage governing that ref, at its head, so a role that lineage does not \
                 declare is refused there. \
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
                 `{\"stdout\": …, \"stderr\": …, \"exit_code\": …}` — litany's own tool \
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
        detail: "Removes the selected conversation and everything litany holds for it — its ref, \
                 worktree, steps and inbox. Refused while it is live. A bare line deletes the one \
                 conversation; typing its name is what arms taking its descendants with it, and \
                 without that litany declines a subtree nobody confirmed.",
    },
];
