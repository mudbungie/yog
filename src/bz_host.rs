//! The embedded brazen host (DESIGN §16.7 W10) — brazen's `bz` binary, in
//! yog's process.
//!
//! yog links `brazen` at an exact pin with the `native-host` feature, which
//! re-exposes the impure shim the `bz` bin owns (`brazen::native`: the rustls
//! `ureq` transport, the XDG credential store and model cache, the loopback
//! login receiver, the system clock/browser/RNG). This module is the *only*
//! place that shim is wired, and it reproduces `bz`'s own `main.rs` verbatim:
//! build [`brazen::Args`], read the route off argv with brazen's authoritative
//! [`brazen::route`], and hand each route its seam bundle.
//!
//! **One entry, two callers.** [`run`] takes its stdio as injected writers, so
//! the same wiring serves both consumers with no second code path:
//!
//! - [`crate::multiplex`]'s `bz` arm — `yog bz <argv…>` — passes the real
//!   process stdio and [`Tty::probe`]. That is `bz` the command, whichever
//!   process image invokes it.
//! - [`RealBzRunner`](crate::config_edit::brazen::RealBzRunner) passes an empty
//!   stdin and two byte buffers, so the config projection (`--dump-config`, the
//!   `--list-providers` table) is a *function call* — no spawn, no pipe, no
//!   parse of a foreign binary's stdout.
//!
//! **What stays out of here** (brazen's own note on the exposure): the
//! process-global effects of `bz`'s `main` do not lift. yog never resets
//! SIGPIPE — the disposition is `SIG_IGN`, so a closed stdout surfaces as a
//! `BrokenPipe` write error that brazen's own `ExitClass::from_io` maps to the
//! same 141, and restoring it would need an `unsafe libc::signal` outside the
//! one sanctioned site (AGENTS.md rule 3). The two isatty facts are read
//! through safe `std::io::IsTerminal` instead of `libc::isatty`.
//!
//! **Env — the wall, not the machine (§16.2 as amended by the blast-radius ruling).** All three
//! of brazen's locations resolve inside the focused workspace's wall
//! ([`crate::world::wall`]), folded once by
//! [`BrazenPaths`](crate::config_edit::brazen::BrazenPaths) off the caller's
//! [`Env`]: the config path is *injected* into the [`brazen::EnvSnapshot`] this
//! module builds (so brazen's own fold can never fall through to
//! `$XDG_CONFIG_HOME`), and the credential store and model cache are yog's own
//! wall-rooted seams ([`store`]) rather than brazen's process-env ones. One
//! fold, three consumers — the pane's presence read, the in-process call, and
//! the spawned `yog bz` on the far side of a fork all name the same files.
//!
//! **No wall, no `bz`.** A seat inside no workspace has no providers,
//! credentials or model cache to read, so [`run`] refuses with
//! [`NO_WALL_CODE`] instead of falling back to the machine's own brazen state.
//! Every yog surface guards before it calls; a spawned `bz` inherits the wall
//! from the loop that fired (§16.2's inheritance note), so the refusal is only
//! ever reached by a `bz` invoked outside any workspace.

use std::io::{IsTerminal as _, Read, Write};

use crate::config_edit::brazen::BrazenPaths;
use crate::xdg::Env;

mod routes;
pub(crate) mod store;
use routes::Seams;

#[cfg(test)]
mod tests;

/// brazen's own config selector, injected into the snapshot from the wall.
const BRAZEN_CONFIG: &str = "BRAZEN_CONFIG";

/// The exit code a `bz` invocation outside any workspace wall answers with —
/// brazen's own usage class (64), because naming no wall is exactly a usage
/// error: the invocation is well-formed but addresses no sphere.
pub(crate) const NO_WALL_CODE: i32 = 64;

/// The message that refusal prints, naming the fix rather than the fault.
const NO_WALL_MSG: &str = "bz: no workspace in this environment — providers, sign-ins and the model \
     cache belong to a workspace, and there is nothing shared to fall back to. \
     Run this inside a yog workspace, or focus one in yog.\n";

/// The wall root a **discovery probe** outside any workspace stands on
/// (bl-52ed). `--help`/`--skill`/`--version` are answered by `brazen::run`
/// *before* it reads a config file or touches a seam, so the three paths folded
/// from this root are provably unread — and an absolute path that cannot exist
/// keeps that provable rather than merely intended: were the invariant ever to
/// break, the read finds nothing instead of finding the machine's own brazen
/// state, which is the one thing §16.2 forbids.
const NO_WALL_PROBE_ROOT: &str = "/nonexistent/yog-no-wall";

/// The two isatty facts `bz`'s shim injects into [`brazen::Args`] — the only
/// terminal knowledge the pure library cannot observe for itself. Carried as
/// one value so [`run`] keeps a narrow parameter list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tty {
    /// `isatty(0)`: an interactive stdin is treated as *absent* input, so a
    /// no-positional invocation prints usage instead of blocking forever.
    pub(crate) stdin: bool,
    /// `isatty(1)`: gates brazen's pretty text skin.
    pub(crate) stdout: bool,
}

impl Tty {
    /// Neither stream is a terminal — every in-process call yog makes, whose
    /// stdio is a byte buffer by construction.
    pub(crate) const PIPED: Tty = Tty {
        stdin: false,
        stdout: false,
    };

    /// Probe the real process stdio through safe `std::io::IsTerminal` (the
    /// `libc::isatty` of `bz`'s shim, minus the `unsafe`).
    pub(crate) fn probe() -> Tty {
        Tty {
            stdin: std::io::stdin().is_terminal(),
            stdout: std::io::stdout().is_terminal(),
        }
    }
}

/// Run one `bz` invocation in-process and return its exit code. `argv` is
/// brazen's argv **without** a program name (what `yog bz …` slices off, and
/// what `std::env::args().skip(1)` gives `bz` itself).
pub(crate) fn run(
    argv: Vec<String>,
    env: &Env,
    tty: Tty,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    // The wall gate, with the one exemption §8.5 requires: a **discovery
    // probe** — an argv that is exactly `--help`/`-h`/`--version`/`-V`/
    // `--skill` — asks brazen what it is, which reads the interface and never
    // the world, so it must answer outside a workspace too (bl-52ed: the top
    // level advertises `yog bz --login` as the sign-in path, and a `--help`
    // that refuses cannot say so). Every other route genuinely needs the wall
    // and is refused without one, credentials never falling back ambiently.
    let paths = match BrazenPaths::of(env) {
        Some(paths) => paths,
        None if crate::multiplex::help::is_discovery(&argv) => {
            BrazenPaths::in_wall(std::path::Path::new(NO_WALL_PROBE_ROOT))
        }
        None => {
            drop(stderr.write_all(NO_WALL_MSG.as_bytes()));
            return NO_WALL_CODE;
        }
    };
    // The wall's config path replaces whatever `BRAZEN_CONFIG` the snapshot
    // carried: brazen's own fold ends at `$XDG_CONFIG_HOME/brazen/config.toml`,
    // and an inherited ambient value would silently collapse every workspace
    // onto one file — the exact thing the ruling forbids.
    let mut vars: std::collections::BTreeMap<String, String> = env.pairs().into_iter().collect();
    vars.insert(
        BRAZEN_CONFIG.to_owned(),
        paths.config.to_string_lossy().into_owned(),
    );
    let args = brazen::Args {
        argv,
        env: brazen::EnvSnapshot(vars),
        tty: tty.stdin,
        stdout_tty: tty.stdout,
    };
    // Route on the control flag, never argv[0] — brazen's own `route`, so the
    // host and the library can never disagree about what an argv means.
    let seams = Seams::wire(&paths, env);
    let code = match brazen::route(&args.argv) {
        brazen::Route::Login => routes::login(&args, &seams, stdout, stderr),
        brazen::Route::ListModels => routes::list_models(&args, &seams, stdout, stderr),
        brazen::Route::ListProviders => routes::list_providers(&args, &seams, stdout, stderr),
        brazen::Route::CountTokens => routes::count_tokens(&args, &seams, stdin, stdout, stderr),
        brazen::Route::Serve => routes::serve(&args, &seams, stdout, stderr),
        brazen::Route::Run => routes::data_plane(args, &seams, stdin, stdout, stderr),
    };
    i32::from(code)
}
