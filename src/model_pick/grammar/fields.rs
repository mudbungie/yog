//! The generic field access every rewrite in this grammar was restating —
//! locate one entry's four-space field, read it, or replace it.
//!
//! [`roles`](super::roles), [`models`](super::models) and [`tools`](super::tools)
//! each carried the same prelude (find the column-0 block, find the two-space
//! entry, find the four-space field, map every miss to the [`GrammarError`] that
//! names it) before doing their one different thing. That prelude lives here
//! once, as [`locate`], and the plain "set this field" case is [`set_field`] —
//! which `set_role_model` is now two applications of.
//!
//! The same three primitives are what the §9.5 typed config pane
//! ([`crate::config_edit::form`]) reads and writes settings through, so the
//! pane's controls, the §9.2 Apply gate and the §9.4 picker are one grammar
//! rather than three readers of one file shape.

use super::{BlockKey, GrammarError, block_end, block_key, entries, field, join};

/// The four-space `<name>:` line of `entry` under `block`: its value and its
/// index in `lines`. Every miss is the refusal that names it — an inline block
/// key, a missing entry (which is also what an absent block means: there is no
/// such entry), or a missing field.
pub(super) fn locate(
    file: &'static str,
    lines: &[&str],
    block: &str,
    entry: &str,
    name: &'static str,
) -> Result<(String, usize), GrammarError> {
    let at = match block_key(lines, block) {
        BlockKey::At(at) => at,
        BlockKey::Inline => {
            return Err(GrammarError::Inline {
                file,
                key: block.to_owned(),
            });
        }
        BlockKey::Absent => {
            return Err(GrammarError::NoEntry {
                file,
                entry: entry.to_owned(),
            });
        }
    };
    let start = entry_line(lines, at, entry).ok_or_else(|| GrammarError::NoEntry {
        file,
        entry: entry.to_owned(),
    })?;
    field(lines, start, name).ok_or_else(|| GrammarError::NoField {
        file,
        entry: entry.to_owned(),
        field: name,
    })
}

/// The line index of the two-space `<entry>:` key under the block at `at`.
fn entry_line(lines: &[&str], at: usize, entry: &str) -> Option<usize> {
    entries(lines, at)
        .into_iter()
        .find(|(name, _)| name == entry)
        .map(|(_, i)| i)
}

/// Every two-space entry key under `block`, in file order — the model ids a
/// `models.yaml` declares, the roles a `providers.yaml` declares. A file with
/// no such block (or an inline one) declares nothing, so it lists nothing:
/// absence is a value, exactly as it is for the §9.2 gate.
pub fn entry_names(text: &str, block: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let BlockKey::At(at) = block_key(&lines, block) else {
        return Vec::new();
    };
    entries(&lines, at)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

/// One entry's four-space field value, or `None` when the block, the entry or
/// the field is not there. The forgiving read the pane derives a row from — a
/// field that is absent is a control the file does not have, not an error.
pub fn entry_field(text: &str, block: &str, entry: &str, name: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let BlockKey::At(at) = block_key(&lines, block) else {
        return None;
    };
    let start = entry_line(&lines, at, entry)?;
    Some(field(&lines, start, name)?.0)
}

/// Replace one entry's four-space field value, preserving every other byte
/// (comments, sibling fields, sibling entries). Declines loudly on anything but
/// the block form lernie writes, like every other rewrite in this grammar.
pub fn set_field(
    file: &'static str,
    text: &str,
    block: &str,
    entry: &str,
    name: &'static str,
    value: &str,
) -> Result<String, GrammarError> {
    let lines: Vec<&str> = text.lines().collect();
    let (_, i) = locate(file, &lines, block, entry, name)?;
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    // The index came from this very line vector, so the assignment lands.
    if let Some(slot) = out.get_mut(i) {
        *slot = format!("    {name}: {value}");
    }
    Ok(join(&out))
}

/// Declare one whole entry — the two-space key plus its four-space fields —
/// replacing any entry of that name outright. The block is created at the end
/// of the file when it is absent, because an entry is what the caller is
/// asserting and a block with no entries asserts nothing. `None` is the one
/// refusal: an *inline* block key (`monitor: {}`) cannot be rewritten without
/// becoming a YAML transform, which this grammar never is.
///
/// Whole-entry replacement rather than field-wise editing is deliberate: the
/// caller states the entry it wants, so a stale sibling field left behind by an
/// older shape cannot survive the rewrite and be read back as policy.
pub fn set_entry(
    text: &str,
    block: &str,
    entry: &str,
    fields: &[(&str, String)],
) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    let body = std::iter::once(format!("  {entry}:"))
        .chain(fields.iter().map(|(k, v)| format!("    {k}: {v}")));
    match block_key(&lines, block) {
        BlockKey::Inline => return None,
        BlockKey::Absent => {
            out.retain(|line| !line.trim().is_empty());
            out.push(format!("{block}:"));
            out.extend(body);
        }
        BlockKey::At(at) => {
            let span = entry_span(&lines, at, entry);
            if let Some((from, to)) = span {
                out.splice(from..to, body);
            } else {
                out.splice((at + 1)..=at, body);
            }
        }
    }
    Some(join(&out))
}

/// Remove one entry and every line it owns. A file that never declared it is
/// returned as it stands — severability is deleting the entry, so deleting one
/// that is already gone is the same world, not an error.
pub fn remove_entry(text: &str, block: &str, entry: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    if let BlockKey::At(at) = block_key(&lines, block)
        && let Some((from, to)) = entry_span(&lines, at, entry)
    {
        out.drain(from..to);
    }
    join(&out)
}

/// The half-open line range one entry owns under the block at `at`: its own
/// two-space key line through the line before the next entry (or the block's
/// end). `None` when the block declares no such entry.
fn entry_span(lines: &[&str], at: usize, entry: &str) -> Option<(usize, usize)> {
    let found = entries(lines, at);
    let from = found.iter().find(|(name, _)| name == entry)?.1;
    let to = found
        .iter()
        .map(|(_, i)| *i)
        .find(|i| *i > from)
        .unwrap_or_else(|| block_end(lines, from));
    Some((from, to))
}
