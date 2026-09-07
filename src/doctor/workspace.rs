//! **The workspace's two** (bl-28f4): the wall a first conversation needs, and
//! who is registered to reach it. Split from [`doctor`](super) on the seam the
//! gesture's own address draws — these answer only when a workspace was named.

use std::path::Path;

use super::Row;
use crate::boundary::dispatch::Deps;

/// The two, for the workspace the gesture named.
pub(super) fn rows(deps: &Deps, name: &str, path: &Path) -> Vec<Row> {
    vec![wall(deps, name, path), clients(deps, name)]
}

/// **Would a conversation started here reach a model?** — asked by the `Prompt`
/// door's own gate, so the doctor and the fire cannot disagree. Its refusal is
/// the best sentence in the suite and it is quoted verbatim as the remedy,
/// which is the whole of this gesture's idea: that sentence, asked before the
/// failure instead of at it.
fn wall(deps: &Deps, name: &str, path: &Path) -> Row {
    // The default birth, on both axes: `config/default` and litany's `worker`
    // (bl-9ced). The doctor asks about the workspace, not about one operator's
    // pending gesture — a start that names a lineage or a role is asking a
    // different question, and the sentence it would earn is the one the door
    // gives it at the fire.
    match crate::boundary::dispatch::signin_gate(deps, path, None, None) {
        Ok(()) => Row::ok(
            "wall",
            format!("{name}'s wall holds a credential a role can use"),
        ),
        Err(refusal) => Row::bad(
            "wall",
            format!("a conversation started in {name} would reach no model"),
            refusal,
        ),
    }
}

/// Who is registered here, who is connected, and who has never once dialled —
/// §5's join, counted. It is informational and passes either way: a workspace
/// driven from one terminal seat has every reason to hold a single row, and a
/// doctor that called that a fault would cry wolf on the ordinary shape.
fn clients(deps: &Deps, name: &str) -> Row {
    let rows = crate::registry::roster::roster(&deps.state_root, &deps.caller.presence, name);
    let present = rows.iter().filter(|row| row.present).count();
    Row::ok(
        "clients",
        format!("{} registered, {present} connected now", rows.len()),
    )
}
