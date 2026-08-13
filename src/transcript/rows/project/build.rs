//! **How a row is built** — the identity it wears, the constructor every
//! variant funnels through, and the preview/body split that decides what a
//! `▶` has to reveal.
//!
//! Split from [`super`] at §12's per-file budget, on the seam the parent's own
//! doc already names: *what an entry becomes* is the exhaustive per-variant
//! match up there; *what a row is made of* is here. Nothing in this file knows
//! the entry vocabulary — it takes a prefix, a payload and a class — which is
//! why the two change for unrelated reasons.

use super::super::{Fold, Row, RowClass, Tone};
use crate::theme::Role;

/// Key namespace for a transcript row's fold override (see [`super::super`]).
const KEY_ROOT: &str = "tx";
/// Characters of payload a contracted row previews before the ellipsis.
const PREVIEW_CAP: usize = 160;

/// A row's stable identity: the entry's filename and the block ordinal.
/// `pub(crate)` because the step spine's placement derivation
/// (`crate::rail::place`) names the row each rule paints above by this same
/// key — one spelling of the identity, not two.
pub(crate) fn key(name: &str, block: usize) -> String {
    format!("{KEY_ROOT}/{name}#{block}")
}

/// Build a row, splitting `payload` into its one-line preview and the body
/// that folding reveals. `expanded` is filled by [`rows`]. `role` is who the
/// row speaks for (§11 role stripe) — `None` on machinery, where nobody is.
pub(super) fn row(
    key: String,
    prefix: String,
    payload: &str,
    class: RowClass,
    tone: Tone,
    role: Option<Role>,
) -> Row {
    let (preview, body) = split(payload);
    Row {
        key,
        prefix,
        preview,
        body,
        hover: String::new(),
        class,
        tone,
        role,
        fold: Fold::Payload,
        expanded: false,
    }
}

/// State in the prefix how big the fold is, in **characters** (bl-1f75,
/// operator: *"I'd like tool result collapses to show me the number of
/// characters in the output"*). Characters, not bytes: a byte count lies about
/// any payload carrying non-ASCII, and this seat is read by a human sizing up
/// a click.
///
/// The count is the **body's** — what the `▶` opens onto — which makes "a row
/// with nothing to fold says nothing" the same rule as the toggle's own, not a
/// second one about small payloads (see [`split`]: the empty body *is* the
/// fact). It also means the plural never arises: a body exists only where the
/// payload is clipped or multi-line, so it is never one character long.
///
/// Derived at projection time from the payload the entry already carries — no
/// field, no cache, nothing stored (§11: the row is a pure projection).
pub(super) fn with_size(mut row: Row) -> Row {
    if !row.body.is_empty() {
        let chars = row.body.chars().count();
        row.prefix = format!("{} · {chars} chars", row.prefix);
    }
    row
}

/// Split a payload into `(one-line preview, foldable body)`. The body is
/// **empty** when the payload is already one line short enough to show whole
/// — the row then has nothing to fold, which is why no separate "foldable"
/// flag exists (the empty body *is* the fact).
fn split(payload: &str) -> (String, String) {
    let first = payload.lines().next().unwrap_or_default();
    let clipped = first.chars().count() > PREVIEW_CAP;
    let preview = if clipped {
        let head: String = first.chars().take(PREVIEW_CAP).collect();
        format!("{head}…")
    } else {
        first.to_string()
    };
    let more = clipped || payload.lines().nth(1).is_some();
    let body = if more {
        payload.to_string()
    } else {
        String::new()
    };
    (preview, body)
}
