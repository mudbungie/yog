//! The **attempt** — one fork of a conversation from one point in its history
//! (VISION §5 V2, the Counterfactualist rung; DESIGN §5.1 #32, §11 rail).
//!
//! *"A pinned notch offers Fork from here: a goal composer seeded empty,
//! firing the ordinary fork with the pinned commit as ref."* That is the
//! whole gesture, and it is **one** gesture: an attempt. A **cohort** — V2's
//! ×N, the parallel candidates an operator compares — is N attempts fired
//! from one notch, and nothing anywhere records that they belong together.
//! Membership is [derived](crate::rail::cohort) from the notch each child
//! hangs on and the ref each forked off, both already on V1's cards, so
//! `N == 1` and `N > 1` are not two paths: they are one path walked once and
//! walked N times.
//!
//! **This is why there is no fan verb, no fan registry and no winner field.**
//! A gesture that fired "the whole fan" would have to name the fan, and a name
//! is a stored fact that the refs already imply. Firing the ordinary fork N
//! times leaves N ordinary `ops.jsonl` rows (§4.2) — N committed execution
//! facts — which is strictly more provenance than one row for N would be.
//!
//! **The upstream verb already exists** (litany bl-a693, in the pin since `=0.0.6`):
//! `litany dispatch <role> <ws> <parent> --goal <text> --from <ref>
//! [--pin <dest>=<src>]`. litany's own words: *"`--from <ref>` is not a second
//! kind of dispatch … the flag reaches the existing `fork_point` field … and
//! changes nothing else"* — so yog composes an argv and adds no mechanism.
//!
//! **The three fire-time controls are three real parameters of that argv**,
//! never a fourth thing yog invents (VISION V2.2, "yog policy made visible"):
//!
//! - **config branch** is the fork point itself (`--from`). One control with
//!   two kinds of value — the pinned commit ("from here", a fork with
//!   ancestry) or `config/<name>` (a clean start, provenance only) — which is
//!   VISION V1.3's ruling verbatim: *"'Clean vs fork' is one spawn gesture
//!   with one parameter — the fork point."*
//! - **model** is the **role** (`<role>`). litany resolves a model from
//!   `roles.<name>.{provider,model}` in the `providers.yaml` of the config
//!   lineage governing the fork point, read at that lineage's head, and from
//!   nowhere else. So the roles a ref declares, **with the model each names**,
//!   are read at that ref ([`roles_at`]): the model shown at the point of choice
//!   cannot lie, because yog is reading the very file the run will resolve
//!   against. Giving an attempt a model no config declares is a config write
//!   — §9.4's [`PickModel`](crate::boundary::Action::PickModel) — not a
//!   dispatch flag, and pretending otherwise would be capability theater.
//! - **skills** are pins (`--pin skills/<name>/SKILL.md=<pool>/<name>/SKILL.md`).
//!   litany's pin is documented as *"standing context a caller (a frontend, an
//!   operator) pins without rewriting the goal or authoring a config commit"*,
//!   and the shipped worker manifest composes `order: skills/**`, so a pinned
//!   skill reaches assembled context by the config's own glob. The pool is the
//!   world's `$LITANY_HOME/skills` ([`skills_root`]) — the same directory the
//!   agent's own `load_skill` tool copies out of, so a pin names exactly what
//!   the agent could have loaded for itself.
//!
//! **Composing the choice is the seat's** (bl-7cc8). The ×N control, the
//! fork-point list and the skill-pool listing were derived here and reached
//! nothing: `Action::Fork` carries one attempt by ruling (§8.5) and fires N
//! times, and no §8.5 reply carries an offer. They are gone; what stays is what
//! the argv itself needs.
//!
//! **Read-only by construction** (VISION §4.10, bl-2b8c). An attempt forks the
//! *conversation* repo and nothing else; project-mutating attempts need
//! §4.10's isolation and binding (yog bl-8746) and are not reachable from
//! here. Nothing in this module can touch a project worktree.

use std::path::{Path, PathBuf};

pub mod choices;
#[cfg(test)]
mod tests;

pub use choices::{roles_at, skills_root};

/// The litany subcommand an attempt is (ARCH §3.4).
const DISPATCH: &str = "dispatch";
const GOAL: &str = "--goal";
const FROM: &str = "--from";
const PIN: &str = "--pin";
/// The skills pool, under the world's `$LITANY_HOME` — and the destination
/// prefix a pinned skill lands at in the child's worktree. One constant,
/// because they are one name: the pin reproduces the pool's own layout.
pub const SKILLS_DIR: &str = "skills";
/// The one file of a skill directory a pin carries: its instructions. A pin's
/// destination is one path, so a skill's `references/**` stay where the
/// agent's own `load_skill` can still fetch them whole.
const SKILL_FILE: &str = "SKILL.md";
/// One attempt's fire-time overrides — everything that varies between the
/// candidates of a cohort. The goal, the workspace and the dispatching parent
/// do not vary (they are what makes the candidates comparable), so they are
/// not here: they ride the [`Fork`](crate::boundary::Action::Fork) action
/// beside this.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attempt {
    /// The fork point (`--from`): the pinned notch's commit, or
    /// `config/<name>`. Empty is not a value — a seat refuses to fire
    /// without one, because a fork with no ref is a different gesture.
    pub from: String,
    /// The role (`<role>`), which is the model: litany resolves the provider
    /// and model id from this name against `from`'s governing config.
    pub role: String,
    /// Skill names from the [`pool`], each pinned into the child's worktree.
    pub skills: Vec<String>,
}

/// One attempt, addressed: the candidate's overrides plus the three facts the
/// whole cohort shares (where, whose history, what for) and the world pool its
/// pins are drawn from. Owned and whole, so [`argv`] is a pure function of one
/// value and the executor carries no second parameter list to keep in step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fire {
    pub workspace: PathBuf,
    /// The dispatching parent's agent id (== its branch name).
    pub parent: String,
    /// The goal, verbatim (§3.3, bl-6920).
    pub goal: String,
    pub attempt: Attempt,
    /// The world's `$LITANY_HOME/skills` ([`skills_root`]).
    pub skills_root: PathBuf,
}

impl Fire {
    /// One attempt, addressed. The skills pool is the **world's**, derived from
    /// the same anchor every other world path is (§16.2) — a fire-time control
    /// reads yog's own substrate, never an ambient one — which is the only fact
    /// the boundary's composition adds to the gesture, so it is stated here
    /// beside the argv it feeds rather than at the chokepoint.
    pub fn at(
        workspace: &Path,
        parent: &str,
        attempt: &Attempt,
        goal: &str,
        yog_data_root: &Path,
    ) -> Self {
        Self {
            workspace: workspace.to_path_buf(),
            parent: parent.to_owned(),
            goal: goal.to_owned(),
            attempt: attempt.clone(),
            skills_root: skills_root(yog_data_root),
        }
    }
}

/// The argv one attempt fires: `litany dispatch <role> <ws> <parent> --goal
/// <goal> --from <ref> [--pin skills/<s>/SKILL.md=<pool>/<s>/SKILL.md]…`.
///
/// **The goal is passed verbatim** — the same rule the start flow's fire keeps
/// (§3.3, bl-6920): what an operator wrote is what the model reads, unmutated.
/// The pin sources are absolute, so a child spawned in any cwd resolves them.
pub fn argv(fire: &Fire) -> Vec<String> {
    let mut out = vec![
        DISPATCH.to_owned(),
        fire.attempt.role.clone(),
        fire.workspace.to_string_lossy().into_owned(),
        fire.parent.clone(),
        GOAL.to_owned(),
        fire.goal.clone(),
        FROM.to_owned(),
        fire.attempt.from.clone(),
    ];
    for skill in &fire.attempt.skills {
        out.push(PIN.to_owned());
        out.push(pin_spec(skill, &fire.skills_root));
    }
    out
}

/// One skill's `<dest>=<src>` pin spec. The destination mirrors the pool's own
/// layout, so a pinned skill and a `load_skill`-loaded one land at the same
/// path and the manifest's `skills/**` glob sees both.
fn pin_spec(skill: &str, skills_root: &Path) -> String {
    let src = skills_root.join(skill).join(SKILL_FILE);
    format!("{SKILLS_DIR}/{skill}/{SKILL_FILE}={}", src.display())
}
