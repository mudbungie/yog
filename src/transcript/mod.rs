//! Transcript view-model (DESIGN §5.1 #12, §11 Altitude-2 Transcript tab).
//!
//! An agent's conversation is a directory of message files at
//! `<workspace>/agents/<agent-id>/messages/`. Each file is
//! `NNN-<origin>.<ext>` — **order lives in the filename** (the `NNN`
//! counter), and **origin lives in the filename token**:
//!
//! | ext  | origin token | entry                                        |
//! |------|--------------|----------------------------------------------|
//! | `md` | `<sender>`   | delivered message, envelope stripped — bar its `epitaph:` |
//! | `json` | `tool`     | `tool_result` (content / is_error / tool_use_id) |
//! | `json` | `<model-id>` | model output — canonical content blocks    |
//!
//! Both `.json` origins carry the **same envelope**: a bare array of canonical
//! blocks as legacy litany committed it, or an API-shaped object wrapping them
//! in `content` — with the provider's token `usage` report as `content`'s
//! sibling when one was reported (lernie ≥0.0.4).
//! One function answers where the blocks live (`parse::block_array`)
//! — two answers left every real `NNN-tool.json` in the Raw bucket (bl-47ec).
//! | other / unparseable name / unparseable bytes | — | Raw bucket (never dropped) |
//!
//! **The directory is not append-only.** litany's compactor deletes message
//! files and squashes the span they lived in, so a hole in the `NNN` counter
//! is entries that were *removed* — [`compaction`] derives each one and seats
//! a virtual [`EntryKind::Compacted`] marker in it, carrying whatever
//! `summary/**` the compactor wrote in their place.
//!
//! Everything is a pure function of the on-disk bytes (§3.5 stateless
//! re-read): no field caches a fact the files already carry. "Tool in
//! progress" is a *query* over the entries (a committed `tool_use` with no
//! committed `tool_result`), never a stored flag (PRINCIPLES: single source
//! of truth).
//!
//! **[`build`] reads only what is committed.** The two *trailing* virtual
//! entries — the live streaming tail and the settled-failure notice — are
//! [`tail`]'s, folded on by the caller from what it already holds: the
//! rendered snapshot's [`Stream`](crate::git_tree::Stream), and the §7.3
//! [`Wound`](crate::steps_view::Wound) off a built steps view. Because the
//! halves have different clocks (§7.2): the committed read is memoized per
//! published snapshot, and the tail moves at frame cadence. Merging them into
//! one build made the tail as slow as the derivation, which is the defect
//! bl-54f7 closed.

mod compaction;
pub(crate) use compaction::seq_of;
mod key;
pub(crate) use key::key;
mod parse;
/// The committed record's disk read — its own file at §12's budget (bl-73e7).
mod read;
/// The two virtual **trailing** entries and the folds that seat them.
mod tail;
pub(crate) mod wire;
use parse::{parse_model, parse_tool_result};
pub use read::build;
/// One message file classified — [`build`]'s own reading of a single entry,
/// for the §7.3 orphaned-tail predicate (`steps_view::orphan`, bl-abba), which
/// needs the **tail** entry alone and must not pay a whole record to get it.
pub(crate) use read::classify;

/// Directory under the workspace holding the per-agent worktrees (ARCH §2.3).
const AGENTS_DIR: &str = "agents";
/// The committed-transcript directory inside an agent's worktree.
const MESSAGES_DIR: &str = "messages";
/// The one reserved `.json` origin token: a `tool_result` payload.
const TOOL_ORIGIN: &str = "tool";
const MD_EXT: &str = "md";
const JSON_EXT: &str = "json";
/// A parsed agent transcript: the ordered `messages/` entries, plus whichever
/// [`tail`] entry the conversation's moment has — the live streaming tail
/// while a call is in flight, the settled-failure notice once it has stopped
/// on a §7.3 wound.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Transcript {
    pub entries: Vec<Entry>,
}

/// One transcript row. `raw` is the verbatim backing bytes surfaced by the
/// Raw toggle for *any* entry (§11 "every tab has a Raw toggle showing
/// verbatim bytes"); `kind` is the parsed projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Source filename (`003-claude-opus.json`), or one of [`tail`]'s
    /// bracketed synthetic names for a virtual entry.
    pub name: String,
    /// Verbatim backing bytes (the file's contents; the folded text for the
    /// streaming entry).
    pub raw: Vec<u8>,
    pub kind: EntryKind,
}

/// The origin classification of a transcript entry (see the module table).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    /// `.md` — a delivered message; `sender` is the filename origin token
    /// and `body` is the deposit's content with its `---` frontmatter
    /// envelope stripped (see [`classify`]). `epitaph` is `Some` exactly on a
    /// **result deposit** — a child's terminal, which asserts how it ended and
    /// may say nothing else (ARCH §2.6).
    Delivered {
        sender: String,
        epitaph: Option<crate::inboxview::Epitaph>,
        body: String,
    },
    /// `NNN-<model>.json` — model output as canonical content blocks, with
    /// the provider's committed `usage` counters when the bytes carry them.
    Model {
        model_id: String,
        blocks: Vec<Block>,
        usage: Usage,
    },
    /// `NNN-tool.json` — a `tool_result`.
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
    /// The live streaming tail folded from the open `response.json` — the
    /// model's reasoning and its answer so far, held apart because they are two
    /// things being said and each becomes its own row, exactly as the
    /// [`Block::Thinking`]/[`Block::Text`] pair of the *committed* entry that
    /// supersedes them will (§7.2 the thinking ruling). Either may be empty:
    /// a model that has only thought so far, or one that answered without
    /// reasoning, and an empty half is simply no row.
    Streaming { thinking: String, text: String },
    /// The conversation's **settled failure**, in the §7.3 wound vocabulary —
    /// it stopped, nobody is driving it, and this is what happened to it last
    /// (bl-015b). Virtual and trailing, like [`Streaming`](Self::Streaming),
    /// and folded on by the same caller ([`Transcript::with_wound`]).
    ///
    /// It carries the [`Wound`](crate::steps_view::Wound) and not the sentence
    /// built from it, for [`Compacted`](Self::Compacted)'s reason: the words a
    /// seat paints are the wound's own projection, and a headless seat runs
    /// that same projection over this decoded entry.
    Wounded { wound: crate::steps_view::Wound },
    /// A span of entries litany's compactor **deleted** — a hole in the `NNN`
    /// counter, standing where they were. `first` and `last` are the missing
    /// counter values, inclusive, and are the only thing this entry asserts.
    /// `summary` is the conversation's whole compaction record, which rides
    /// the earliest gap and is empty on every other one *and* wherever the
    /// compactor left none — the pairing is unavailable on disk and is not
    /// guessed ([`compaction`]). Virtual: no file backs it, exactly as none
    /// backs [`Streaming`](Self::Streaming).
    Compacted {
        first: usize,
        last: usize,
        summary: String,
    },
    /// Unparseable filename or unparseable bytes — surfaced verbatim rather
    /// than dropped (§15 Y12: "surface them in a Raw bucket").
    Raw,
}

/// The committed `usage` record's token counters, verbatim from the bytes
/// (lernie ≥0.0.4 seals the provider's report beside `content`:
/// `{"content":[…],"usage":{"input_tokens":5,…}}`). Counter names are the
/// provider's own — no vocabulary is pinned here, so a counter brazen adds
/// rides through with no edit. Empty is the general path: a legacy bare-array
/// entry, or a provider that reported nothing (a `0` would be a lie).
pub type Usage = std::collections::BTreeMap<String, u64>;

/// One content block of a model message (§4.4 canonical blocks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Text(String),
    Thinking(String),
    /// A tool call: rendered as a chip with id / name / input summary.
    ToolUse {
        id: String,
        name: String,
        input_summary: String,
    },
}

impl Transcript {
    /// Is `tool_use_id` a committed `tool_use` still in progress — i.e. no
    /// committed `tool_result` anywhere in the sequence names it? Derived on
    /// demand, never stored (DESIGN §5.1 #12, §11). The id is **opaque**:
    /// `call_…` (OpenAI) and `toolu_…` (Anthropic) pair by byte equality and
    /// no shape is assumed.
    pub fn tool_in_progress(&self, tool_use_id: &str) -> bool {
        !self.entries.iter().any(|e| {
            matches!(&e.kind, EntryKind::ToolResult { tool_use_id: id, .. } if id == tool_use_id)
        })
    }
}

#[cfg(test)]
mod tests;
