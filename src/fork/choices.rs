//! **What one seat offers** — the fork points an attempt may fire from and the
//! skills it may carry, each derived on demand and stored nowhere. Split from
//! the attempt itself at §12's budget on the seam the module already names:
//! [`super`] is *what an attempt is and the argv it fires*, this is *what this
//! workspace and this world make available to fire*.

use crate::config_edit::branch::{config_branches, config_file, governing_config};
use crate::model_pick::grammar::{self, RoleModel};
use std::path::{Path, PathBuf};

use super::{CONFIG_REF, SKILL_FILE, SKILLS_DIR};

/// The `providers.yaml` a fork point's governing config declares its roles in
/// (lernie ARCH §4.3) — the one file that binds a role to a model.
const PROVIDERS: &str = "providers.yaml";

/// The world's skill pool: every directory under `skills_root` that carries a
/// `SKILL.md`, sorted. A directory without one is not a skill, and an absent
/// pool is no skills rather than an error — the composer then offers none,
/// which is the general path with an empty input.
pub fn pool(skills_root: &Path) -> Vec<String> {
    let Ok(read) = std::fs::read_dir(skills_root) else {
        return Vec::new();
    };
    let mut names: Vec<String> = read
        .flatten()
        .filter(|e| e.path().join(SKILL_FILE).is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// One fork point the composer offers, with the policy it carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForkPoint {
    /// What the operator reads: `here` for the pinned notch, else the config
    /// branch's name — the same two words V1's card labels already use.
    pub label: String,
    /// What `--from` receives: the pinned commit oid, or `config/<name>`.
    pub refspec: String,
    /// The roles this ref's **governing config commit** declares, each with
    /// the provider row and model id it names. Read from that commit's
    /// `providers.yaml` — the file lernie itself resolves against — so the
    /// model shown at the point of choice is the model that will run. Empty
    /// when the ref names no config lineage yog can reach: the composer then
    /// offers nothing to fire rather than guessing a role.
    pub roles: Vec<RoleModel>,
}

/// Everything the fork composer offers for one pinned notch: where an attempt
/// may fork from, and which skills it may carry. Derived on demand from the
/// workspace repo and the world's pool; stored nowhere.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Choices {
    pub points: Vec<ForkPoint>,
    pub skills: Vec<String>,
}

impl Choices {
    /// The fork point with this refspec, if it is one this seat offers.
    pub fn point(&self, refspec: &str) -> Option<ForkPoint> {
        self.points.iter().find(|p| p.refspec == refspec).cloned()
    }

    /// Is there anything to fire? A workspace whose config lineage yog cannot
    /// reach declares no roles anywhere, and a composer with no role to name
    /// would be a button that cannot work — so the seat does not paint.
    pub fn fireable(&self) -> bool {
        self.points.iter().any(|p| !p.roles.is_empty())
    }
}

/// Derive the composer's choices for a pinned notch: **here** (the pinned
/// commit, a fork carrying the conversation's own ancestry) followed by every
/// `config/<name>` branch (a clean start, provenance only). Each point's roles
/// are read at the config commit that governs it, which for a config head is
/// itself and for the pinned commit is its nearest `config/*` ancestor — the
/// one derivation §5.1 #17 already makes, asked at a ref instead of a tip.
///
/// A point whose config cannot be resolved carries no roles rather than
/// vanishing: the operator sees the ref and sees that it offers nothing, which
/// is a fact about the workspace and not a silence.
pub fn choices(workspace: &Path, pinned_commit: &str, skills_root: &Path) -> Choices {
    let mut points = vec![point(
        "here".to_owned(),
        pinned_commit.to_owned(),
        workspace,
    )];
    for branch in config_branches(workspace).unwrap_or_default() {
        let refspec = format!("{CONFIG_REF}{}", branch.name);
        points.push(point(branch.name, refspec, workspace));
    }
    Choices {
        points,
        skills: pool(skills_root),
    }
}

/// One fork point, with the roles its governing config declares.
fn point(label: String, refspec: String, workspace: &Path) -> ForkPoint {
    ForkPoint {
        roles: roles_at(workspace, &refspec),
        label,
        refspec,
    }
}

/// The roles a ref's governing config commit declares (`providers.yaml`'s
/// `roles:` block), each with its provider row and model id. Reuses §9.4's own
/// grammar reader, so the picker and the fork composer can never disagree
/// about what a config file says.
pub fn roles_at(workspace: &Path, refspec: &str) -> Vec<RoleModel> {
    let Ok(gov) = governing_config(workspace, refspec) else {
        return Vec::new();
    };
    let Ok(bytes) = config_file(workspace, &gov.oid, PROVIDERS) else {
        return Vec::new();
    };
    grammar::roles(&String::from_utf8_lossy(&bytes))
}

/// The world's skills pool directory, `$LERNIE_HOME/skills` — the same path
/// lernie's own `load_skill` tool resolves. Derived from the world layout, so
/// yog's nested substrate (§16.2) and the pool it offers are one fact.
pub fn skills_root(yog_data_root: &Path) -> PathBuf {
    // Bound rather than chained: tarpaulin's llvm engine mis-attributes a
    // multi-line method chain's tail as uncovered, and rustfmt's chain width
    // will not keep this one on a single line.
    let world = crate::world::layout_under(yog_data_root);
    world.lernie.join(SKILLS_DIR)
}
