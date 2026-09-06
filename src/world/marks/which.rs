//! **Which store a directory's tasks live in** (DESIGN §16.2 as amended by the
//! one-store-per-project ruling, bl-262a).
//!
//! Split from [`super`] at §12's pre-split band, on the seam the invariant
//! draws: that module is *what a space is* — balls' two homes, the world's, an
//! agent's own, the host's — and this one is *whose space a given directory
//! resolves*, which is a different question with a different answer.

use std::path::{Path, PathBuf};

use super::Space;
use crate::xdg::Env;

/// **The space a DIRECTORY's balls state lives in — one store per project**
/// (§16.2 as amended, ruling 1 on bl-262a).
///
/// The nested world isolates yog's OWN substrate. It never meant, and cannot
/// mean, that a directory the operator already tracks with `bl` has a second
/// store because yog is the one asking: balls keys a clone on `(state home,
/// invocation path)`, so overriding the state home for an operator's checkout
/// produced a store at the right path with nothing in it — an agent asked for
/// a report on a project's board and wrote, in prose, that the board was empty
/// while `bl -C <repo> list` printed four balls.
///
/// So the branch is on the **directory**, which is balls' own key:
///
/// - **World-owned** — anything under `<yog-data-root>`: a litany workspace
///   checkout, a wall, a `work/<id>` worktree cut inside the world. These are
///   yog's own directories, nobody else addresses them, and they resolve
///   [`super::space`] — the world's, or the agent's `YOG_MARKS` space where one is
///   layered on. §16.3's per-agent default lives entirely here.
/// - **Everything else** — the operator's own checkouts: [`Space::host`], the
///   store their own shell resolves. `YOG_MARKS` does not reach them, which is
///   the ruling: a project's board is the project's, not the visiting agent's,
///   and §16.3's ball rung already said as much for the one case it could see
///   ("its `bl` *is* the board's").
///
/// Severability is unchanged, and that is what keeps the branch honest: the
/// world's clones stay under `<yog-data-root>` so one `rm -rf` still erases the
/// whole world, while an operator's clone was never yog's to erase.
pub fn space_for(env: &Env, dir: &Path) -> Space {
    if world_owned(env, dir) {
        super::space(env)
    } else {
        Space::host(env)
    }
}

/// Does the world own `dir`? One test — is it under `<yog-data-root>` — because
/// that root IS the world (§16.2's severability clause says so from the other
/// side: one `rm -rf` of it erases everything yog owns). Both paths are
/// canonicalized where they exist, so a symlinked data root or a `/tmp` that is
/// really `/private/tmp` compares as itself.
pub fn world_owned(env: &Env, dir: &Path) -> bool {
    real(dir).starts_with(real(&env.yog_data_root()))
}

/// `p` resolved through symlinks when it exists, else `p` itself — a path that
/// is not there yet still has to answer this question.
fn real(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::{space_for, world_owned};
    use crate::world::marks::Space;
    use crate::xdg::Env;
    use std::path::Path;

    /// A world composed the way `Engine::boot` composes one: the anchor is
    /// ambient, the state home is nested, and the host's own is carried in
    /// beside it (§16.2, bl-262a).
    fn world() -> Env {
        crate::world::compose(&Env::from_pairs([
            ("HOME", "/home/u"),
            ("XDG_DATA_HOME", "/home/u/.local/share"),
            ("XDG_STATE_HOME", "/home/u/.local/state"),
            ("XDG_CONFIG_HOME", "/home/u/.config"),
        ]))
    }

    /// **A directory the world owns resolves the world's space** — a litany
    /// workspace checkout, which is where §16.3's per-agent default lives.
    #[test]
    fn a_world_owned_directory_resolves_the_worlds_space() {
        let world = world();
        let inside = Path::new("/home/u/.local/share/yog/world/litany/workspaces/alba");
        assert!(world_owned(&world, inside));
        assert_eq!(
            space_for(&world, inside),
            crate::world::marks::space(&world)
        );
    }

    /// **Every other directory resolves the store its own operator resolves**
    /// (ruling 1 on bl-262a) — both of balls' homes, because a landing holds
    /// one store worktree and two `tasks_branch` readings would thrash it.
    #[test]
    fn an_operators_own_checkout_resolves_the_hosts_space() {
        let world = world();
        let outside = Path::new("/home/u/dev/proj");
        assert!(!world_owned(&world, outside));
        let space = space_for(&world, outside);
        assert_eq!(space, Space::host(&world));
        assert_eq!(space.state, Path::new("/home/u/.local/state"));
        assert_eq!(space.config, Path::new("/home/u/.config"));
    }

    /// **An agent's own space does not reach an operator's checkout.** The
    /// per-agent default (§16.3) governs the world's directories; a project's
    /// board is the project's, whoever is asking.
    #[test]
    fn an_own_marks_space_does_not_reach_an_operators_checkout() {
        let agent = world().with_overrides(&[(
            crate::world::marks::YOG_MARKS,
            "/home/u/.local/share/yog/world/walls/alba/marks",
        )]);
        assert_eq!(
            space_for(&agent, Path::new("/home/u/dev/proj")),
            Space::host(&agent),
            "the directory decides, not the visitor"
        );
        assert_eq!(
            space_for(&agent, Path::new("/home/u/.local/share/yog/world/litany/x")),
            Space::own(Path::new("/home/u/.local/share/yog/world/walls/alba/marks")),
            "…and inside the world the agent's own space still decides"
        );
    }
}
