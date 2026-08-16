//! **The chokepoint's one address resolution, and the raise it carries**
//! (REMOTE §8, §4.1) — split off [`dispatch`](super::dispatch) at §12's cap
//! (bl-4e08) on a real seam: everything left there is the `Action` table, and
//! this is the one thing that stands *ahead* of it.

use super::{Action, Deps};

/// The chokepoint's one address resolution, plus **the raise** (bl-8bbc).
///
/// Every gesture but one addresses a workspace that exists, and
/// [`ws_path`](crate::app::Snapshot::ws_path) is the whole answer. The
/// exception is [`Action::Prepare`], the §8.1 start family's mutating half,
/// whose `Step::EnsureWorkspace` **founds an absent workspace** — which is what
/// the window has always done by handing `prepare` a `<names-root>/<name>` path
/// directly, and what no seat but the window could do while the resolution
/// refused every name the enumeration lacked. So a `Prepare` naming an
/// unenumerated workspace resolves to yog's flat names root (§3.1), and the
/// name it founds is the operator's typed name exactly as at bootstrap.
///
/// **It can found or resume, never join.** A directory already at that path
/// that **is a workspace** (§3.1: it holds `repo.git`) is one this caller's
/// enumeration does not hold — another client's, hidden by REMOTE §4 scope —
/// and joining it would be the privilege escalation the scope exists to
/// prevent, so it refuses with the resolver's own sentence.
///
/// **Since bl-6c9e that is a statement about SCOPE and nothing else.** The
/// enumeration the intake hands over is now the live one
/// ([`addressable`](crate::app::addressable)), so "exists but is not in my set"
/// no longer includes *a wall this very caller founded a millisecond ago* — the
/// case that made a second `Prepare` refuse, and that made every non-`Prepare`
/// gesture naming the newborn refuse with it.
///
/// **And since bl-c9d2 a directory that is NOT a workspace does not refuse.**
/// No `repo.git` means no root enumerates it, so it is inside nobody's scope
/// and joining it escalates nothing — it is a birth that died between making
/// the directory and making the marker (`lernie new` orders them that way),
/// and refusing it wedged the name forever behind an `unknown workspace`
/// sentence about addressing. The raise resolves to the path and the
/// idempotent `EnsureWorkspace` decides: its create skips only on the marker,
/// so the resume runs `lernie new` against the debris and whatever *that*
/// says — success into an empty shell, or lernie's own destination refusal —
/// is an in-band, truthful sentence with a logged ops row, which the wedge
/// never had.
///
/// The scope refusal is also the one place existence is
/// observable to a scoped client, which REMOTE §4 records as a ruling: a
/// namespace with creation *by name* cannot also make a name's availability
/// unknowable, and what leaks is a name, never a workspace's contents.
pub(super) fn resolve_workspace(
    deps: &Deps,
    action: &Action,
    name: &str,
) -> Result<std::path::PathBuf, String> {
    let refusal = match deps.snapshot.ws_path(name) {
        Ok(path) => return Ok(path),
        Err(e) => e,
    };
    if !matches!(action, Action::Prepare { .. }) || !crate::naming::is_component(name) {
        return Err(refusal);
    }
    let raised = crate::binding::names_root(&deps.yog_data_root).join(name);
    if crate::binding::is_workspace(&raised) {
        return Err(refusal);
    }
    Ok(raised)
}
