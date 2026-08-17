//! The `bl` arm — **filled by W8**: balls' own thin bin, verbatim. The mutating
//! verbs keep their subprocess (balls' change-worktree + ff-only-seal CAS and
//! its plugin chain are process-shaped, §16.7), but the process on the other
//! side is yog. Typed store *reads* never come here — they are in-process
//! ([`crate::projects::runner::BlStore`]).
//!
//! **W9 adds the identity an agent tool needs** ([`crate::world::tools`] seeds
//! the shim that reaches it): [`default_actor`] prefers `$YOG_NAME` over
//! `$USER`, so a verb the caller left unstamped claims under the workspace
//! name — §3.3's `--as` stamp, applied at the one place balls reads the
//! default rather than by rewriting argv. An explicit `--as` still wins, and
//! verbs with no `--as` are untouched. No argv parsing, no per-verb flag table.
//!
//! **The full verb surface runs (bl-2930; the W9 refusal is deleted).** balls
//! binds its sibling plugin binaries (`bl-delivery`, `bl-tracker`) from
//! `Edge::exe_dir` — so the arm converges the world's tool shims and names
//! `world/tools/bl` as the running executable ([`targets`]): `exe_dir` is the
//! tools dir, the seed's sibling rule (`exe_dir/<name>`) finds the plugin
//! shims there, and a `prime` founds a checkout whose plugin chain re-enters
//! yog ([`super::bl_delivery`]/[`super::bl_tracker`]) — the same
//! converge-on-the-way-in the lernie arm does for its re-entry targets (W11).

use balls::edge::Edge;
use std::env;
use std::io::IsTerminal;
use std::io::Write as _;
use std::path::PathBuf;

use crate::world::marks::Space;
use crate::world::tools;

use super::landing;

/// The workspace-identity env var yog stamps on every workspace-scoped
/// spawn (§8, §3.3); it rides down the whole chain (detached driver → tool
/// subprocess → the agent's bash → this shim), which is what makes it the
/// right default actor.
const YOG_NAME: &str = "YOG_NAME";

/// `yog bl <argv…>` → `balls::run`. Reproduces `bl`'s `main` exactly: the
/// host environment is read ONCE here, at the process boundary, into an
/// [`Edge`] (balls' own rule — the library does no env reads), and the exit
/// code rides back to [`super::dispatch`]. The env that is read live is the
/// **world's, and then the space's** — [`stand`] puts the process where the
/// `Edge` names before balls reads a byte (bl-81c9, bl-c21d) — so a bare `yog
/// bl` at an ambient shell reaches exactly the clones and worktrees yog's board
/// reads, an own space's `claim` cuts its worktree in that space, and a spawned
/// one re-folds the identical set. The world's tool shims are converged on the
/// way in (one read, no write, in the steady state) so a `prime` — however
/// reached — binds real sibling paths.
pub(super) fn run(args: &[String]) -> i32 {
    let probe = super::help::is_discovery(args);
    let space = stand(probe);
    let bl_exe = match targets(probe) {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "yog bl: seed world tool shims: {e}");
            return 1;
        }
    };
    let edge = edge(bl_exe, &space);
    if !probe {
        let world = crate::world::layout(&crate::xdg::Env::from_env()).root;
        landing::report(landing::converge(&edge, &world));
    }
    balls::run(&edge, args)
}

/// **Stand this process where the [`Edge`] will name**, and answer the §16.3
/// space it stood in — the arm's whole "which balls state is this?" act, in one
/// place, so the space the `Edge` carries and the state home balls' plugin
/// children read can never be two answers (bl-c21d).
///
/// Two folds, one layer apart, each closing the same hole from its own side: the
/// world ([`crate::world::inhabit`], bl-81c9) because balls' plugin chain spawns
/// `bl-delivery`/`bl-tracker`/`git` as children that resolve `$XDG_STATE_HOME`
/// out of their own env — so a bare `yog bl claim` used to cut its worktree in
/// the operator's ambient territory while sealing the ball in yog's world — and
/// then the space ([`crate::world::inhabit_space`]), because those same children
/// hold no `Edge` either, so an own space kept its store and left its worktrees
/// in the world's plugin territory.
///
/// **A discovery probe folds nothing** (bl-52ed, the reason `probe` is threaded
/// rather than re-asked): `yog bl --help` reads balls' interface, and asking what
/// a verb does must not depend on a world existing. The space is still resolved —
/// [`edge`] needs one — but from the ambient env, and balls answers from argv
/// before it resolves anything that space names.
fn stand(probe: bool) -> Space {
    if probe {
        return crate::world::marks::space(&crate::xdg::Env::from_env());
    }
    crate::world::inhabit();
    // Read AFTER the world fold: an absent `YOG_MARKS` resolves the world's own
    // space, which is the world's state home — the value just written.
    let space = crate::world::marks::space(&crate::xdg::Env::from_env());
    crate::world::inhabit_space(&space);
    space
}

/// Converge the world's tool shims and return the world's `bl` — the path
/// handed to balls as the running executable, so `Edge::exe_dir` is the
/// tools dir where the `bl-delivery`/`bl-tracker` sibling shims live
/// (bl-2930; the W11 lernie-arm mechanism). The tools dir derives from the
/// ambient anchor (`$XDG_DATA_HOME/yog/world/tools`, §16.2 — never a world
/// override, so every process in the chain resolves the same dir).
///
/// **A discovery probe converges nothing** (bl-52ed): `yog bl --help` reads
/// balls' interface, not the world, so it must not materialize six shims under
/// a fresh world root — nor fail outright before help on a read-only one. The
/// exe is still the world's `bl`; for a probe it is only *computed*, and balls
/// answers from argv before it resolves anything the path is used for. `probe`
/// is [`run`]'s one reading of that question, passed down rather than asked
/// twice.
fn targets(probe: bool) -> std::io::Result<PathBuf> {
    let dir = crate::world::layout(&crate::xdg::Env::from_env()).tools;
    if !probe {
        tools::ensure_tools(&dir)?;
    }
    Ok(dir.join(tools::BL))
}

/// The default `--as` identity for an embedded `bl` op (§16.7 W9, §3.3):
/// `$YOG_NAME` when the harness stamped one, else `$USER` (balls' own
/// default), else balls' `"unknown"`. Empty reads as absent, the env
/// convention the rest of yog follows. Pure over the two raw values so every
/// branch is testable without mutating the process env.
pub(super) fn default_actor(yog_name: Option<String>, user: Option<String>) -> Option<String> {
    yog_name.filter(|n| !n.is_empty()).or(user)
}

/// The host inputs for one embedded `bl` invocation, resolved verbatim as
/// balls' `bl` binary resolves them — except three folds, each at the one
/// place balls reads the value rather than by rewriting argv: the default
/// actor ([`default_actor`]'s `$YOG_NAME`-first fold); the executable, which
/// is the world's `bl` shim ([`targets`]) rather than `current_exe()`, so
/// the sibling-binding seam points at the world tools dir; and **balls' two
/// home directories, which come from the §16.3 space
/// ([`marks::space`](crate::world::marks::space)) rather than from
/// `$XDG_CONFIG_HOME`/`$XDG_STATE_HOME`**.
///
/// The env those three read is the world's, [`stand`] having folded it in
/// already (bl-81c9) — so `space`'s absent-`YOG_MARKS` arm resolves the
/// *world's* space and not the ambient one it used to. The `space` is taken
/// rather than re-resolved for the same reason: it is the one [`stand`] stood
/// this process in, and a second reading could answer differently (bl-c21d).
///
/// That third fold is what makes a per-agent branch possible at all, and it is
/// balls' own seam: balls' library does no env reads (its bl-bfa8 rule), so the
/// host supplies the layout — the same exit §16.2 takes for brazen's
/// credential/cache seams. Absent `YOG_MARKS` it resolves the world's space, so
/// nothing about a project-bound agent, yog's own verbs, or the board's reads
/// changes — except that balls' *config* home stops being the operator's
/// ambient `~/.config/balls` and nests with the rest of the world (§16.3's
/// module doc for what that leak cost).
fn edge(bl_exe: PathBuf, space: &Space) -> Edge {
    Edge::resolve(
        env::var_os("HOME").map(PathBuf::from).unwrap_or_default(),
        Some(space.config.to_string_lossy().into_owned()),
        Some(space.state.to_string_lossy().into_owned()),
        env::current_dir().unwrap_or_default(),
        default_actor(env::var(YOG_NAME).ok(), env::var("USER").ok()),
        env::var("BALLS_PLUGIN_DEPTH").ok(),
        Some(bl_exe),
        env::var_os("PATH"),
        env::var("NO_COLOR").ok(),
        std::io::stdout().is_terminal(),
        env::var("BALLS_CLOCK").ok(),
    )
}
