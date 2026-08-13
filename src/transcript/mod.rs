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
//! blocks as legacy lernie committed it, or an API-shaped object wrapping them
//! in `content` — with the provider's token `usage` report as `content`'s
//! sibling when one was reported (lernie ≥0.0.4).
//! One function answers where the blocks live (`parse::block_array`)
//! — two answers left every real `NNN-tool.json` in the Raw bucket (bl-47ec).
//! | other / unparseable name / unparseable bytes | — | Raw bucket (never dropped) |
//!
//! Everything is a pure function of the on-disk bytes (§3.5 stateless
//! re-read): no field caches a fact the files already carry. "Tool in
//! progress" is a *query* over the entries (a committed `tool_use` with no
//! committed `tool_result`), never a stored flag (PRINCIPLES: single source
//! of truth).
//!
//! **[`build`] reads only what is committed.** The live streaming tail is
//! [`Transcript::with_live`], a virtual trailing entry the *caller* folds on
//! from the rendered snapshot's [`Stream`](crate::git_tree::Stream) — because
//! the two have different clocks (§7.2): the committed read is memoized per
//! published snapshot, and the tail moves at frame cadence. Merging them into
//! one build made the tail as slow as the derivation, which is the defect
//! bl-54f7 closed.

use std::path::{Path, PathBuf};

mod parse;
mod render;
mod rows;
mod spine;
use parse::{parse_model, parse_tool_result};
pub use render::{Reading, render};
pub(crate) use rows::key;
pub use rows::{AutoExpand, Fold, Row, RowClass, Tone, rows};

/// Directory under the workspace holding the per-agent worktrees (ARCH §2.3).
const AGENTS_DIR: &str = "agents";
/// The committed-transcript directory inside an agent's worktree.
const MESSAGES_DIR: &str = "messages";
/// The one reserved `.json` origin token: a `tool_result` payload.
const TOOL_ORIGIN: &str = "tool";
const MD_EXT: &str = "md";
const JSON_EXT: &str = "json";
/// Synthetic filename for the virtual live-streaming entry.
const STREAMING_NAME: &str = "«live»";

/// A parsed agent transcript: the ordered `messages/` entries, plus — when
/// the latest step is in flight — the live streaming tail as a virtual
/// trailing entry.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Transcript {
    pub entries: Vec<Entry>,
}

/// One transcript row. `raw` is the verbatim backing bytes surfaced by the
/// Raw toggle for *any* entry (§11 "every tab has a Raw toggle showing
/// verbatim bytes"); `kind` is the parsed projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Source filename (`003-claude-opus.json`), or [`STREAMING_NAME`] for
    /// the virtual streaming entry.
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

    /// This transcript with the live tail appended as a virtual trailing entry
    /// (§7.2). `stream` is the rendered snapshot's own fold — never a disk read
    /// from here, which is what lets the caller run this per frame while the
    /// committed half stays memoized per published snapshot.
    ///
    /// A stream that has said nothing yet adds no entry: an empty live row is
    /// not the same claim as a model that has begun, and "waiting for the API"
    /// is the §11 live mark's to say, not a blank line's.
    #[must_use]
    pub fn with_live(&self, stream: &crate::git_tree::Stream) -> Transcript {
        let (thinking, text) = (
            stream.thinking.clone().unwrap_or_default(),
            stream.text.clone().unwrap_or_default(),
        );
        let mut entries = self.entries.clone();
        if !thinking.is_empty() || !text.is_empty() {
            entries.push(Entry {
                name: STREAMING_NAME.to_string(),
                raw: format!("{thinking}{text}").into_bytes(),
                kind: EntryKind::Streaming { thinking, text },
            });
        }
        Transcript { entries }
    }
}

/// Build the **committed** transcript for `agent_id` in `workspace`: the
/// `messages/` directory and nothing else. The live tail is
/// [`Transcript::with_live`] — see the module doc for why the two are not one
/// call.
pub fn build(workspace: &Path, agent_id: &str) -> Transcript {
    let dir = workspace.join(AGENTS_DIR).join(agent_id).join(MESSAGES_DIR);
    Transcript {
        entries: read_messages(&dir),
    }
}

/// Enumerate `messages/` as entries in filename order. The zero-padded `NNN`
/// counter makes lexicographic filename order the true message order, so a
/// plain string sort suffices. Non-files (a stray subdir) are skipped; an
/// absent directory yields no entries.
fn read_messages(dir: &Path) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<(String, PathBuf)> = read
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            path.is_file()
                .then(|| (e.file_name().to_string_lossy().into_owned(), path))
        })
        .collect();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
        .into_iter()
        .map(|(name, path)| {
            let raw = std::fs::read(&path).unwrap_or_default();
            let kind = classify(&name, &raw);
            Entry { name, raw, kind }
        })
        .collect()
}

/// Classify one entry by its filename origin token and bytes.
///
/// A delivered `.md` is a **deposit file moved verbatim** — lernie's
/// `deliver_message` is a literal `rename(2)` and "the file's frontmatter
/// travels untouched" (ARCH §2.11) — so its bytes open with the
/// `---\nfrom: …\n---\n` envelope, not with the message. It is parsed by
/// the one envelope parser yog has ([`crate::inboxview::parse_deposit`]);
/// a second copy here would be a second truth about the same bytes. The
/// envelope's asserted fields are dropped from the parsed view exactly as
/// the model-id line and the `tool_use_id`s are (DESIGN §11) — the framing
/// `sender` is the filename's, and the Raw toggle still shows the envelope
/// verbatim. **`epitaph:` is the one exception, and it is the rule's own
/// reason**: the dropped fields are re-asserted elsewhere (the sender by the
/// filename, the timestamp by the file order), but nothing else carries the
/// ending, and on a body-less result deposit it is the whole message (bl-71e8).
fn classify(name: &str, raw: &[u8]) -> EntryKind {
    let Some((origin, ext)) = parse_name(name) else {
        return EntryKind::Raw;
    };
    match ext {
        MD_EXT => {
            let deposit = crate::inboxview::parse_deposit(raw);
            EntryKind::Delivered {
                sender: origin.to_string(),
                epitaph: deposit.epitaph,
                body: deposit.body,
            }
        }
        JSON_EXT if origin == TOOL_ORIGIN => parse_tool_result(raw).unwrap_or(EntryKind::Raw),
        JSON_EXT => parse_model(origin, raw),
        _ => EntryKind::Raw,
    }
}

/// Split `NNN-<origin>.<ext>` into `(origin, ext)`. The `NNN` is validated
/// (leading digit run) but not returned — filename order already carries it.
/// Anything not matching the shape is `None` (→ Raw bucket).
fn parse_name(name: &str) -> Option<(&str, &str)> {
    let (stem, ext) = name.rsplit_once('.')?;
    let (num, origin) = stem.split_once('-')?;
    if num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) || origin.is_empty() {
        return None;
    }
    Some((origin, ext))
}

#[cfg(test)]
mod tests;
