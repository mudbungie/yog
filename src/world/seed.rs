//! World litany-home seeding via litany's **own** bootstrap verb (DESIGN §16.6
//! W3). On the first Start against an unseeded world yog founds the nested
//! `LITANY_HOME` by invoking litany's `prime` — never by reproducing litany's
//! seed logic (DESIGN §14 rejection, "yog aping litany's seeding": a second
//! seeder drifts from the first). The invocation is litany's tested upstream
//! contract `LITANY_HOME=<home> litany prime` (litany `tests/prime_cli.rs`):
//! argv-less beyond the verb, seed-if-absent, idempotent, product-less on
//! success (litany ARCH §2.2, "Founding the harness root").
//!
//! **The seeded marker (litany ARCH §2.2 / §4.2).** `prime` "lays down what a
//! ready installation carries: the default global `models.yaml` …", written at
//! the config root (`harness_root::models_path` = `<config-root>/models.yaml`),
//! which `LITANY_HOME` collapses onto — the layout's [`litany`](Layout::litany)
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
//! both litany roots onto `LITANY_HOME`, and yog's `litany` [`Cli`] carries that
//! override standing (composed once at construction, §16.2) — so this module
//! sets no per-call env: `prime` rides the same world env as every other verb,
//! one source. It only chooses the cwd-less logged spawn ([`verbs::run_logged_cwdless`]),
//! since `prime` resolves its target from `LITANY_HOME`, not cwd.

use std::io;
use std::path::{Path, PathBuf};

use crate::actions::verbs::{self, Outcome};
use crate::cli_outbound::Cli;
use crate::opslog::Origin;
use crate::world::Layout;

/// litany's bootstrap subcommand (ARCH §2.2). The whole argv is `[PRIME]`.
const PRIME: &str = "prime";
/// The global `models.yaml` `prime` founds at the config root (ARCH §4.2); its
/// presence under `LITANY_HOME` is the seeded marker.
const MODELS_YAML: &str = "models.yaml";

/// The engine home's name before the REMOTE §12 version fence — `<world>/lernie`
/// beside today's `<world>/litany` (DESIGN §16.2). A world carrying it was
/// founded by a `lernie 0.0.x` engine and has not been migrated.
const LERNIE_ERA_HOME: &str = "lernie";

/// A world-seed failure. `prime` ran non-zero (its stderr rides back in the
/// [`Outcome`], already appended to `ops.jsonl`), the world is a lernie-era one
/// that has not been migrated ([`Unmigrated`](SeedError::Unmigrated)), or the
/// spawn failed ([`Io`](SeedError::Io) — a missing `litany`): the two that ran a
/// child are durable ops rows, the spawn failure a Z5 synthetic-failure line, so
/// none is un-logged.
#[derive(Debug, thiserror::Error)]
pub enum SeedError {
    #[error("`litany prime` failed (exit {}): {}", .0.exit, .0.stderr)]
    Prime(Outcome),
    #[error(
        "this world was founded by the lernie-era engine and still holds {0} — \
         priming now would found a second, empty engine home beside it. Migrate \
         it first: rename that directory to `litany`, then rewrite every mark in \
         each `workspaces/*/repo.git` from `refs/lernie/*` to `refs/litany/*`."
    )]
    Unmigrated(PathBuf),
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Whether the world's litany home is already seeded (DESIGN §16.6 W3): the
/// global `models.yaml` `prime` founds is present at the config-root location,
/// which `LITANY_HOME` collapses onto (`layout.litany`). Pure — one `is_file`
/// probe of what a ready installation minimally carries (litany ARCH §2.2/§4.2).
pub fn seeded(layout: &Layout) -> bool {
    layout.litany.join(MODELS_YAML).is_file()
}

/// Ensure the world's litany home is seeded (DESIGN §16.6 W3) — the general path
/// with the seed present (§3.4). Already-seeded worlds short-circuit
/// (`Ok(false)`: nothing run, nothing logged); otherwise `litany prime` founds
/// the home (`Ok(true)`) and its outcome lands in `ops.jsonl`. A non-zero
/// `prime` rides back as [`SeedError::Prime`]. `LITANY_HOME` is not set here —
/// it rides the standing world env `litany` carries (§16.6 W2), so the seed and
/// every other verb name the same nested home; `prime` resolves its target from
/// that env, not cwd, hence the cwd-less [`verbs::run_logged_cwdless`].
///
/// `origin` is the §7.3 attribution of the **start that needed the seed**
/// (bl-48f8): the seed is a substrate step of someone else's gesture, never a
/// gesture of its own, so it banners wherever that start was offered.
///
/// **A lernie-era world refuses instead of seeding** (DESIGN §16.2, REMOTE §12).
/// The engine renamed with no compatibility shim, so an unmigrated world's
/// conversations sit under `<world>/lernie` where nothing looks any more, and
/// `prime` would happily found an empty `<world>/litany` beside them — the
/// stranding, made permanent and silent. The probe is not a bootstrap branch: a
/// fresh world has no such directory and takes the general path with empty
/// inputs, and a migrated world never reaches it (it is seeded). yog does not
/// perform the migration — it is one operator paste over an enumerable
/// population inside the pre-stability fence that exists for exactly this, and
/// splitting it between boot code and that paste would be two representations of
/// one act. This is §5's use-is-attempt: the refusal lands at the first gesture
/// that would do harm, naming the remedy.
pub fn ensure_seeded(
    litany: &Cli,
    state_root: &Path,
    ts: &str,
    layout: &Layout,
    origin: Origin,
) -> Result<bool, SeedError> {
    if seeded(layout) {
        return Ok(false);
    }
    let era = layout.root.join(LERNIE_ERA_HOME);
    if era.is_dir() {
        return Err(SeedError::Unmigrated(era));
    }
    let out = verbs::run_logged_cwdless(litany, state_root, ts, &[PRIME], origin)?;
    if !out.ok() {
        return Err(SeedError::Prime(out));
    }
    Ok(true)
}

#[cfg(test)]
mod tests;
