//! The anchored block grammar yog reads and writes `providers.yaml` through:
//! its `roles:` block — the §9.4 picker's one write, and the single home of a
//! role's (provider row, model id) pointer. It read `models.yaml`'s `models:`
//! block too until bl-9c8a, when the one fact in it (the §5.1 #35 window)
//! moved to the step record brazen writes it on (see `rows.rs`).
//!
//! **This is not a YAML parser and must never become one.** yog declares no
//! YAML dependency (§9.2) and litany's own parser is private (its crate exposes
//! only `cmd`), so it recognizes exactly the block shape litany's
//! template authors — a top-level key at column 0, two-space entry keys,
//! four-space fields —
//!
//! ```text
//! roles:
//!   worker:
//!     provider: codex
//!     model: gpt-5.4
//! ```
//!
//! and **declines loudly** ([`GrammarError`], rendered in ichor per §7.3) on
//! anything else, pointing at the §9.2 / §9.3 raw editors. Declining is not a
//! dead end because those raw surfaces already exist: the picker is the fast
//! path over the shape litany itself writes, never a second authority on YAML.
//!
//! Every function here is pure text → text, so every arm is table-tested.
//! Rewrites preserve every byte outside the lines they target; the output is
//! normalized to `\n` line endings with a trailing newline.

/// Why the picker refused a config file (§7.3). Each variant names the file and
/// the shape that was expected, because the operator's next move is the raw
/// editor and they need to know what to fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrammarError {
    /// A top-level `<key>:` block key exists but carries an inline value
    /// (`models: {}`, flow style). Rewriting it would be a YAML transform.
    Inline { file: &'static str, key: String },
    /// No such two-space entry under the block — or the entry is flow-style,
    /// which this grammar deliberately does not recognize.
    NoEntry { file: &'static str, entry: String },
    /// The entry exists but lacks a four-space field the rewrite must move.
    NoField {
        file: &'static str,
        entry: String,
        field: &'static str,
    },
}

impl std::fmt::Display for GrammarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inline { file, key } => write!(
                f,
                "{file}: {key}: carries an inline value; yog edits only the \
                 block form litany writes — use the raw editor"
            ),
            Self::NoEntry { file, entry } => write!(
                f,
                "{file}: no `  {entry}:` block entry; yog edits only the block \
                 form litany writes — use the raw editor"
            ),
            Self::NoField { file, entry, field } => write!(
                f,
                "{file}: `  {entry}:` declares no `    {field}:` line; yog edits \
                 only the block form litany writes — use the raw editor"
            ),
        }
    }
}

mod entries;
mod fields;
mod roles;
mod rows;
mod tools;

pub use entries::{entry_names, remove_entry, set_entry};
pub use fields::{entry_field, remove_field, set_field, upsert_field};
pub use roles::{EFFORT, MODEL, PRIORITY, PROVIDER, roles, set_role_model};
pub use rows::is_unknown_row;
pub use tools::{flow_members, flow_value};

/// The file this grammar is spoken over, named in every refusal.
pub const PROVIDERS_YAML: &str = "providers.yaml";

/// The column-0 block key the file's entries hang under — the role map. Named
/// once here because three rewrites and the §9.5 pane all spell it.
pub const ROLES: &str = "roles";

/// One role's assignment as `providers.yaml` declares it (§5.1 #27) — **the
/// whole entry, not just the pointer** (bl-2410).
///
/// The two required halves are the model binding; the two optional ones are the
/// §9.4 tuning knobs `/effort` and `/priority` write. One type and one reader
/// ([`roles`]) for all four, because they are one entry: a second struct that
/// carried the knobs beside this one would be two readings of one line-block,
/// and the second would drift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleModel {
    pub role: String,
    pub provider: String,
    pub model: String,
    /// The role's `effort:` line **verbatim**, or `None` when it has none.
    ///
    /// A `String` and not the closed [`Effort`](crate::model_pick::Effort) the
    /// *gesture* takes, and the difference is the point: a gesture **asserts** a
    /// level out of a closed set, while this **reports** what the file holds —
    /// and yog does not own that file. The §9.1 raw editor is the operator's
    /// own authority, so `effort: extreme` is a thing this can be asked to
    /// describe. Normalizing it to `None` would say *nothing is set*, which is
    /// precisely the defect this read exists to end; carrying the word lets a
    /// control show that it does not recognize it. The same discipline
    /// [`is_unknown_row`](crate::model_pick::grammar::is_unknown_row) keeps: a
    /// question that went unanswered is never reported as a refusal.
    pub effort: Option<String>,
    /// Whether the role asks the provider's priority lane — `true` exactly when
    /// the line says `true`.
    ///
    /// A `bool` where `effort` is a string, because this is a **checkbox** and
    /// litany reads it as one: `false` and omitted are one fact upstream, so
    /// there is no third state for a word to name and nothing an unrecognized
    /// one could mean but *not asking*. Reporting `false` for anything but
    /// `true` is therefore not a normalization — it is the engine's own
    /// reading, said back.
    pub priority: bool,
}

/// Where a top-level block key sits, or why it cannot be used.
pub(super) enum BlockKey {
    /// `<key>:` alone on a column-0 line, at this index.
    At(usize),
    /// `<key>:` with an inline value — refused.
    Inline,
    /// No such key anywhere at column 0 — the caller may create it.
    Absent,
}

/// Locate `<key>:` at column 0. A key with anything after the colon is
/// [`BlockKey::Inline`]; absence is a value, not a fault (the caller decides
/// whether creating the block is lawful).
pub(super) fn block_key(lines: &[&str], key: &str) -> BlockKey {
    for (i, line) in lines.iter().enumerate() {
        let Some(rest) = line.strip_prefix(key) else {
            continue;
        };
        let Some(rest) = rest.strip_prefix(':') else {
            continue;
        };
        if rest.trim().is_empty() {
            return BlockKey::At(i);
        }
        return BlockKey::Inline;
    }
    BlockKey::Absent
}

/// Does this line end the block that started at column 0? Blank and comment
/// lines never do; a non-indented line always does.
fn ends_block(line: &str) -> bool {
    let trimmed = line.trim_start();
    !trimmed.is_empty() && !trimmed.starts_with('#') && !line.starts_with(' ')
}

/// The two-space entry keys directly under the block at `at`, as
/// `(name, line index)` in file order. A `  name: value` line is *not* an
/// entry — the flow form this grammar declines. Nothing here validates the
/// name: a nonsense key simply matches no role and no model id, so a guard
/// would only be a second way to say "not found".
pub(super) fn entries(lines: &[&str], at: usize) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (offset, line) in lines.iter().enumerate().skip(at + 1) {
        if ends_block(line) {
            break;
        }
        if let Some(rest) = line.strip_prefix("  ")
            && !rest.starts_with(' ')
            && let Some(name) = rest.strip_suffix(':')
        {
            out.push((name.to_string(), offset));
        }
    }
    out
}

/// One past the last line the block starting at `at` owns — the first line
/// [`ends_block`] calls the end, or the file's own end. The counterpart to
/// [`entries`] for a rewrite that removes or replaces a whole entry: the last
/// entry in a block runs to here.
pub(super) fn block_end(lines: &[&str], at: usize) -> usize {
    lines
        .iter()
        .enumerate()
        .skip(at + 1)
        .find(|(_, line)| ends_block(line))
        .map_or(lines.len(), |(i, _)| i)
}

/// A four-space `field: value` line within the entry starting at `at`, as
/// `(value, line index)`. Scanning stops at the next entry or the block's end.
pub(super) fn field(lines: &[&str], at: usize, name: &str) -> Option<(String, usize)> {
    for (offset, line) in lines.iter().enumerate().skip(at + 1) {
        if ends_block(line) {
            return None;
        }
        let Some(rest) = line.strip_prefix("    ") else {
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            return None;
        };
        if let Some(value) = rest.strip_prefix(name).and_then(|r| r.strip_prefix(':')) {
            return Some((value.trim().to_string(), offset));
        }
    }
    None
}

/// Re-join edited lines: `\n` endings, one trailing newline.
pub(super) fn join(lines: &[String]) -> String {
    let mut out = lines.join("\n");
    out.push('\n');
    out
}
