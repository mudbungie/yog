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
/// **It can only ever found, never join.** A directory already at that path is
/// a workspace this caller's enumeration does not hold — a stale snapshot, or
/// the REMOTE §4 scope hiding another client's — and joining it would be the
/// privilege escalation the scope exists to prevent, so it refuses with the
/// resolver's own sentence. That refusal is also the one place existence is
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
    if raised.exists() {
        return Err(refusal);
    }
    Ok(raised)
}
