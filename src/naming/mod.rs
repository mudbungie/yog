//! **Wire names** (REMOTE §8, bl-f5f6): how a workspace or a project is
//! addressed when a path may not cross the boundary.
//!
//! A boundary gesture used to carry an absolute `PathBuf`. Across machines that
//! is meaningless — the client's filesystem is not the engine's — and it is a
//! disclosure besides: an operator's home root in every envelope, every deposit
//! file and every reply a seat can read. The wire spelling is the **name**, and
//! the engine resolves it to a path at the one chokepoint that already holds
//! the world ([`dispatch`](crate::boundary::dispatch::dispatch) /
//! [`answer`](crate::boundary::answer::answer)).
//!
//! **Two nouns, and the rule differs because the nouns do.**
//!
//! - A **workspace already has a name**: §3.1 says its directory leaf *is* the
//!   name, and §3.2 makes that same leaf the `--as` identity every ball claim
//!   is stamped with. So [`leaf`] is the whole rule — no derivation, no second
//!   spelling, and **no special case for a foreign workspace**: lernie's
//!   `workspaces/`/`replays/` leaves are the auto-ids the tab strip already
//!   paints as their identity (`nav::tabs::tab`: "the display name is the path
//!   leaf"). Two roots holding one leaf is a world whose §3.2 join is already
//!   ambiguous — both would claim `--as home` — so [`by_leaf`] refuses it
//!   naming the token rather than inventing a disambiguator for a world that is
//!   broken one level down.
//! - A **project has no name at all**: its identity is the decoded balls
//!   invocation path (§5.1 #1), and two checkouts of one repo legitimately
//!   share a basename. So one is derived — [`name_of`], the shortest trailing
//!   run of components no other enumerated project shares, which is the
//!   basename wherever that is already unique.
//!
//! **It is the same mapping read in both directions.** The frame holds paths
//! and spells the forward reads where a seat's selection becomes a gesture; the
//! engine spells the resolvers where a gesture becomes an act. Nothing is
//! stored: a name is derived from the live enumeration exactly as the §3.5
//! binding and the §11 roster label are.
//!
//! **The roster label is the project name, elided** ([`crate::projects::labels`]).
//! That was `projects`' own private derivation until this module took it, and
//! two copies of "shortest unique tail" would have drifted the moment one
//! learned about a case the other did not — so what the operator reads off the
//! left panel is exactly the word they may type at `--project`.

use std::path::{Path, PathBuf};

/// A workspace's name (§3.1): its directory leaf. Empty for a rootless path,
/// which is never a real workspace — the general path with no input.
///
/// The one definition, read by the §3.2 `--as`/`YOG_NAME` stamp, the §16.2 wall
/// lookup, the search corpus and the boundary's addressing alike.
pub fn leaf(workspace: &Path) -> String {
    workspace
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// True iff `name` is a **plain path component** — non-empty, no separator of
/// either platform, and not one of the two directory names every filesystem
/// already spends (`.`, `..`).
///
/// The one home of that question (bl-8bbc), because two nouns now become
/// directory names off a wire: a §4 client identity, which is a *certificate's*
/// text and therefore an untrusted peer's, and the §3.1 workspace name a raise
/// founds. A name that could carry a separator is a name that could address the
/// filesystem, and the check belongs beside [`leaf`] — the inverse operation —
/// rather than at each caller.
pub fn is_component(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
}

/// The workspace in `set` whose [`leaf`] is `name`, or the refusal naming the
/// token. Ambiguity refuses too, and says so: a leaf two roots both hold cannot
/// address one workspace, and a guess would act on the wrong world.
pub fn by_leaf(set: &[PathBuf], name: &str) -> Result<PathBuf, String> {
    let mut hits = set.iter().filter(|p| leaf(p) == name);
    let Some(first) = hits.next() else {
        return Err(format!("unknown workspace {name:?}"));
    };
    match hits.next() {
        None => Ok(first.clone()),
        Some(_) => Err(format!("ambiguous workspace {name:?}")),
    }
}

/// The wire name of the project at `path` within `set`: the shortest trailing
/// run of `path`'s components that no member of `set` **other than `path`
/// itself** shares, falling back to the whole path when no run is unique.
///
/// **Injective over `set`** — two distinct members cannot name alike, which is
/// what lets [`resolve`] take the first match rather than counting. A `path`
/// the set does not hold names *itself* (no suffix of it occurs, so none is
/// unique), and [`resolve`] then refuses that name — the alias is impossible
/// rather than merely unlikely.
pub fn name_of(set: &[PathBuf], path: &Path) -> String {
    let depth = path.components().count();
    (1..=depth)
        .map(|k| (suffix(path, k), k))
        .find(|(cand, k)| set.iter().filter(|o| &suffix(o, *k) == cand).count() == 1)
        .map_or_else(|| suffix(path, depth), |(cand, _)| cand)
}

/// The project in `set` that `name` addresses, or the refusal naming the token.
///
/// **A name nothing answers is a refusal, never a guess** — the same strict
/// discipline the gesture codec decodes by (§8.5): a gesture is an instruction,
/// so an address that resolves to nothing must not be silently rewritten into
/// one that resolves to something.
pub fn resolve(set: &[PathBuf], name: &str) -> Result<PathBuf, String> {
    set.iter()
        .find(|p| name_of(set, p) == name)
        .cloned()
        .ok_or_else(|| format!("unknown project {name:?}"))
}

/// `path`'s last `k` components, rendered — the whole path when it is shorter.
fn suffix(path: &Path, k: usize) -> String {
    let total = path.components().count();
    path.components()
        .skip(total.saturating_sub(k))
        .collect::<PathBuf>()
        .display()
        .to_string()
}

#[cfg(test)]
mod tests;
