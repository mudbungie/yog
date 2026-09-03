//! **What this world makes available to a fork** — the roles a ref's governing
//! config declares, and where the skill pool lives.
//!
//! What a *seat offers* used to live here too: the fork-point list, the skill
//! pool listing, and the `Choices` fold over them. They left with the composer
//! that consumed them (bl-7cc8) — `Action::Fork` carries one attempt by ruling
//! (§8.5) and no reply carries a choice, so the offer was a seat's arithmetic
//! held in a server. What remains is read by production: §9.4's role grammar at
//! a ref, and the world's pool path.

use crate::config_edit::branch::{config_file, governing_config};
use crate::model_pick::grammar::{self, RoleModel};
use std::path::{Path, PathBuf};

use super::SKILLS_DIR;

/// The `providers.yaml` a fork point's governing config declares its roles in
/// (litany ARCH §4.3) — the one file that binds a role to a model.
const PROVIDERS: &str = "providers.yaml";

/// The roles the config a ref resolves declares (`providers.yaml`'s `roles:`
/// block), each with its provider row and model id. Reuses §9.4's own grammar
/// reader — and the same which-config-governs derivation every other surface
/// asks — so the picker, the fork composer and the engine can never disagree
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

/// The world's skills pool directory, `$LITANY_HOME/skills` — the same path
/// litany's own `load_skill` tool resolves. Derived from the world layout, so
/// yog's nested substrate (§16.2) and the pool it offers are one fact.
pub fn skills_root(yog_data_root: &Path) -> PathBuf {
    // Bound rather than chained: tarpaulin's llvm engine mis-attributes a
    // multi-line method chain's tail as uncovered, and rustfmt's chain width
    // will not keep this one on a single line.
    let world = crate::world::layout_under(yog_data_root);
    world.litany.join(SKILLS_DIR)
}
