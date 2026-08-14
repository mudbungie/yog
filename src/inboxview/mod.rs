//! Inbox deposit view-model (DESIGN §11 Inbox tab; ARCH §2.11 deposit).
//!
//! An agent's inbox is `<workspace>/inbox/<agent-id>/<sender>-<NNN>.md`
//! (§2.11): the path carries framing (the sender), the frontmatter carries
//! asserted facts (`from:` / `deposited_at:`, plus `epitaph:` /
//! `terminal_ref:` on a result message, §2.6), and the body is the content
//! verbatim. Yog is a pure reader (§3.5): this module parses one deposit
//! file and enumerates a listing, both pure over injected paths, deriving
//! nothing it can read.
//!
//! Parsing is **forgiving** (brazen's forgiving-read stance): a file
//! without a well-formed `---` frontmatter block renders as a raw body with
//! every field absent, so a half-written or hand-edited deposit never
//! becomes an error.
//!
//! [`parse_deposit`] is the **one** reader of that envelope in yog, and its
//! reach is wider than this tab: delivery moves the deposit file into the
//! transcript by `rename(2)` with "the frontmatter travelling untouched"
//! (§2.11), so `messages/NNN-<sender>.md` is these very bytes and
//! [`crate::transcript`] parses them here rather than keeping a second
//! truth about one format.

use crate::nav::convs::Titles;
use std::path::Path;

mod render;
pub(crate) mod wire;
pub use render::render;

/// Workspace subdir holding per-agent inboxes (ARCH §2.11).
const INBOX_DIR: &str = "inbox";
/// Extension of a deposited message file — the atomic-rename temp files
/// are `.<name>.tmp` dotfiles, excluded by this suffix.
const MESSAGE_EXT: &str = ".md";

/// The pinned manner of an agent's ending (ARCH §2.6), from a result
/// message's `epitaph:` frontmatter. `Unknown` preserves a forward-compat
/// value verbatim (the `v=1` tolerate-unknown stance, §3.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Epitaph {
    FinalResponse,
    Stopped,
    BudgetExhausted,
    Died,
    Unknown(String),
}

impl Epitaph {
    /// The on-disk `epitaph:` value → typed variant (ARCH §2.6 hyphenated
    /// spellings); anything else rides through as [`Epitaph::Unknown`].
    ///
    /// `pub(crate)` since bl-7067: it is also [`label`](Self::label)'s inverse,
    /// which is what reads the ending back off the wire. One table both ways,
    /// rather than a second one beside the codec.
    pub(crate) fn parse(value: &str) -> Epitaph {
        match value {
            "final-response" => Epitaph::FinalResponse,
            "stopped" => Epitaph::Stopped,
            "budget-exhausted" => Epitaph::BudgetExhausted,
            "died" => Epitaph::Died,
            other => Epitaph::Unknown(other.to_string()),
        }
    }

    /// The on-screen wording of an ending (§2.6) — **the one** such mapping,
    /// read by the Inbox tab and by the transcript's result-deposit row alike
    /// (§11: one word for one fact, wherever it surfaces). An unknown
    /// forward-compat value rides through verbatim.
    pub fn label(&self) -> String {
        match self {
            Epitaph::FinalResponse => "final-response".to_string(),
            Epitaph::Stopped => "stopped".to_string(),
            Epitaph::BudgetExhausted => "budget-exhausted".to_string(),
            Epitaph::Died => "died".to_string(),
            Epitaph::Unknown(value) => value.clone(),
        }
    }
}

/// One parsed inbox deposit (ARCH §2.11). Frontmatter fields are `Option`
/// — a malformed file yields all-`None` with the whole content as
/// [`body`](Deposit::body). `epitaph` / `terminal_ref` are present only on
/// a result message (§2.6); an empty `body` is a result whose agent never
/// spoke.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Deposit {
    pub sender: Option<String>,
    pub deposited_at: Option<String>,
    pub epitaph: Option<Epitaph>,
    pub terminal_ref: Option<String>,
    pub body: String,
}

/// Parse one deposit file's bytes into a [`Deposit`]. Forgiving: content
/// without a leading `---\n … \n---\n` frontmatter block is taken as a raw
/// body with every field absent.
pub fn parse_deposit(bytes: &[u8]) -> Deposit {
    let text = String::from_utf8_lossy(bytes);
    match split_frontmatter(&text) {
        Some((frontmatter, body)) => Deposit {
            sender: field(frontmatter, "from"),
            deposited_at: field(frontmatter, "deposited_at"),
            epitaph: field(frontmatter, "epitaph").map(|v| Epitaph::parse(&v)),
            terminal_ref: field(frontmatter, "terminal_ref"),
            body: body.to_string(),
        },
        None => Deposit {
            body: text.into_owned(),
            ..Deposit::default()
        },
    }
}

/// One listing entry: a deposit file's name and its **verbatim backing
/// bytes** beside the parse of them. The transcript entry's shape, for the
/// same reason (§11: the Raw toggle shows the file, the parsed view shows
/// what yog made of it) — the envelope this tab's parsed view drops is still
/// in `raw`, so nothing the file said is unreachable.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InboxEntry {
    /// Deposit filename (`user-001.md`) — the Raw view's header.
    pub name: String,
    /// The file's bytes, unaltered.
    pub raw: Vec<u8>,
    pub deposit: Deposit,
}

impl InboxEntry {
    /// Whether this deposit exists **only in memory** — §7.2's pending echo,
    /// the message yog has sent and the driver has not yet written. A listed
    /// deposit *is* a file, so it always has a name; an empty one cannot come
    /// from [`list_inbox`], which makes this a query rather than a flag. Its
    /// one consumer is the §11 tone — faded while a send is only yog's word for
    /// it, brightening when the derivation makes it a statement.
    pub fn in_memory(&self) -> bool {
        self.name.is_empty()
    }
}

/// The one-line `✉ from · at` header of a deposit — the wording every seat of
/// the §5.1 #11 derivation shares (the Inbox tab's listing and the
/// inbox-composer's pending rows, bl-929d), so "pending mail" reads identically
/// across altitudes. Absent fields read as `?` rather than vanishing, keeping a
/// hand-edited deposit legible.
///
/// **The sender is an agent, so it wears the §3.3 ladder — from rung one**
/// (bl-b6d0, ruling in the ball): the ladder's answer over the frame's own
/// roster — the same one function the conversation list's title, the centre
/// header, the composer's target line and the transcript's speaker read. Since
/// bl-1eb0 the roster it reads is [`Titles`], the id→title table, rather than
/// the engine's agent set: a name a seat paints must be one the wire can carry
/// it (REMOTE §9.4). This
/// seat called [`id_floor`](crate::nav::convs::id_floor) *directly* until then
/// — the ladder's FLOOR as if it were the ladder — so a deposit from a named
/// peer painted its raw id in a frame where four other seats painted its name.
/// bl-63a1's floor spelling (the terminal generation only, never the whole
/// ancestry chain) is unchanged and still reached: it is where the ladder lands
/// for a sender no agent here carries — `user`, the operator's own deposits,
/// spelled whole because a stampless token is its own terminal segment, and a
/// foreign or deleted id alike. No branch, and nothing is stored on the deposit:
/// the `from:` fact stays the file's, and the name is derived where it is
/// painted, so the row and the roster cannot disagree within one frame.
pub fn header_line(deposit: &Deposit, titles: &Titles) -> String {
    let from = deposit
        .sender
        .as_deref()
        .map_or_else(|| "?".to_owned(), |id| titles.name(id));
    let at = deposit.deposited_at.as_deref().unwrap_or("?");
    format!("✉ {from} · {at}")
}

/// Enumerate an agent's inbox (`<workspace>/inbox/<agent-id>/*.md`),
/// oldest-first by filename (the sender-namespaced `<sender>-<NNN>.md`
/// sequence, ARCH §2.11). Atomic-rename temp dotfiles and non-`.md`
/// entries are excluded; an unreadable entry is skipped; a missing inbox
/// is an empty listing. Pure over the injected workspace path.
pub fn list_inbox(workspace: &Path, agent_id: &str) -> Vec<InboxEntry> {
    let dir = workspace.join(INBOX_DIR).join(agent_id);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(MESSAGE_EXT))
        })
        .collect();
    files.sort();
    files
        .iter()
        .filter_map(|p| {
            let raw = std::fs::read(p).ok()?;
            Some(InboxEntry {
                name: p.file_name()?.to_string_lossy().into_owned(),
                deposit: parse_deposit(&raw),
                raw,
            })
        })
        .collect()
}

/// Split `---\n<frontmatter>\n---\n<body>` into `(frontmatter, body)`, or
/// `None` when the leading delimiter or the closing `\n---\n` is absent.
/// The first `\n---\n` closes the block (frontmatter precedes any body), so
/// a body that itself contains the delimiter is safe.
fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    text.strip_prefix("---\n")?.split_once("\n---\n")
}

/// The value of frontmatter line `<key>: <value>`, or `None`. A line with
/// no `": "` separator, or a different key, is skipped.
fn field(frontmatter: &str, key: &str) -> Option<String> {
    frontmatter
        .lines()
        .filter_map(|line| line.split_once(": "))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.to_string())
}

#[cfg(test)]
mod tests;
