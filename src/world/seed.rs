//! World lernie-home seeding via lernie's **own** bootstrap verb (DESIGN §16.6
//! W3). On the first Start against an unseeded world yog founds the nested
//! `LERNIE_HOME` by invoking lernie's `prime` — never by reproducing lernie's
//! seed logic (DESIGN §14 rejection, "yog aping lernie's seeding": a second
//! seeder drifts from the first). The invocation is lernie's tested upstream
//! contract `LERNIE_HOME=<home> lernie prime` (lernie `tests/prime_cli.rs`):
//! argv-less beyond the verb, seed-if-absent, idempotent, product-less on
//! success (lernie ARCH §2.2, "Founding the harness root").
//!
//! **The seeded marker (lernie ARCH §2.2 / §4.2).** `prime` "lays down what a
//! ready installation carries: the default global `models.yaml` …", written at
//! the config root (`harness_root::models_path` = `<config-root>/models.yaml`),
//! which `LERNIE_HOME` collapses onto — the layout's [`lernie`](Layout::lernie)
//! dir. [`seeded`] probes exactly that file: what `prime` *guarantees*, not a
//! re-derivation of its full footprint (tools/skills pools, workflows/
//! workspaces dirs). `models.yaml` is hand-edited by contract (§4.2), so its
//! presence is the stable "installation founded" signal.
//!
//! **Skip is the general path, not a bootstrap branch (§3.4).** [`ensure_seeded`]
//! runs `prime` only when [`seeded`] is false; a seeded world is the ordinary
//! case with the seed present, and the skip returns cleanly with nothing run and
//! nothing logged — idempotent-or-convergent like every other start step.
//!
//! **The nesting rides the standing world env (§16.6 W2).** `prime` collapses
//! both lernie roots onto `LERNIE_HOME`, and yog's `lernie` [`Cli`] carries that
//! override standing (composed once at construction, §16.2) — so this module
//! sets no per-call env: `prime` rides the same world env as every other verb,
//! one source. It only chooses the cwd-less logged spawn ([`verbs::run_logged_cwdless`]),
//! since `prime` resolves its target from `LERNIE_HOME`, not cwd.

use std::io;
use std::path::Path;

use crate::actions::verbs::{self, Outcome};
use crate::cli_outbound::Cli;
use crate::opslog::Origin;
use crate::world::Layout;

/// lernie's bootstrap subcommand (ARCH §2.2). The whole argv is `[PRIME]`.
const PRIME: &str = "prime";
/// The global `models.yaml` `prime` founds at the config root (ARCH §4.2); its
/// presence under `LERNIE_HOME` is the seeded marker.
const MODELS_YAML: &str = "models.yaml";

/// A world-seed failure. `prime` ran non-zero (its stderr rides back in the
/// [`Outcome`], already appended to `ops.jsonl`), or its spawn failed
/// ([`Io`](SeedError::Io) — a missing `lernie`): both are durable ops rows now,
/// the spawn failure a Z5 synthetic-failure line, so neither is un-logged.
#[derive(Debug, thiserror::Error)]
pub enum SeedError {
    #[error("`lernie prime` failed (exit {}): {}", .0.exit, .0.stderr)]
    Prime(Outcome),
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Whether the world's lernie home is already seeded (DESIGN §16.6 W3): the
/// global `models.yaml` `prime` founds is present at the config-root location,
/// which `LERNIE_HOME` collapses onto (`layout.lernie`). Pure — one `is_file`
/// probe of what a ready installation minimally carries (lernie ARCH §2.2/§4.2).
pub fn seeded(layout: &Layout) -> bool {
    layout.lernie.join(MODELS_YAML).is_file()
}

/// Ensure the world's lernie home is seeded (DESIGN §16.6 W3) — the general path
/// with the seed present (§3.4). Already-seeded worlds short-circuit
/// (`Ok(false)`: nothing run, nothing logged); otherwise `lernie prime` founds
/// the home (`Ok(true)`) and its outcome lands in `ops.jsonl`. A non-zero
/// `prime` rides back as [`SeedError::Prime`]. `LERNIE_HOME` is not set here —
/// it rides the standing world env `lernie` carries (§16.6 W2), so the seed and
/// every other verb name the same nested home; `prime` resolves its target from
/// that env, not cwd, hence the cwd-less [`verbs::run_logged_cwdless`].
///
/// `origin` is the §7.3 attribution of the **start that needed the seed**
/// (bl-48f8): the seed is a substrate step of someone else's gesture, never a
/// gesture of its own, so it banners wherever that start was offered.
pub fn ensure_seeded(
    lernie: &Cli,
    state_root: &Path,
    ts: &str,
    layout: &Layout,
    origin: Origin,
) -> Result<bool, SeedError> {
    if seeded(layout) {
        return Ok(false);
    }
    let out = verbs::run_logged_cwdless(lernie, state_root, ts, &[PRIME], origin)?;
    if !out.ok() {
        return Err(SeedError::Prime(out));
    }
    Ok(true)
}

#[cfg(test)]
mod tests;
