//! The production [`BzRunner`] impl — the thin shell behind the pure
//! view-model.
//!
//! **§16.7 W10 split the surface in two; bl-dff8 closed the split.** Every read
//! verb here — `--dump-config` at a temp, `--dump-config` effective,
//! `--list-providers`, and now `--list-models` — runs **in-process** through the
//! linked brazen ([`crate::bz_host`]) against the world [`Env`]: no spawn, no
//! pipe, no version skew between the validator and the thing being validated.
//! The roster kept a subprocess (`yog bz --list-models …`) while it had one
//! caller, the §9.4 picker, which must not block a frame; that caller does not
//! use this runner — it drives its own streamed [`Cli`](crate::cli_outbound::Cli)
//! ([`crate::model_pick::query`]) because it paints each line as it lands. Every
//! caller of *this* method is already off-frame (the boundary's answer
//! chokepoint), so the fork bought nothing and cost a second brazen.
//!
//! The filesystem seam ([`RealFileIo`](super::super::RealFileIo)) is shared
//! across every editor and lives in `config_edit::effects`.

use super::{BzOutcome, BzRunner, ProviderRow, provider_rows};
use crate::bz_host::{self, Tty};
use crate::xdg::Env;
use std::path::Path;

/// `bz` runner: the [`Env`] every read folds through, and nothing else — the
/// wall in that env is what decides which sphere's config, credentials and
/// model cache brazen answers over (§16.2 as amended).
#[derive(Debug, Clone)]
pub struct RealBzRunner {
    env: Env,
}

impl RealBzRunner {
    /// Build over an explicit [`Env`] — the seam the tests drive the reads
    /// through (a hermetic wall). Production folds through
    /// [`resolve`](Self::resolve).
    #[cfg(test)]
    pub(crate) fn new(env: Env) -> Self {
        Self { env }
    }

    /// The runner for the composed `world` (§16.6 W2), wall and all: brazen's
    /// three locations fold out of this `Env` directly, so every read answers
    /// the sphere the caller lensed on and never the machine's own brazen state.
    pub fn resolve(world: &Env) -> Self {
        Self { env: world.clone() }
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
        self.capture(&["--list-models", "--provider", provider, "--json"])
    }
}

#[cfg(test)]
mod tests;
