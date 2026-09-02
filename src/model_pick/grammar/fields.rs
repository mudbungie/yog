//! **One entry's four-space field** — locate it, read it, replace it, add it or
//! drop it.
//!
//! [`roles`](super::roles), [`models`](super::models) and [`tools`](super::tools)
//! each carried the same prelude (find the column-0 block, find the two-space
//! entry, find the four-space field, map every miss to the [`GrammarError`] that
//! names it) before doing their one different thing. That prelude lives here
//! once, as [`locate`], and the plain "set this field" case is [`set_field`] —
//! which `set_role_model` is now two applications of.
//!
//! The same primitives are what the §9.5 typed config pane
//! ([`crate::config_edit::form`]) reads and writes settings through, so the
//! pane's controls, the §9.2 Apply gate and the §9.4 picker are one grammar
//! rather than three readers of one file shape.
//!
//! **What is NOT here is the entry** ([`super::entries`], bl-23bd): declaring a
//! whole entry, dropping one, and listing a block's entries are acts at the
//! altitude above, with their own callers (the §4.3 armed loop's `cadence.yaml`
//! rows), and this file had been carrying both since it was the place the
//! shared prelude landed. The two halves meet at [`entry_line`] and
//! [`entry_span`], which are the address of an entry and therefore that file's
//! to answer.

use super::entries::{entry_line, entry_span};
use super::{BlockKey, GrammarError, block_key, field, join};

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
/// the block form litany writes, like every other rewrite in this grammar.
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

/// Set one entry's four-space field whether or not the entry already carries
/// it (bl-23bd) — [`set_field`] with the missing-field case answered by adding
/// the line instead of refusing.
///
/// **Why this is not just `set_field` widened.** `set_field` refuses a field the
/// entry does not have, and that refusal is right for every caller it had: they
/// *move* a value the file already states, so an absent line means the file is
/// not the shape they were told it was. An **optional** assignment field is a
/// different fact — absent is its off state, so setting it is an add exactly as
/// often as it is a replace, and a caller made to tell the two apart would be
/// re-deriving what this function already had to look up.
///
/// The three refusals that are not about the field survive untouched: an inline
/// block, a block that is not there, an entry that is not there. You cannot add
/// a field to a role that does not exist, and guessing the role into being is
/// the YAML transform this grammar never is.
pub fn upsert_field(
    file: &'static str,
    text: &str,
    block: &str,
    entry: &str,
    name: &'static str,
    value: &str,
) -> Result<String, GrammarError> {
    match set_field(file, text, block, entry, name, value) {
        Err(GrammarError::NoField { .. }) => insert_field(file, text, block, entry, name, value),
        settled => settled,
    }
}

/// The half-open span one entry owns, or the refusal naming why it has none —
/// [`locate`]'s prelude minus its last step, shared by the two rewrites that
/// must find an entry without requiring the field to be there already.
fn span_of(
    file: &'static str,
    lines: &[&str],
    block: &str,
    entry: &str,
) -> Result<(usize, usize), GrammarError> {
    let missing = || GrammarError::NoEntry {
        file,
        entry: entry.to_owned(),
    };
    let at = match block_key(lines, block) {
        BlockKey::At(at) => at,
        BlockKey::Inline => {
            return Err(GrammarError::Inline {
                file,
                key: block.to_owned(),
            });
        }
        BlockKey::Absent => return Err(missing()),
    };
    entry_span(lines, at, entry).ok_or_else(missing)
}

/// Add a four-space field line to an entry that lacks it, **last among the
/// fields it already has** — after the entry's own key line and its existing
/// run, before the next entry. Blank lines inside the span are left where they
/// are: the insertion goes after the last line that carries anything, so a file
/// that separates its entries with blank lines keeps its shape.
fn insert_field(
    file: &'static str,
    text: &str,
    block: &str,
    entry: &str,
    name: &'static str,
    value: &str,
) -> Result<String, GrammarError> {
    let lines: Vec<&str> = text.lines().collect();
    let (from, to) = span_of(file, &lines, block, entry)?;
    // The entry's own key line is `from` and is never blank, so the scan needs
    // no fallback: it starts there and only ever moves forward onto another
    // line that carries something. A blank line or a comment between entries
    // is stepped over rather than inserted after, which is what keeps a file
    // that spaces its entries out looking the way its author left it.
    let mut last = from;
    for (offset, line) in lines.iter().enumerate().take(to).skip(from) {
        if !line.trim().is_empty() {
            last = offset;
        }
    }
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    out.insert(last + 1, format!("    {name}: {value}"));
    Ok(join(&out))
}

/// Drop one entry's four-space field line, leaving every other byte where it
/// is.
///
/// **Two absences, and only one of them is a fault.** An entry that never
/// carried the field is returned as it stands — removing what is already gone
/// is the same world, which is what lets an *off* switch be one act rather than
/// a read followed by a decision. But an entry that is not **there** is a
/// different thing entirely, and answering it with the unchanged file would
/// report success for a write that reached nothing. So the entry is required
/// and the field is not, which is exactly the asymmetry
/// [`upsert_field`] keeps, and it is why both refusals live here rather than in
/// each caller: a caller that gated the entry itself would be a second home for
/// one rule, and the two would phrase it differently.
pub fn remove_field(
    file: &'static str,
    text: &str,
    block: &str,
    entry: &str,
    name: &str,
) -> Result<String, GrammarError> {
    let lines: Vec<&str> = text.lines().collect();
    let (from, _) = span_of(file, &lines, block, entry)?;
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_string()).collect();
    if let Some((_, i)) = field(&lines, from, name) {
        out.remove(i);
    }
    Ok(join(&out))
}
