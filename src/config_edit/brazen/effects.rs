//! The production [`BzRunner`] impl — the thin shell behind the pure
//! view-model.
//!
//! **§16.7 W10 split the surface in two.** The three *read* verbs
//! (`--dump-config` at a temp, `--dump-config` effective, `--list-providers`)
//! are config projection: data-shaped, so they run **in-process** through the
//! linked brazen ([`crate::bz_host`]) against the world [`Env`] — no spawn, no
//! pipe, no version skew between the validator and the thing being validated.
//! `--list-models` reaches the network and writes brazen's on-disk cache, so it
//! stays a subprocess — now `yog bz --list-models …`, yog's own executable under
//! the `bz` namespace ([`Binary::Bz`] is self-multiplexed), which is the same
//! linked implementation on the far side of the fork.
//!
//! The filesystem seam ([`RealFileIo`](super::super::RealFileIo)) is shared
//! across every editor and lives in `config_edit::effects`.

use super::{BzOutcome, BzRunner, ProviderRow, provider_rows};
use crate::bz_host::{self, Tty};
use crate::cli_outbound::{Binary, Chunk, Cli, ExitInfo};
use crate::xdg::Env;
use std::path::Path;

/// `bz` runner. Carries the [`Env`] the in-process reads fold through and the
/// [`Cli`] the one remaining spawn execs.
#[derive(Debug, Clone)]
pub struct RealBzRunner {
    cli: Cli,
    env: Env,
}

impl RealBzRunner {
    /// Build over an explicit `Cli`/`Env` pair — the seam the recorder tests
    /// drive both halves through (a hermetic `BRAZEN_CONFIG` for the in-process
    /// reads, a recorder script for the spawn). Production folds through
    /// [`resolve`](Self::resolve).
    #[cfg(test)]
    pub(crate) fn new(cli: Cli, env: Env) -> Self {
        Self { cli, env }
    }

    /// Resolve the `bz` spawn target in the composed `world` (§16.6 W2) and
    /// keep that same `Env` for the in-process reads. **Both halves carry the
    /// wall** (§16.2 as amended): the in-process reads fold brazen's three
    /// locations out of the `Env` directly, and the one spawn gets the wall
    /// layered on top of the world's own overrides — so `--list-models` writes
    /// the cache of the workspace whose picker asked for it, not the machine's.
    pub fn resolve(world: &Env) -> Self {
        let mut overrides = crate::world::overrides(world);
        overrides.extend(crate::world::wall::pairs_of(world));
        Self {
            cli: Cli::resolve_in_world(Binary::Bz, &overrides),
            env: world.clone(),
        }
    }

    /// Run `bz <args>` **in this process** (§16.7 W10) and fold the captured
    /// stdio into a [`BzOutcome`]. stdin is empty: no read verb consumes it.
    fn capture(&self, args: &[&str]) -> BzOutcome {
        let argv = args.iter().map(|a| (*a).to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let exit = bz_host::run(
            argv,
            &self.env,
            Tty::PIPED,
            &mut std::io::empty(),
            &mut out,
            &mut err,
        );
        BzOutcome {
            success: exit == 0,
            stdout: String::from_utf8_lossy(&out).into_owned(),
            stderr: String::from_utf8_lossy(&err).into_owned(),
        }
    }

    /// Spawn `bz <args>`, drain stdout/stderr, fold to a [`BzOutcome`]. A
    /// spawn failure is itself a non-success outcome carrying the error.
    fn collect(&self, args: &[&str]) -> BzOutcome {
        match self.cli.run(args) {
            Ok(stream) => {
                let mut out = Vec::new();
                let mut err = Vec::new();
                let mut success = false;
                for chunk in stream {
                    match chunk {
                        Chunk::Stdout(b) => out.extend(b),
                        Chunk::Stderr(b) => err.extend(b),
                        Chunk::Exited(e) => success = e == ExitInfo::Code(0),
                    }
                }
                BzOutcome {
                    success,
                    stdout: String::from_utf8_lossy(&out).into_owned(),
                    stderr: String::from_utf8_lossy(&err).into_owned(),
                }
            }
            Err(e) => BzOutcome {
                success: false,
                stdout: String::new(),
                stderr: e.to_string(),
            },
        }
    }
}

impl BzRunner for RealBzRunner {
    fn dump_config_at(&self, config: &Path) -> BzOutcome {
        self.capture(&["--config", &config.display().to_string(), "--dump-config"])
    }

    fn dump_config_effective(&self) -> BzOutcome {
        self.capture(&["--dump-config"])
    }

    fn providers(&self) -> Vec<ProviderRow> {
        provider_rows(&self.capture(&["--list-providers", "--json"]).stdout)
    }

    fn list_models(&self, provider: &str) -> BzOutcome {
        self.collect(&["--list-models", "--provider", provider, "--json"])
    }
}

#[cfg(test)]
mod tests;
