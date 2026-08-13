//! Tests for the §9.4 picker's pure half. Split by concern: the generic
//! [`fields`](super::grammar::entry_field) access every rewrite shares, the
//! block [`grammar`](super::grammar), the roster [`query`](super::query), the
//! composed [`plan`](super::plan) + the sentences the surface paints, the
//! conversation [`header`](super::header) line and its drift clause, the
//! `models.yaml` read-back that judges a declared provider row (`validate`),
//! and the [`remedy`](super::remedy) a credential-shaped roster failure routes
//! to.

mod fields;
mod grammar;
mod header;
mod plan;
mod query;
mod remedy;
mod validate;

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
