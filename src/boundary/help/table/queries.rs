//! The **query** half of the §8.5 verb table: every populating read, how it is
//! typed and what it answers. Split from [`super`](super) at §12's 300-line cap
//! (bl-dc0c) along the §8.5 taxonomy's own line — actions mutate, queries
//! populate — and joined back by [`table`](crate::boundary::help::table), so
//! the operator still meets one roster and no seat reads a half of it.

use crate::boundary::help::HelpRow;

pub const QUERIES: &[HelpRow] = &[
    HelpRow {
        verb: "workspaces",
        usage: "/workspaces",
        summary: "the workspaces with their attention rollups",
        detail: "Every workspace yog can see, classified, with the attention count, the agent \
                 count and whether anything in it is running.",
    },
    HelpRow {
        verb: "conversations",
        usage: "/conversations",
        summary: "the focused workspace's conversation rows",
        detail: "One row per root conversation of the focused workspace, subtree-aggregated and \
                 ordered attention > running > recency — the same rows the window's list paints.",
    },
    HelpRow {
        verb: "balls",
        usage: "/balls",
        summary: "every ball⇄workspace binding fact",
        detail: "The join rows: which ball is claimed by which workspace, in which state. The \
                 same derivation the roster's balls section renders.",
    },
    HelpRow {
        verb: "work-diff",
        usage: "/work-diff [<ball> <path>]",
        summary: "what this workspace's agents changed in their project",
        detail: "The project-side answer to \"what did this agent do?\": for every ball the \
                 focused workspace holds, the changes on that ball's work branch that are not \
                 yet on the branch it delivers into — one row per changed file, with lines \
                 added and removed. Name a ball and a path to read that one file's patch \
                 instead. It is a plain git read of the project repository and changes nothing; \
                 a repository it cannot read, and a branch that is not there yet, are each said \
                 outright rather than shown as an empty list.",
    },
    HelpRow {
        verb: "board",
        usage: "/board",
        summary: "the fleet board — every live ball in its column",
        detail: "The balls section as columns: ready, gated, claimed, blocked. The three \
                 familiar rungs are derived exactly as `bl list` derives them; gated is the \
                 fourth, and it is balls' own close-blocker rule — a ball you could claim but \
                 could not deliver, shown with the ball whose close mints its gate. Each \
                 claimed row names the conversations working it, and carries its spend plus, \
                 for an epic, the rollup over its live subtree across every workspace.",
    },
    HelpRow {
        verb: "providers",
        usage: "/providers",
        summary: "one workspace's provider table, with each row's credential fact",
        detail: "brazen's provider rows (`bz --list-providers`) paired with the credential \
                 presence read: which rows are signed in, which need no credential, which have \
                 a key stored. The same rows the login pane's `↻` and the brazen config pane's \
                 read-only rows render — one derivation, every seat. Scoped to the seat's \
                 workspace (`--ws`, or the focused one): providers and their sign-ins belong to \
                 a workspace, so the same row can read signed-in in one and not in another.",
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
    },
    HelpRow {
        verb: "ops",
        usage: "/ops [n]",
        summary: "the last n ops rows, newest last",
        detail: "The tail of `ops.jsonl` — every gesture anything has fired, with its outcome. \
                 Defaults to the last 50 rows.",
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
    },
    HelpRow {
        verb: "help",
        usage: "/help [verb]",
        summary: "what any command does — this list, or one command's page",
        detail: "Help is itself a gesture, and a higher-order one: it is asked about a command. \
                 With no verb it is this roster; with one it is that command's page. The same \
                 answer comes from `<verb> --help`, from a bare `/`, and from `yog gesture \
                 --help` at a terminal — one question, one answer, every seat.",
    },
];
