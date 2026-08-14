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
        verb: "lineages",
        usage: "/lineages",
        summary: "this workspace's config lineages, and the files each one holds",
        detail: "The policy branches of the seat's workspace — the lineages a conversation is \
                 born on — each with its tip commit and every file that commit holds. It is the \
                 listing `/config branch <lineage> <path>` then reads a file out of, and the same \
                 two dropdowns the config pane fills: pick a lineage, pick a path, read the \
                 bytes, then send an edit back with text after those same words. A workspace \
                 whose repository cannot be read is said outright rather than shown as no \
                 lineages at all.",
    },
    HelpRow {
        verb: "models",
        usage: "/models <provider>",
        summary: "the model ids one provider is offering right now",
        detail: "Asks the provider row what models it serves (`bz --list-models`) and answers \
                 the ids it listed, in its order — the same roster the model picker fills when \
                 you open it. Nothing is cached: the list belongs to the provider and can change \
                 without yog. Name the row, as `/providers` lists it; the sign-in it is listed \
                 against is the seat's workspace, so a row signed in elsewhere refuses here. Its \
                 answer is what `/model <role> <provider> <model-id>` then assigns — this says \
                 what exists, that says what a role uses. A provider that offers nothing, or \
                 cannot be asked, refuses with its own words rather than an empty list.",
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
