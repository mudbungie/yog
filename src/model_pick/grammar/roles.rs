//! The `providers.yaml` half of the §9.4 block grammar: reading back every role
//! the file declares, and rewriting exactly one role's `provider:`/`model:`
//! fields in place.
//!
//! The sibling of [`models`](super::models) and [`tools`](super::tools) — one
//! module per file the picker touches, all three over [`super`]'s shared
//! anchored primitives. Pure text → text; every byte outside the two rewritten
//! lines survives, and anything off-grammar declines loudly rather than being
//! guessed at.

use super::{
    BlockKey, GrammarError, PROVIDERS_YAML, ROLES, RoleModel, block_key, entries, field, set_field,
};

/// Every role `providers.yaml` declares, in file order. A role whose block
/// lacks `provider:` or `model:` is omitted — it is not an assignment yet, and
/// a rewrite of it would refuse anyway ([`set_role_model`]).
pub fn roles(providers_yaml: &str) -> Vec<RoleModel> {
    let lines: Vec<&str> = providers_yaml.lines().collect();
    let BlockKey::At(at) = block_key(&lines, ROLES) else {
        return Vec::new();
    };
    entries(&lines, at)
        .into_iter()
        .filter_map(|(role, i)| {
            Some(RoleModel {
                role,
                provider: field(&lines, i, PROVIDER)?.0,
                model: field(&lines, i, MODEL)?.0,
            })
        })
        .collect()
}

/// Rewrite one role's `provider:` and `model:` lines in place, preserving
/// every other byte (comments, `tools:`, sibling roles) — two applications of
/// the one [`set_field`] rewrite. The pair is all-or-nothing: a role missing
/// either line refuses and the intermediate text is dropped unwritten.
/// The role assignment's four-space field names, said once (bl-23bd): two
/// required — the (row, id) pointer §9.4 writes — and two optional tuning knobs
/// litany reads as `Option`s. `&'static str` because [`GrammarError::NoField`]
/// carries the name it could not find.
pub const PROVIDER: &str = "provider";
/// The model id half of the pointer.
pub const MODEL: &str = "model";
/// The role's reasoning-effort level (litany ARCH §4.3, upstream bl-acba).
pub const EFFORT: &str = "effort";
/// The role's priority-lane request (upstream bl-f587).
pub const PRIORITY: &str = "priority";

pub fn set_role_model(
    providers_yaml: &str,
    role: &str,
    provider: &str,
    model: &str,
) -> Result<String, GrammarError> {
    let out = set_field(
        PROVIDERS_YAML,
        providers_yaml,
        ROLES,
        role,
        PROVIDER,
        provider,
    )?;
    set_field(PROVIDERS_YAML, &out, ROLES, role, MODEL, model)
}
