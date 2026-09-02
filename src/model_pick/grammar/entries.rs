//! **One block's entries** — every two-space key it declares, one whole entry
//! written or dropped, and the line address the field primitives beside this
//! ([`super::fields`]) locate a field within.
//!
//! Split off `fields` at §12's pre-split band on the altitude seam that module
//! was already straddling (bl-23bd): a field belongs to an entry, an entry
//! belongs to a block, and the callers differ — the §9.4 picker and the §9.5
//! pane move fields, while the §4.3 armed loop's `cadence.yaml` rows are whole
//! entries appearing and disappearing. Declines the same way every rewrite in
//! this grammar does: on anything but the block form litany itself writes.

use super::{BlockKey, block_end, block_key, entries, join};

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

/// The line index of the two-space `<entry>:` key under the block at `at`.
pub(super) fn entry_line(lines: &[&str], at: usize, entry: &str) -> Option<usize> {
    entries(lines, at)
        .into_iter()
        .find(|(name, _)| name == entry)
        .map(|(_, i)| i)
}

/// The half-open line range one entry owns under the block at `at`: its own
/// two-space key line through the line before the next entry (or the block's
/// end). `None` when the block declares no such entry.
pub(super) fn entry_span(lines: &[&str], at: usize, entry: &str) -> Option<(usize, usize)> {
    let found = entries(lines, at);
    let from = found.iter().find(|(name, _)| name == entry)?.1;
    let to = found
        .iter()
        .map(|(_, i)| *i)
        .find(|i| *i > from)
        .unwrap_or_else(|| block_end(lines, from));
    Some((from, to))
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
