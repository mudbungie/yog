//! Global search (DESIGN §8.5): **one derived query over the world's own
//! bytes**, and the §8.5 boundary's only asynchronous one.
//!
//! It is a query by the §4.8 taxonomy — it populates, it returns typed data
//! both frontends render, it has a headless spelling
//! ([`Query::Search`](crate::boundary::Query::Search)) — and like
//! [`help`](crate::boundary::help) it is set apart by its *subject*: the
//! published [`Snapshot`] names **where** every searchable thing is, and the
//! bytes are re-read at ask time. That is the whole no-index discipline (I1):
//! there is no second store to drift, so a match is a statement about the file
//! as it is right now, not as some index remembered it.
//!
//! **One hit per address, and the addresses are the existing ones.** A result
//! is something you can already select — a ball, a workspace, a conversation
//! ([`Address`]) — never a coordinate invented for search. A conversation whose
//! transcript matches forty times is one row, because one row is what you open.
//! That is also the bound: the hit count cannot exceed the world's subject
//! count, and [`MAX`] caps what is returned.
//!
//! **Ranking is a total order over facts, so it is deterministic**: the matched
//! [`Field`]'s tier first (an id beats a title beats a body), then the byte
//! offset of the match, then the address itself. No score, no tie to break by
//! chance.
//!
//! **Case folding is ASCII, deliberately.** A Unicode fold can change a
//! string's byte length, which would make the reported offset lie about the
//! bytes it points into — and the bytes are what is authoritative here.

use crate::app::Snapshot;
use std::path::PathBuf;

mod corpus;
mod excerpt;
#[cfg(test)]
mod tests;
mod worker;

pub use excerpt::excerpt;
pub use worker::{SearchThread, Searcher};

/// How many hits an answer carries at most (§8.5). A bound, not a knob: the
/// operator asks *what matches*, and how much of the world yog is willing to
/// hand back in one answer is yog's decision, spelled once here so the GUI and
/// the envelope cannot disagree about it.
pub const MAX: usize = 50;

/// Which field of a subject matched — and, by its order, the rank tier: what a
/// thing **is** beats what it is **for** beats what it **says**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Field {
    /// The subject's own identity: a ball id, a workspace name, a conversation
    /// name or agent id.
    Name,
    /// What it is for: a ball title, a conversation's goal.
    Summary,
    /// The bulk: a ball body, a transcript entry's bytes.
    Text,
}

impl Field {
    /// The field's headless token — the JSON spelling of the tier. Internal
    /// (rule 2): the encoding is [`reply`](crate::boundary::reply)'s job, and a
    /// consumer outside this crate reads the encoded string, never this.
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Summary => "summary",
            Self::Text => "text",
        }
    }
}

/// Where a hit lives: an address yog already selects by, and nothing else.
/// Ordered so the sort's last tie-break is a fact rather than an accident.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Address {
    /// A ball in a project (§3.5) — what every `bl` verb takes.
    Ball { project: PathBuf, id: String },
    /// An enumerated workspace (§3.1).
    Workspace { path: PathBuf },
    /// A conversation: a root or member agent in a workspace (§11).
    Conversation { workspace: PathBuf, agent: String },
}

impl Address {
    /// The address's headless token (internal, as [`Field::token`] is).
    pub(crate) fn token(&self) -> &'static str {
        match self {
            Self::Ball { .. } => "ball",
            Self::Workspace { .. } => "workspace",
            Self::Conversation { .. } => "conversation",
        }
    }
}

/// One result: where it is, which field matched, where in that field, and the
/// matched line as the operator reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub at: Address,
    pub field: Field,
    /// Byte offset of the match within the matched field's own text.
    pub offset: usize,
    pub excerpt: String,
}

/// A whole answer: the ranked hits, and — never silently — the sources that
/// could not be read. An unreadable corner of the world shrinks the corpus, it
/// does not make the world unsearchable, so both halves ride back together.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
    pub hits: Vec<Hit>,
    /// Each unreadable source, named with why — sorted, so two runs over one
    /// world say the same thing.
    pub unreadable: Vec<String>,
}

impl Found {
    /// Nothing to show — no hits and nothing that could not be read. It is the
    /// answer's own predicate rather than the seat's, because two seats now ask
    /// it: the §11 center strip, deciding whether to offer the Search tab at
    /// all, and the pane, deciding what to paint. Spelled twice it would drift.
    pub fn is_empty(&self) -> bool {
        self.hits.is_empty() && self.unreadable.is_empty()
    }
}

/// Run the query (§8.5). `wanted` is the asker's liveness: it is consulted
/// between conversations — where the disk work is — so a superseded search
/// abandons rather than finishing work nobody is waiting for. An abandoned run
/// returns what it had; the publisher discards it, because the seq it answers
/// is no longer the seq being asked.
///
/// An empty (or whitespace-only) query matches nothing. Not a special case: it
/// is the general path with no input, and every seat renders its own empty.
pub fn run(snap: &Snapshot, text: &str, wanted: &dyn Fn() -> bool) -> Found {
    let needle = text.trim().to_ascii_lowercase();
    let mut found = Found::default();
    if needle.is_empty() {
        return found;
    }
    found.unreadable = corpus::unreadable(snap);
    for (at, fields) in corpus::from_snapshot(snap) {
        push(&mut found.hits, &needle, at, &fields);
    }
    for (workspace, agent, mut fields) in corpus::conversations(snap) {
        if !wanted() {
            break;
        }
        fields.extend(corpus::read_conversation(
            &workspace,
            &agent,
            &mut found.unreadable,
        ));
        let at = Address::Conversation { workspace, agent };
        push(&mut found.hits, &needle, at, &fields);
    }
    found.unreadable.sort();
    found.unreadable.dedup();
    rank(&mut found.hits);
    found
}

/// Keep one hit per address: the subject's **best** field match, chosen by the
/// same order the ranking uses, so "best" means one thing in this module.
fn push(hits: &mut Vec<Hit>, needle: &str, at: Address, fields: &[(Field, String)]) {
    let best = fields
        .iter()
        .filter_map(|(field, text)| {
            let offset = text.to_ascii_lowercase().find(needle)?;
            Some((*field, offset, excerpt(text, offset)))
        })
        .min_by_key(|(field, offset, _)| (*field, *offset));
    if let Some((field, offset, excerpt)) = best {
        hits.push(Hit {
            at,
            field,
            offset,
            excerpt,
        });
    }
}

/// One hit as a row of text — what it is, where, and what matched. Lives here
/// rather than at a render site so the window, a TUI and a log all read a hit
/// the same way, and the shell stays glue.
pub fn label(hit: &Hit) -> String {
    let at = match &hit.at {
        Address::Ball { id, .. } => format!("ball {id}"),
        Address::Workspace { path } => format!("workspace {}", crate::start::leaf_name(path)),
        Address::Conversation { workspace, agent } => {
            format!("{}/{agent}", crate::start::leaf_name(workspace))
        }
    };
    format!("{at} — {}", hit.excerpt)
}

/// Sort by tier, then match position, then address — a total order over the
/// hit's own facts — and cut to [`MAX`].
fn rank(hits: &mut Vec<Hit>) {
    hits.sort_by(|a, b| {
        (a.field, a.offset)
            .cmp(&(b.field, b.offset))
            .then_with(|| a.at.cmp(&b.at))
    });
    hits.truncate(MAX);
}
