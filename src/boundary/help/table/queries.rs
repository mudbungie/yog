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
        verb: "clients",
        usage: "/clients",
        summary: "the machines registered in this workspace, who is connected, and what they offer",
        detail: "One row per client registered in the seat's workspace: its name, whether it \
                 holds a live connection right now, and the tools it has advertised. Presence \
                 is read at the moment you ask and is true only then — a client that answers \
                 here may be gone a second later, which is why nothing durable records it. \
                 What each client advertises, by contrast, was written when it last presented \
                 its set and stands whether or not it is connected. A machine is registered by \
                 an operator's own act on the server, never over the wire.",
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
    },
    HelpRow {
        verb: "files",
        usage: "/files [<path>]",
        summary: "the conversation's working files, and one file's contents",
        detail: "The agent's own worktree read-only: goal, soul, summaries, skills and whatever \
                 work products it has written, as a sorted listing with each entry's size. Name a \
                 path exactly as the listing gives it to read that file's contents instead; only \
                 a file this same listing just named can be opened, and a large one comes back \
                 truncated with its true size. The listing is bounded, and says when it hit the \
                 bound. A conversation whose worktree has been torn down is said outright rather \
                 than shown as an empty directory.",
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
    },
    HelpRow {
        verb: "agent",
        usage: "/agent",
        summary: "the selected conversation itself: what it is called, what it is doing, what may \
                  be done to it",
        detail: "One conversation's own facts, as any seat paints them: the id and the \
                 conversation it belongs to, the name it goes by — and whether that name is one \
                 peers can actually address it with — the commit its policy is frozen against, \
                 whether it is running right now, the marks it wears (notified, over budget, in \
                 conflict, holding a tool call, abandoned), what kind of work is in flight \
                 anywhere beneath it, and whether Stop and its children cascade are offered. A \
                 conversation this workspace does not carry answers as its own root, stopped and \
                 unmarked, rather than refusing.",
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
        verb: "invocations",
        usage: "/invocations",
        summary: "a tool host's next work: wait for what this machine has been asked to run",
        detail: "The follow-class read. It does not answer until this machine has an invocation \
                 or the engine's hold expires — thirty seconds — so a tool host waits here \
                 rather than polling, and asks again the moment it is answered. It names \
                 nothing: the queue it drains is the one addressed to the certificate this \
                 connection presented, which is why a caller inside the world (`yog gesture`, \
                 the deposit inbox, the window) is refused rather than handed somebody's work. \
                 An empty answer is the ordinary answer of a hold that ended quietly, not a \
                 failure. Each row is `{\"invocation\": …, \"tool\": …, \"input\": …}`; run \
                 it and post the result with `/complete`.",
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
