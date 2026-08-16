//! Tests for the §9.4 picker's pure half. Split by concern: the generic
//! [`fields`](super::grammar::entry_field) access every rewrite shares, the
//! block [`grammar`](super::grammar), the roster [`query`](super::query), the
//! composed [`plan`](super::plan) + the sentences the surface paints, the
//! conversation [`header`](super::header) line and its drift clause, the
//! `models.yaml` read-back that judges a declared provider row (`validate`),
//! the [`remedy`](super::remedy) a credential-shaped roster failure routes to,
//! and the protocol-`capability` gate that keeps a row brazen ships but cannot
//! serve a role out of both config files (bl-3d22).

mod capability;
mod fields;
mod grammar;
mod header;
mod plan;
mod query;
mod remedy;
mod validate;

/// brazen's effective table as the fixtures need it: one row per name, keyless,
/// on a dialect that **carries tools** (bl-3d22). A fixture of bare names could
/// not exercise the pick gate at all — the gate asks two questions of the table
/// and only one of them is answerable from a name.
pub(crate) fn table(names: &[&str]) -> Vec<crate::config_edit::brazen::ProviderRow> {
    rows_on(names, "openai_chat")
}

/// The same table on a named dialect — the seam the capability tests aim a
/// `claude_code` row, and an unspellable protocol, through.
pub(crate) fn rows_on(
    names: &[&str],
    protocol: &str,
) -> Vec<crate::config_edit::brazen::ProviderRow> {
    names
        .iter()
        .map(|name| crate::config_edit::brazen::ProviderRow {
            name: (*name).to_owned(),
            protocol: protocol.to_owned(),
            auth: "none".to_owned(),
        })
        .collect()
}

/// `providers.yaml` exactly as lernie's own `template/providers.yaml` writes
/// it — the shape the grammar is defined over (§9.4). Every rewrite test reads
/// this back, so a template change upstream fails here rather than in the UI.
pub(crate) const TEMPLATE_PROVIDERS: &str = "roles:\n  worker:\n    provider: codex\n    model: gpt-5.4\n    \
     tools: [bash, read_file, load_skill]\n  compactor:\n    provider: codex\n    model: gpt-5.4-mini\n";

/// `models.yaml` as lernie's `install/models.yaml` seeds it: a comment header,
/// a top-level `models:` block, two-space entries.
pub(crate) const SEEDED_MODELS: &str = "# Global config-root models.yaml (ARCH §4.2).\n\nmodels:\n  \
     gpt-5.4:\n    provider: codex\n    model_id: gpt-5.4\n    capabilities: [tool_use_native, streaming]\n    \
     context_window: 400000\n";
