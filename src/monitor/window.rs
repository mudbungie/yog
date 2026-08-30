//! The evidence one check reads (VISION §4.9): the goal verbatim, and the
//! transcript delta since the last-checked sha.
//!
//! Both come off disk yog already derives from — `goal.md` and the committed
//! `messages/` transcript (§5.1 #12) — so watching an agent adds no wire tap
//! and no store. **v1 reads the committed transcript only**: the in-flight
//! streaming tail is deliberately not folded in, because staging text has no
//! commit to replay a verdict against.
//!
//! The delta is a *derivation from the sha the ops row names*, never a
//! remembered cursor: `git diff` between the last-checked sha and the branch
//! tip lists the message files that appeared, and those entries are the window.
//! A first-ever check has no sha to diff from and so reads the whole
//! transcript — the general path with an empty baseline, not a bootstrap case.
//!
//! **Everything here is quoted as data.** The fold below emits transcript
//! content under a heading and never re-frames it as a message to the judge;
//! tool-lessness (the check holds no verbs at all) is what bounds the damage a
//! poisoned transcript can do, and the policy prompt says the rest.
//!
//! **A compacted record rides the window, summary and all** (the bl-fde5
//! ruling, recorded in VISION §4.9). litany's compactor deletes message files
//! and hands the agent `summary/NNN.md` in their place (§5.1 #12), so after a
//! compaction the summary is part of what the agent reads on **every** step —
//! and §4.9's premise is that the monitor reads what the agent read. The
//! counterargument was folding non-agent words into the evidence; it loses
//! because the summary is litany's artifact the agent actually consumed, not
//! yog's commentary, and omitting it hands the judge a window with a hole
//! exactly where the agent's context was rewritten. It is quoted as data under
//! a stated heading like every other line. The marker rides **every** window,
//! not only the delta it landed in: it is standing context the way the goal
//! is, and the delta diff (`--diff-filter=AM` over `messages/`) cannot see a
//! deletion, so gating the marker on the delta would omit it from the one
//! check where the compaction is news.

use crate::transcript::{Block, Entry, EntryKind, Transcript};
use std::path::Path;

/// The committed-transcript directory inside an agent's worktree, as the diff
/// pathspec spells it.
const MESSAGES: &str = "messages/";

/// How many `char`s of folded transcript one check sends, kept from the **tail**
/// — the newest work is the work being judged, and a delta that outgrew the
/// budget has already told the operator more than one call should carry.
const WINDOW_MAX: usize = 24_000;

/// What a check reads.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Evidence {
    /// `goal.md` verbatim — the agent's assignment, quoted, never extracted.
    pub goal: String,
    /// The folded transcript delta.
    pub window: String,
}

/// Gather the evidence for `agent` in `workspace`. `since` is the last-checked
/// sha (absent on the first check); `tip` the branch tip the verdict will name.
pub fn gather(workspace: &Path, agent: &str, since: Option<&str>, tip: &str) -> Evidence {
    let dir = workspace.join("agents").join(agent);
    let goal = std::fs::read_to_string(dir.join("goal.md")).unwrap_or_default();
    let transcript = crate::transcript::build(workspace, agent);
    let window = fold(&transcript, delta(workspace, since, tip).as_deref());
    Evidence { goal, window }
}

/// The message-file basenames that appeared between `since` and `tip`, or
/// `None` when there is no baseline to diff from (a first check) or git could
/// not answer — either way the whole transcript is the window, which is the
/// conservative reading: a check never sees *less* than it should.
fn delta(workspace: &Path, since: Option<&str>, tip: &str) -> Option<Vec<String>> {
    let repo = workspace.join(crate::git_tree::REPO_DIR);
    let names = crate::git_tree::diff_names(&repo, since?, tip, MESSAGES).ok()?;
    Some(
        names
            .iter()
            .filter_map(|p| p.strip_prefix(MESSAGES).map(str::to_owned))
            .collect(),
    )
}

/// Fold the selected entries to plain text, tail-clipped. `only` names the
/// entries to keep; `None` keeps them all. A compaction marker is kept
/// regardless — it is standing context, no file backs it, and no diff can name
/// it (module doc).
fn fold(transcript: &Transcript, only: Option<&[String]>) -> String {
    let text: String = transcript
        .entries
        .iter()
        .filter(|e| {
            matches!(e.kind, EntryKind::Compacted { .. })
                || only.is_none_or(|names| names.contains(&e.name))
        })
        .map(say)
        .collect::<Vec<_>>()
        .join("\n");
    tail(&text)
}

/// One entry as a line-oriented quotation. Model turns carry their text and,
/// when the provider committed them, their thinking; tool calls carry name and
/// input summary; a tool result carries its content and whether it errored.
fn say(entry: &Entry) -> String {
    match &entry.kind {
        EntryKind::Delivered { sender, body, .. } => format!("[message from {sender}]\n{body}\n"),
        EntryKind::Model { blocks, .. } => {
            let said: Vec<String> = blocks.iter().map(block).collect();
            format!("[agent]\n{}\n", said.join("\n"))
        }
        EntryKind::ToolResult {
            content, is_error, ..
        } => {
            let mark = if *is_error { " (error)" } else { "" };
            format!("[tool result{mark}]\n{content}\n")
        }
        // The compaction record, quoted as data (module doc): the span the
        // counter proves deleted, and the summary litany handed the agent in
        // its place — what the agent read, which is what the judge reads.
        EntryKind::Compacted {
            first,
            last,
            summary,
        } => {
            let span = format!("entries {first:03}\u{2013}{last:03}");
            if summary.is_empty() {
                format!("[record compacted here: {span} deleted; no summary on this mark]\n")
            } else {
                format!(
                    "[record compacted here: {span} deleted; litany's summary replaced \
                     them in the agent's context]\n{summary}\n"
                )
            }
        }
        // None of these reaches a v1 check's text: the streaming tail is
        // excluded by construction (`build(.., false)` — staging text has no
        // commit to replay a verdict against), and a Raw entry is bytes yog
        // could not classify.
        EntryKind::Streaming { .. } | EntryKind::Raw => String::new(),
    }
}

fn block(block: &Block) -> String {
    match block {
        Block::Text(text) => text.clone(),
        Block::Thinking(text) => format!("(thinking) {text}"),
        Block::ToolUse {
            name,
            input_summary,
            ..
        } => format!("(tool {name}) {input_summary}"),
    }
}

/// The last [`WINDOW_MAX`] `char`s, with a stated elision when anything was cut
/// — the judge is told its view is partial rather than shown a text that
/// silently starts mid-sentence.
fn tail(text: &str) -> String {
    let len = text.chars().count();
    if len <= WINDOW_MAX {
        return text.to_owned();
    }
    let kept: String = text.chars().skip(len - WINDOW_MAX).collect();
    format!("[earlier work elided]\n{kept}")
}

#[cfg(test)]
mod tests;
