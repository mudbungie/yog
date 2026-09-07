//! The reads whose subject is **a workspace, or the world above it** (§8.5) —
//! split from [`queries`](super::queries) at §12's budget (bl-c088) on the seam
//! of what a read is aimed at: these eleven answer what exists, what each
//! workspace holds, what its agents changed and which providers, machines and
//! lineages it resolves, while everything left in
//! [`QUERIES`](super::queries::QUERIES) is aimed at one conversation or names
//! nothing at all.
//!
//! It is a prefix of the roster, not a second list:
//! [`table`](crate::boundary::help::table) joins it back ahead of `QUERIES`, so
//! an operator meets one list in one order.

use crate::boundary::help::{HelpRow, Surface};

/// The enumeration, its bindings and rollups, the two project-side derivations,
/// the V4 board, and the §9 tables a workspace resolves.
pub const WORLD: &[HelpRow] = &[
    HelpRow {
        verb: "workspaces",
        usage: "/workspaces",
        summary: "the workspaces with their attention rollups",
        detail: "Every workspace yog can see, classified, with the attention count, the agent \
                 count and whether anything in it is running.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "conversations",
        usage: "/conversations",
        summary: "the focused workspace's conversation rows",
        detail: "One row per root conversation of the focused workspace, subtree-aggregated and \
                 ordered attention > running > recency — the same rows the window's list paints.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "balls",
        usage: "/balls",
        summary: "every ball⇄workspace binding fact",
        detail: "The join rows: which ball is claimed by which workspace, in which state. The \
                 same derivation the roster's balls section renders.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "workspace-balls",
        usage: "/workspace-balls",
        summary: "the balls the focused workspace holds, with what each has cost",
        detail: "Every ball bound to the focused workspace: its id, its \u{a7}3.5 badge, the \
                 project its `bl` verbs run in, the workspace name they stamp `--as`, and the \
                 tokens and money its conversations have spent on it. `/balls` answers the \
                 whole world's binding table; this answers one workspace, which is what the \
                 window's balls section paints.",
        surface: Surface::Control,
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
        surface: Surface::Control,
    },
    HelpRow {
        verb: "science",
        usage: "/science",
        summary: "every delivery attempt of this workspace, with what it cost and how it ended",
        detail: "One row per attempt — the ordinary claim and each fan candidate alike: the goal \
                 it was fired with, the instruction documents frozen onto its dispatch commit, \
                 the config commit it is governed by (which is where its model and skills are \
                 named), the two refs of its project diff with both commits, the commit those \
                 two ends departed from, the delivery commit \
                 when its target's history records one, its tokens and wall seconds, what it \
                 last said, every message delivered into it, and its outcome: accepted when the \
                 target records its own delivery, rejected when a sibling's landed instead or it \
                 was discarded, reworked when it has since incorporated the target and could \
                 deliver again, pending when none of that has happened. Everything is derived \
                 when you ask — nothing here is stored, so the same row a minute later is a \
                 statement about the world a minute later.",
        surface: Surface::Control,
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
        surface: Surface::Control,
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
        surface: Surface::Control,
    },
    HelpRow {
        verb: "roles",
        usage: "/roles",
        summary: "what this workspace's roles are actually set to, and how they are tuned",
        detail: "One row per role this workspace's config declares: the provider row and model \
                 id bound to it, the effort level it asks for, and whether it asks for the \
                 priority lane. This is what `/model`, `/effort` and `/priority` have set — \
                 read back from the same place they write it, so a control can open showing \
                 what is in force instead of blank. It is the other half of `/providers`, and \
                 the two do not overlap: that one is per provider row and says what a row is \
                 capable of, this one is per role and says what has been chosen. Under \
                 follow-the-tip these are the settings every conversation here resolves at its \
                 next step, not just the next one started. A workspace whose config declares no \
                 role answers an empty list rather than refusing — nothing set is a state a \
                 fresh workspace is really in. Scoped to the seat's workspace (`--ws`, or the \
                 focused one), exactly as `/providers` is.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "doctor",
        usage: "/doctor",
        summary: "is this box wired up: every check a first conversation needs, and the act \
                  that fixes each",
        detail: "One row per check, each carrying the fact it read and — when it fails — the \
                 act that fixes it. The engine's own four always answer, because the box this \
                 gesture is for may hold no workspace at all: the wire material, the endpoint \
                 `wire/address` names (a `127.0.0.1:0` is a request only the listener learns \
                 the answer to, and no seat can dial it), what this process actually BOUND — \
                 which the file cannot say and the boot says once on stderr — and the git \
                 identity every workspace and every step is committed under. With a workspace \
                 selected it adds that workspace's two: whether a conversation started there \
                 would reach a model, asked by the same gate the fire itself passes and quoting \
                 the sentence it would refuse with, and who is registered to reach it. It reads \
                 and never writes: a diagnostic has to work on a box that is broken in the way \
                 it is diagnosing, so every remedy is a sentence naming the operator's own act. \
                 No tally rides with the rows — count them.",
        surface: Surface::Control,
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
                 its set and stands whether or not it is connected. **And when it last \
                 spoke**, which is the fact that tells a machine asleep right now from one that \
                 has never once connected: presence reads false for both, and on a terminal \
                 seat it reads false for everything, since every verb opens and closes its own \
                 connection. A row with no last-seen at all has never dialled — the enrolment \
                 was minted and abandoned, and it is the row an operator can safely remove. A \
                 machine is registered by an operator's own act on the server, never over the \
                 wire.",
        surface: Surface::Control,
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
        surface: Surface::Control,
    },
    HelpRow {
        verb: "proposals",
        usage: "/proposals [<proposal-id>]",
        summary: "what a reviewer has staged for this workspace's config, and how big it is",
        detail: "A reviewer agent that learned something writes it as a real config patch and \
                 parks it on a branch of its own, waiting for you. This lists them: which \
                 reviewer staged each one, which lineage it would move, how many lines it \
                 changes, and what the reviewer called it. `fresh` means the lineage still \
                 stands where the reviewer read it, so the patch can be taken as it is; `stale` \
                 means somebody advanced that lineage since, and the answer is to reject it and \
                 let the next checkpoint re-derive from where things now stand — a stale patch \
                 was written against a config that no longer governs anything. Name one and you \
                 get it whole, message and diff, beside the listing. Nothing is staged is an \
                 empty answer, not a refusal.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: "proposal",
        usage: "/proposal <proposal-id> <accept|reject>",
        summary: "take a reviewer's staged config patch, or throw it away",
        detail: "The decision the whole learning loop is built around: read what an agent \
                 concluded, then say yes or no. Accepting fast-forwards the patch's lineage onto \
                 it and deletes the staging branch, and every conversation on that lineage picks \
                 the change up at its next step with no act per conversation. Rejecting deletes \
                 the staging branch and nothing else — the reviewer's own conversation survives \
                 as the record of why it thought so. Both words are required: naming no \
                 proposal would mean something different the day a second one is staged, and a \
                 verdict that defaulted would make throwing work away the easy half. An accept \
                 whose lineage moved under it refuses and names where that lineage now stands, \
                 rather than merging somebody's memory forward.",
        surface: Surface::Control,
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
        surface: Surface::Control,
    },
];
