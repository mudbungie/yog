//! Binary resolution: which executable a [`Cli`] physically execs, and under
//! what leading argv (DESIGN §16.7 W12, the self-multiplex spine).
//!
//! **The one switch point.** Every namespace resolves in one of two modes, and
//! the choice lives in exactly one place — [`Binary::self_multiplexed`], a
//! per-namespace `const` (`Bl` since W8, `Bz` since W10, `Lernie` since W11 —
//! all ON):
//!
//! - **host mode** (now only via override): the physical program is the tool
//!   itself — the `*_BINARY` override path when that env var is set and
//!   non-empty (the test seam / escape hatch, always winning), else the PATH
//!   name. No argv prefix.
//! - **self-multiplex mode** (the default for all three): the physical program
//!   is yog's own `current_exe()`, prefixed with the namespace, so the spawn is
//!   `yog <namespace> <args…>` and yog dispatches it in-process. The
//!   `*_BINARY` override still wins when set.
//!
//! **Severability.** Flipping a namespace to the embedded crate is a one-line
//! edit to its `self_multiplexed` arm — no call site changes, no scattered
//! conditionals. Until an arm flips, its spawns keep resolving the host binary,
//! so nothing breaks while the multiplex arms are still stubs. That is why W12
//! ships the whole spine with the switch defaulted OFF.
//!
//! **The logical/physical split.** [`Cli::binary`] returns the *logical* name
//! ([`prefix`](Cli)`[0]` in self-mode, else the physical `program`), so every
//! ops-log argv projection (§8.2) records `["lernie", …]` whatever the physical
//! target is — the ops surface is invariant across the switch.

use std::ffi::OsString;
use std::path::PathBuf;

use super::Cli;

/// How a [`Binary`] physically resolves (§16.7 W12, bl-3ff4) — the one switch,
/// with one variant per shape the world actually has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    /// yog's own `current_exe()` under a leading verb word: `yog <ns> <args…>`,
    /// dispatched in-process by [`crate::multiplex`].
    Namespace(&'static str),
    /// yog's own `current_exe()`, bare — the binary that IS yog's argv surface.
    SelfBare,
}

/// A harness binary yog drives (DESIGN §8): the `(override var, PATH name)`
/// pair and the per-namespace self-multiplex switch live here — the one place
/// each namespace's resolution policy is stated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binary {
    Lernie,
    Bl,
    Bz,
    /// **yog itself** (bl-3ff4): the world's `yog` shim, so an agent's bash can
    /// drive yog's own headless surface (`yog gesture …`, §8.5 / VISION §4.8)
    /// without a host install — and so it reaches THIS yog rather than the
    /// operator's installed one, which drifts stale against the build under
    /// drive (bl-d1af's defect class). Not a tool yog *spawns*; it is on the
    /// roster because the roster is what the world seeds.
    Yog,
    /// balls' delivery plugin sibling (`bl-delivery`, §16.7 W9-amended /
    /// bl-2930): answered by the multiplex arm over
    /// `balls::delivery_bin::run`, so the embedded `bl prime` can bind a
    /// plugin chain that is yog itself.
    BlDelivery,
    /// balls' tracker plugin sibling (`bl-tracker`), the same seam over
    /// `balls::tracker::run`.
    BlTracker,
    /// The capability control (§8.6, VISION §4.11): the executable lernie's
    /// tool-control seam consults before every granted tool invocation. Not an
    /// agent tool — nothing types it — but it seats in the same world-tools
    /// roster, because what it must be is exactly what they must be: yog's own
    /// process, named by absolute path, so no host binary can shadow the
    /// adjudicator.
    ToolControl,
}

impl Binary {
    /// The environment-variable override and the host PATH-name default.
    const fn env_and_default(self) -> (&'static str, &'static str) {
        match self {
            Binary::Lernie => ("LERNIE_BINARY", "lernie"),
            Binary::Bl => ("BL_BINARY", "bl"),
            Binary::Bz => ("BZ_BINARY", "bz"),
            Binary::BlDelivery => ("BL_DELIVERY_BINARY", "bl-delivery"),
            Binary::BlTracker => ("BL_TRACKER_BINARY", "bl-tracker"),
            Binary::ToolControl => ("YOG_TOOL_CONTROL_BINARY", crate::control::SUBCMD),
            Binary::Yog => ("YOG_BINARY", "yog"),
        }
    }

    /// The one switch point (§16.7 W12), and it says three things rather than
    /// two (bl-3ff4). It was a `bool` — host binary or namespaced self-multiplex
    /// — until the world needed a shim for **yog itself**, whose target is yog's
    /// own executable carrying *no* verb word; two booleans would have encoded
    /// that, and two booleans admit a fourth state that means nothing. **Every
    /// tool namespace is [`Namespace`](Target::Namespace)**: `Bl` since W8
    /// (balls is linked; [`crate::multiplex`]'s `bl` arm calls `balls::run`),
    /// `Bz` since W10 (brazen is linked; the `bz` arm is [`crate::bz_host`]),
    /// `Lernie` since W11 (lernie is linked; its arm is the thin exec binding
    /// in `multiplex/lernie.rs`), and the two balls plugin siblings since
    /// bl-2930 (their arms are the promoted `delivery_bin::run` /
    /// `tracker::run` boundaries). The `*_BINARY` env overrides still win — the
    /// test seam / escape hatch back to a host binary.
    const fn target(self) -> Target {
        match self {
            Binary::Bl => Target::Namespace("bl"),
            Binary::Bz => Target::Namespace("bz"),
            Binary::Lernie => Target::Namespace("lernie"),
            Binary::BlDelivery => Target::Namespace("bl-delivery"),
            Binary::BlTracker => Target::Namespace("bl-tracker"),
            Binary::ToolControl => Target::Namespace(crate::control::SUBCMD),
            // yog IS the argv surface, so its shim carries no verb word — a
            // prefix here would spawn `yog yog …`, which routes nowhere.
            Binary::Yog => Target::SelfBare,
        }
    }
}

impl Cli {
    /// A host-mode `Cli` over `binary` taken verbatim (program == binary, no
    /// prefix, no standing env) — the base constructor for tests, the `yog
    /// exec` hatch, and [`resolve`](Self::resolve)'s override/PATH arms.
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Self {
            program: binary.into(),
            prefix: Vec::new(),
            env: Vec::new(),
            wrapper: Vec::new(),
        }
    }

    /// A self-multiplex `Cli` (§16.7 W12): the physical program is yog's own
    /// `current_exe`, prefixed with `namespace`, so the spawn is `yog
    /// <namespace> <args…>` while [`binary`](Self::binary) still reports the
    /// logical `namespace` for the ops log.
    fn self_target(current_exe: PathBuf, namespace: &str) -> Self {
        Self {
            program: current_exe,
            prefix: vec![namespace.to_string()],
            env: Vec::new(),
            wrapper: Vec::new(),
        }
    }

    /// Resolve `binary` from the ambient environment (override var if set and
    /// non-empty, else the switch-gated default) — wraps
    /// [`resolve_with`](Self::resolve_with) over `std::env`. No standing env;
    /// [`resolve_in_world`](Self::resolve_in_world) adds it.
    pub fn resolve(binary: Binary) -> Self {
        Self::resolve_with(
            binary,
            |k| std::env::var_os(k),
            std::env::current_exe().ok(),
        )
    }

    /// Resolve `binary` and stand the world's nesting `overrides` (§16.2,
    /// [`world::overrides`](crate::world::overrides)) on every spawn, so each
    /// child nests in yog's world (§16.6 W2 / §16.4's correctness argument). The
    /// overrides are opaque pairs — this crate stays generic, knowing nothing of
    /// *which* vars nest (that lives in [`crate::world`]).
    pub fn resolve_in_world(binary: Binary, overrides: &[(String, String)]) -> Self {
        Self::resolve(binary).with_env(overrides.to_vec())
    }

    /// Resolution over injected `lookup` (name → value) and `current_exe` — the
    /// seam that tests every branch without `std::env::set_var` (`unsafe` in
    /// edition 2024). The override wins whenever set and non-empty; otherwise
    /// [`default_target`](Self::default_target) applies the per-namespace
    /// switch. `pub(crate)` — driven by the `resolve_with` unit tests (also the
    /// `tests/` recorder's production-wiring note).
    pub(crate) fn resolve_with(
        binary: Binary,
        lookup: impl Fn(&str) -> Option<OsString>,
        current_exe: Option<PathBuf>,
    ) -> Self {
        let (env_var, default) = binary.env_and_default();
        match lookup(env_var) {
            Some(v) if !v.is_empty() => Self::new(PathBuf::from(v)),
            _ => Self::default_target(default, binary.target(), current_exe),
        }
    }

    /// The default (no-override) resolution, gated by the per-namespace switch
    /// [`Binary::self_multiplexed`] (§16.7 W12): OFF → the host PATH-name
    /// binary (no namespace is OFF any more — W8/W10/W11 flipped all three); ON →
    /// yog's own `current_exe` under a `[namespace]` prefix (`yog <namespace>
    /// …`). The switch survives its migration as the severability seam. `current_exe`
    /// unavailable falls back to the host name — a spawn that still names the
    /// tool, never a panic. `pub(crate)`: the resolution seam the
    /// `resolve_with`/self-multiplex unit tests drive without ambient-env or
    /// `current_exe()` mutation.
    pub(crate) fn default_target(
        default: &str,
        target: Target,
        current_exe: Option<PathBuf>,
    ) -> Self {
        match (target, current_exe) {
            (Target::Namespace(ns), Some(exe)) => Self::self_target(exe, ns),
            // yog's own argv surface, carrying no verb word (bl-3ff4).
            (Target::SelfBare, Some(exe)) => Self::new(exe),
            _ => Self::new(PathBuf::from(default)),
        }
    }
}
