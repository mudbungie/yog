//! The **query** half of the §8.5 verb table: every populating read, how it is
//! typed and what it answers. Split from [`super`](super) at §12's 300-line cap
//! (bl-dc0c) along the §8.5 taxonomy's own line — actions mutate, queries
//! populate — and joined back by [`table`](crate::boundary::help::table), so
//! the operator still meets one roster and no seat reads a half of it.
//!
//! Cut once more at the budget (bl-c088): the eleven reads aimed at a workspace
//! or the world above it are [`world::WORLD`](super::world::WORLD), joined back
//! immediately ahead of these. What is left here is aimed at **one
//! conversation**, or at nothing the seat has to name — the queue, the trail,
//! the search, the routing poll and help itself.

use crate::boundary::help::{HelpRow, Surface};

pub const QUERIES: &[HelpRow] = &[
    HelpRow {
        verb: "transcript",
        usage: "/transcript",
        summary: "the selected conversation, message by message",
        detail: "The chat itself: every committed message of the selected conversation — what was \
                 delivered to it, what the model said back (its reasoning, its answer and every \
                 tool it called), and what each tool returned — in order, each row carrying the \
                 file's own bytes beside the parse of them. A model call in flight right now is \
                 folded on as a live trailing row, so this says what the window says about the \
                 same moment. It is aimed at the seat's workspace and selected conversation; the \
                 whole thing is a read of files on disk and changes nothing.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "steps",
        usage: "/steps",
        summary: "every step this conversation has taken",
        detail: "One row per model call: how it ended (complete, failed, killed), how many \
                 attempts it took, what it spent, the branch tip it was assembled against, and \
                 when it started and ended. A step whose driver produced nothing at all is marked \
                 as such, with the adapter's own reason when it left one, and a step that failed \
                 on credentials names the provider row to log in to. Read one step's actual \
                 records with `/step <seq>`.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "step",
        usage: "/step <seq>",
        summary: "one step's records — request, response, staging, tools",
        detail: "The drill-in for a single step of the selected conversation, named by the \
                 sequence the list shows (`001`). Answers that step's `meta`, the wire request \
                 that was sent, the staged transcript entry, every event of the response stream, \
                 and every tool call's input and output — each as parsed data with the bytes it \
                 parsed from beside it. Records that are missing say so, and records that are not \
                 JSON come back verbatim and framed as unparseable rather than dropped.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "files",
        usage: "/files [<path>] [--at <commit>]",
        summary: "the conversation's working files, and one file's contents",
        detail: "The agent's own worktree read-only: goal, soul, summaries, skills and any work \
                 product written there, as a sorted listing with each entry's size. Name a \
                 path exactly as the listing gives it to read that file's contents instead; only \
                 a file this same listing just named can be opened, and a large one comes back \
                 truncated with its true size. The listing is bounded, and says when it hit the \
                 bound. A conversation whose worktree has been torn down is said outright rather \
                 than shown as an empty directory. A conversation bound to a work target — a \
                 path or ball start — runs every tool step there instead, so its deliverable is \
                 not in this listing at all: the answer then carries `working_dir`, the \
                 directory it is in, and its absence means this listing is where the work \
                 lands. `--at` reads a commit's tree instead of the \
                 worktree — the same agent-context-files-as-of the window's notch pin shows — and \
                 a commit this conversation never recorded simply holds no files.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "governing",
        usage: "/governing [--at <commit>]",
        summary: "the config commit this conversation resolves its policy from, and what it \
                  holds",
        detail: "A conversation forks off a commit of a `config/*` lineage, and the fork settles \
                 which lineage governs it — not which commit. Control then resolves that \
                 lineage's current head at every step boundary, so an edit you make now reaches \
                 this conversation at its next step, with nothing to press. This answers which \
                 commit it resolves — short and full — which lineage it follows, and every path \
                 that commit's tree holds: the souls, the workflow, the manifest, the provider \
                 table and the descriptions it is actually running under. When two or more \
                 lineages have diverged over its fork point there is no one head to follow, so \
                 it is held on that fork commit and the answer says how many reached it; \
                 `/retarget` is what settles that. `--at` starts the walk from a different \
                 commit and bare it is the conversation's own branch tip, so a seat need not \
                 know one to ask — but it names a rev, not a moment: policy as of an earlier \
                 step is no longer derivable from ancestry, and what records it is each step's \
                 own request. A workspace that cannot be read, and a commit that forks off no \
                 config lineage at all, are each said outright: a conversation with no policy is \
                 not a reading.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "rail",
        usage: "/rail",
        summary: "the conversation's spine: every operable commit and what hangs off it",
        detail: "One notch per step, each carrying the commit that step read against, its spend, \
                 and where in the chat its rule sits — the points a conversation can be forked or \
                 replayed from. Beside them, the children dispatched from this conversation: who \
                 each is, where it forked from, what it is doing, what it has spent and the last \
                 thing it said. A conversation nobody forked from answers notches and no children, \
                 which is the honest empty case rather than an error.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "inbox",
        usage: "/inbox",
        summary: "the mail deposited for this conversation but not yet delivered",
        detail: "Every message sitting in the selected conversation's inbox: who sent it, when it \
                 was deposited, the body, and — on a subagent's result message — how that agent \
                 ended and the commit it ended at. A half-written or hand-edited deposit is \
                 rendered rather than refused, with whatever fields it actually stated. Delivered \
                 mail is not here; it has moved into the transcript.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "agent",
        usage: "/agent",
        summary: "the selected conversation itself: what it is called, what it is doing, what may \
                  be done to it",
        detail: "One conversation's own facts, as any seat paints them: the id and the \
                 conversation it belongs to, the name it goes by — and whether that name is one \
                 peers can actually address it with — the commit its policy resolves from, \
                 whether it is running right now, the marks it wears (notified, over budget, in \
                 conflict, holding a tool call, abandoned), what kind of work is in flight \
                 anywhere beneath it, and whether Stop and its children cascade are offered. A \
                 conversation this workspace does not carry answers as its own root, stopped and \
                 unmarked, rather than refusing.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "attention",
        usage: "/attention",
        summary: "everything waiting on you, across every workspace",
        detail: "The decision queue: one row per conversation asking for you, anywhere yog can \
                 see — why it is asking (it notified you, it came to rest, it hit its budget, a \
                 transfer was declined, its mail is undelivered), what it last said, how long it \
                 has waited, and the workspace and conversation to aim an answer at. The order is \
                 the one the down arrow walks at the window; the count is the strip's. Answer a \
                 row with `/message`, `/stop` or `/seen`; hand one on by messaging somebody else \
                 about it.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "ops",
        usage: "/ops [n]",
        summary: "the last n ops rows, newest last",
        detail: "The tail of `ops.jsonl` — one line per action anything attempted, with its \
                 outcome. Actions only: a query reads the world and changes nothing, so it \
                 leaves no row, and what was read before an action was chosen cannot be \
                 recovered here. Defaults to the last 50 rows.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "search",
        usage: "/search [text…]",
        summary: "find the text anywhere: balls, workspaces, conversations, transcripts",
        detail: "One query across the whole world — ball ids, titles and bodies (closed ones \
                 too), workspace and conversation names, conversation goals, and every committed \
                 transcript entry. The text is the whole tail; matching is case-insensitive over \
                 ASCII. Nothing is indexed: the files are read as they are now, so a hit is a \
                 statement about the bytes on disk. One result per thing you can open — a \
                 transcript that matches forty times is one row — ranked by what matched (an id \
                 before a title before a body), and capped. Sources it could not read are named \
                 in the answer rather than silently dropped. At the window it is Ctrl+F, and \
                 results are clickable; a search with no text clears the last one.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "capture",
        usage: "/capture <invocation>",
        summary: "what one routed invocation captured, if the far machine has answered yet",
        detail: "The asking side's poll, and the other half of `/invoke`. It never waits: \
                 `capture` is absent while the machine is still running the tool, present once \
                 it has answered — and the answer is read exactly once, so the second ask reads \
                 as absent. How long to keep asking is yours to decide, and that deadline is \
                 what makes an offline machine a refusal you can see rather than a wait with no \
                 end. A handle you did not invoke reads as absent too.",
        surface: Surface::Machine,
    },
    HelpRow {
        verb: "help",
        usage: "/help [verb]",
        summary: "what any command does — this list, or one command's page",
        detail: "Help is itself a gesture, and a higher-order one: it is asked about a command. \
                 With no verb it is this roster; with one it is that command's page. The same \
                 answer comes from `<verb> --help`, from a bare `/`, and from `yog gesture \
                 --help` at a terminal — one question, one answer, every seat.",
        surface: Surface::Control,
    },
];
