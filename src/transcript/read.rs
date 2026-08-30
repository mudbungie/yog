//! **Reading the committed record** (§5.1 #12): the `messages/` directory,
//! turned into ordered [`Entry`] values.
//!
//! Split from [`super`] at §12's per-file budget (bl-73e7), on the seam the
//! module doc already draws and the follow lane made load-bearing: what a
//! transcript *is* and what it projects lives there, and the disk read that
//! produces one lives here. The two have different clocks — this is the
//! committed half, refreshed when the step commits, and the tail beside it now
//! arrives frame by frame over the follow lane.

use std::path::{Path, PathBuf};

use super::{
    AGENTS_DIR, Entry, EntryKind, JSON_EXT, MD_EXT, MESSAGES_DIR, TOOL_ORIGIN, Transcript,
    compaction, parse_model, parse_tool_result,
};

/// Build the **committed** transcript for `agent_id` in `workspace`: the
/// `messages/` directory, with a marker seated in every hole compaction left
/// in its counter ([`compaction`] — the directory is not append-only, and a
/// readdir alone renders a rewritten record as if it were the whole record).
/// The live tail is [`Transcript::with_live`] — see the module doc for why
/// the two are not one call.
pub fn build(workspace: &Path, agent_id: &str) -> Transcript {
    let agent = workspace.join(AGENTS_DIR).join(agent_id);
    let read = read_messages(&agent.join(MESSAGES_DIR));
    Transcript {
        entries: compaction::splice(&agent, read),
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
/// A delivered `.md` is a **deposit file moved verbatim** — litany's
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
pub fn classify(name: &str, raw: &[u8]) -> EntryKind {
    let Some((_, origin, ext)) = parse_name(name) else {
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

/// Split `NNN-<origin>.<ext>` into `(NNN, origin, ext)`. Anything not matching
/// the shape is `None` (→ Raw bucket).
///
/// The counter is **returned** since bl-7bd2: filename order carries where an
/// entry sits, but only its value says which entries are *not there*
/// ([`compaction`]), and one parse of this shape is the whole of what either
/// caller may know about it.
pub(super) fn parse_name(name: &str) -> Option<(&str, &str, &str)> {
    let (stem, ext) = name.rsplit_once('.')?;
    let (num, origin) = stem.split_once('-')?;
    if num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) || origin.is_empty() {
        return None;
    }
    Some((num, origin, ext))
}
