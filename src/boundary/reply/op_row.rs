//! **The §4.2 trail row, both directions** (§8.5, §7.3; bl-4d81) — the one
//! answered row whose derived readings *are* the answer.
//!
//! Every other listing row spells a fact somebody else's type already holds.
//! This one spells the durable `ops.jsonl` line **and** the three readings §7.3
//! names — is it a failure, what its `exit` says in words, and where it stands
//! now — because the failure banner §7.3 mandates is otherwise five derivations
//! re-implemented per seat (the sentinel table, the `128+n` reading, the
//! `(cwd, verb)` retirement key, the ack watermark scan and the origin
//! grouping), whose failure mode is silent divergence rather than a compile
//! error. That is the seam this file is cut on, and it is why the two
//! directions sit together here rather than one in [`super::rows`] and the
//! other in [`super::rows::decode`].

use serde_json::{Value, json};

use crate::boundary::codec::fields::{i64_of, str_of};
use crate::boundary::codec::{origin_token, parse_origin};
use crate::opslog::{OpRow, OpView, Standing};

/// The wire's five words for [`Standing`] — [`Standing::token`]'s other half.
/// Two spellings of one vocabulary is this codec's standing shape: the match is
/// the compile gate, the table is the parser, and the round trip over every arm
/// is what holds them together.
const STANDINGS: [(&str, Standing); 5] = [
    ("clean", Standing::Clean),
    ("detached", Standing::Detached),
    ("live", Standing::Live),
    ("retired", Standing::Retired),
    ("acked", Standing::Acked),
];

/// One trail row on the wire: the durable line verbatim, then the derived facts.
///
/// The line is `ts`/`argv`/`cwd`/`exit`/`stdout`/`stderr` plus the stamped
/// `origin`, which crosses because it cannot be derived (bl-48f8). Beside it:
/// `failed` — §7.3's own predicate, `OpRow::failed`; `exit_label` —
/// `ExitKind::label`, so no seat ever paints a bare `-3`; and `standing` —
/// `opslog::Standing`, §6's retirement folded with §4.2's ack watermark. "One
/// banner per origin" is then the `live` rows grouped by `origin`.
///
/// `failed` rides beside `standing` rather than being read off it because it is
/// the **row's own** question, answerable of a single row held alone — an
/// expanded detail, a banner quoting one line — while `standing` is a fact
/// about that row's place in a tail. Two questions, and only the second needs
/// the tail present to ask.
pub(super) fn op_row(view: &OpView) -> Value {
    let row = &view.row;
    json!({
        "ts": row.ts, "argv": row.argv, "cwd": row.cwd, "exit": row.exit,
        "stdout": row.stdout, "stderr": row.stderr,
        "origin": origin_token(row.origin),
        "failed": row.failed(), "exit_label": row.exit_label(),
        "standing": view.standing.token(),
    })
}

/// The same row read back. `standing` is read **strictly** — it is the one
/// derived field a reader cannot recompute, since it is a fact about the row's
/// place in a tail this frame may hold only part of.
///
/// `failed` and `exit_label` are deliberately *not* read into anything: they
/// are recomputable from the line the row already carries, and the encoder
/// above recomputes them, so a round trip returns the frame exactly while the
/// crate keeps one home for each reading. That is the same discipline
/// `conv_row`'s `display` is decoded under (bl-7067) — the wire states a
/// derivation so a seat need not do it; it does not thereby gain a second
/// authority for it.
pub(crate) fn decode(v: &Value) -> Result<OpView, String> {
    let o = v.as_object().ok_or("ops row: not an object")?;
    Ok(OpView {
        row: OpRow {
            ts: str_of(o, "ts")?,
            argv: str_of(o, "argv")?,
            cwd: str_of(o, "cwd")?,
            exit: i32::try_from(i64_of(o, "exit")?).map_err(|_| "ops row: exit out of range")?,
            stdout: str_of(o, "stdout")?,
            stderr: str_of(o, "stderr")?,
            origin: parse_origin(&str_of(o, "origin")?)?,
        },
        standing: crate::boundary::codec::fields::pick(o, "standing", &STANDINGS)?,
    })
}

#[cfg(test)]
mod tests;
