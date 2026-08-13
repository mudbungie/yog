//! The one rendering a hit carries: the matched **line**, windowed around the
//! match (§8.5 search).
//!
//! A line is the unit because a line is what an operator recognizes; the window
//! is because a `messages/` entry can be one enormous line and a result row is
//! not a pane. Both bounds are byte counts walked back to a char boundary — no
//! slice is ever taken on a guess (AGENTS.md rule 4: no panic paths).

/// Bytes of context kept before the match.
const LEAD: usize = 40;
/// Bytes of the line the window spans, at most.
const WIDTH: usize = 160;
/// What marks a window that does not reach its line's edge.
const ELISION: char = '…';

/// The matched line around byte `offset`, elided to [`WIDTH`]. `offset` is a
/// byte index into `text`; an offset past the end simply yields the tail, which
/// is the general path rather than a checked error.
pub fn excerpt(text: &str, offset: usize) -> String {
    let offset = offset.min(text.len());
    let start = text
        .get(..offset)
        .and_then(|head| head.rfind('\n'))
        .map_or(0, |i| i + 1);
    let end = text
        .get(offset..)
        .and_then(|tail| tail.find('\n'))
        .map_or(text.len(), |i| offset + i);
    let line = text.get(start..end).unwrap_or_default();
    let from = boundary(line, offset.saturating_sub(start).saturating_sub(LEAD));
    let to = boundary(line, from.saturating_add(WIDTH));
    let mut out = String::new();
    if from > 0 {
        out.push(ELISION);
    }
    out.push_str(line.get(from..to).unwrap_or_default().trim());
    if to < line.len() {
        out.push(ELISION);
    }
    out
}

/// The char boundary at or before byte `at` — the one place this module is
/// allowed to decide where a slice may start.
fn boundary(text: &str, at: usize) -> usize {
    if at >= text.len() {
        return text.len();
    }
    text.char_indices()
        .map(|(i, _)| i)
        .take_while(|i| *i <= at)
        .last()
        .unwrap_or(0)
}
