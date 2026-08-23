//! Help (§8.5): **a query, and a higher-order one** — it is asked *about* a
//! gesture, and it is the same question at every seat.
//!
//! It belongs to the §8.5 taxonomy exactly where the others do: it populates
//! rather than mutates, it returns typed data both frontends render, and it has
//! a headless spelling ([`Query::Help`](super::Query::Help)). What sets it apart
//! is its subject — **the derivation is over the interface, not the world** —
//! and that has one consequence worth stating: help is the only query with no
//! snapshot to read, so any seat can answer it *in place*, with no consumer, no
//! deposit and no wait. The deposit path still answers it (parity is not
//! optional), but nothing has to go that way to learn what a verb does.
//!
//! [`table`] is the single source: the line reader's refusals, the roster a
//! bare `/` prints, the per-verb detail, and the parity test that no spelling
//! drifts from a gesture all read this one list. A verb here that the reader
//! does not answer — or the reverse — is a test failure, not a doc bug.

pub mod table;
#[cfg(test)]
mod tests;

/// The whole verb roster — the acts on a conversation or a ball, then the
/// standing/settings verbs, then the queries, then the follow-class reads
/// whose answer is a sequence (bl-73e7), in that order. A **function**,
/// not a const, because the list outgrew one file at §12's cap (bl-dc0c,
/// bl-2d19) and const slices cannot be concatenated in a const: the split is a
/// line budget, so it must not become a second list anyone can read half of.
/// Cheap: `HelpRow` is `Copy` over `'static` strs.
pub fn table() -> Vec<HelpRow> {
    [
        table::ACTIONS,
        table::standing::STANDING,
        table::queries::QUERIES,
        table::following::FOLLOWING,
    ]
    .concat()
}

/// One command, as help states it: how it is typed, what it does in a line, and
/// the paragraph an operator reads when they ask about it specifically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelpRow {
    /// The gesture's one name — the line's `/verb` and the envelope's `op`.
    pub verb: &'static str,
    pub usage: &'static str,
    pub summary: &'static str,
    pub detail: &'static str,
}

/// Whether `verb` names a gesture. The reader, the codec and every seat ask
/// this one question rather than carrying a second list.
pub fn known(verb: &str) -> bool {
    table().iter().any(|row| row.verb == verb)
}

/// The help asked for: one row when a verb is named, the whole table when it is
/// not. An unknown verb is refused before it reaches here (the codec's strict
/// decode, the reader's unknown-command refusal), so this is total.
pub fn rows(verb: Option<&str>) -> Vec<HelpRow> {
    match verb {
        Some(name) => table().into_iter().filter(|row| row.verb == name).collect(),
        None => table(),
    }
}

/// Help as text — the one rendering every seat prints (the composer's note, the
/// terminal's stdout, a TUI's pane). **One row is a page, many are a roster**:
/// asking about a verb earns its paragraph, asking about everything earns the
/// list, because a wall of paragraphs answers no question anyone asked.
pub fn render(rows: &[HelpRow]) -> String {
    match rows {
        [row] => format!("{}\n\n{}", row.usage, row.detail),
        many => many
            .iter()
            .map(|row| format!("  {}\n      {}", row.usage, row.summary))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

/// The roster, as the tail of a refusal that has just named an unknown verb.
pub fn roster() -> String {
    render(&rows(None))
}
