//! The anchored block grammar yog reads and writes the two config files
//! through: `providers.yaml`'s `roles:` — the §9.4 picker's one write, and the
//! single home of a role's (provider row, model id) pointer — and
//! `models.yaml`'s `models:`, which no longer reaches litany at all and is
//! yog's own table (see `models.rs`).
//!
//! **This is not a YAML parser and must never become one.** yog declares no
//! YAML dependency (§9.2) and litany's own parser is private (its crate exposes
//! only `cmd`), so it recognizes exactly the block shape litany's
//! template authors — a top-level key at column 0, two-space entry keys,
//! four-space fields —
//!
//! ```text
//! roles:                      models:
//!   worker:                     gpt-5.4:
//!     provider: codex             context_window: 400000
//!     model: gpt-5.4
//! ```
//!
//! The `models:` entry is the shape yog WRITES since bl-3ffa — the id and the one
//! fact anything reads out of it. The read side takes the block's older four-field
//! shape unchanged (`provider`, `model_id`, `capabilities` beside it), because
//! these are anchored line reads: a field nothing looks for is a line nothing
//! looks at.
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

mod fields;
mod models;
mod roles;
mod tools;

pub use fields::{entry_field, entry_names, remove_entry, set_entry, set_field};
pub use models::{DEFAULT_CONTEXT_WINDOW, context_windows, declare_model, is_unknown_row};
pub use roles::{roles, set_role_model};
pub use tools::{flow_members, flow_value};

/// The two files this grammar is spoken over, named in every refusal.
pub const PROVIDERS_YAML: &str = "providers.yaml";
pub const MODELS_YAML: &str = "models.yaml";

/// The column-0 block key each file's entries hang under — the role map and the
/// model map. Named once here because three rewrites and the §9.5 pane all
/// spell them.
pub const ROLES: &str = "roles";
pub const MODELS: &str = "models";

/// One role's assignment as `providers.yaml` declares it (§5.1 #27).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleModel {
    pub role: String,
    pub provider: String,
    pub model: String,
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
