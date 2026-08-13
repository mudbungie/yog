//! The world's **agent tools** — `<yog-data-root>/world/tools/` (DESIGN §16.4,
//! §16.7 W9). This module owns that directory end to end: the shim files yog
//! seeds into it and the `PATH` entry that makes them the tools an agent finds.
//!
//! **The surface an agent actually uses is bash.** §16.4's premise — "binaries
//! matter only as the plane on which lernie's worker agents drive tools by
//! bash" — decides the mechanism: a worker types `bl close bl-1a2b` in its
//! `bash` tool, so `bl` must *resolve* to yog's embedded balls, not to whatever
//! host binary sits on the ambient `PATH`. Hence two facts, both here:
//!
//! 1. [`ensure_shim`] seeds `<world>/tools/bl` — a `/bin/sh` re-exec of yog's
//!    own executable under the `bl` namespace (`yog bl "$@"`, the multi-call
//!    pattern of `--editor-apply` / `yog env`, §8.4). It is written from the
//!    [`Cli`] yog itself spawns ([`Cli::exec_words`]), so the shim and yog's own
//!    `bl` spawns cannot name different targets — one fact, one home.
//! 2. [`prepend_path`] puts that directory in front of the ambient `$PATH` in
//!    the world override set (§16.2), so every child of yog — the detached
//!    `lernie prompt`, its driver, the driver's `bash` tool, the shim — inherits
//!    a `PATH` on which the world's `bl` wins.
//!
//! **Identity rides the env, not the argv.** The shim passes the caller's
//! arguments through verbatim; the `--as` stamp §3.3 promises is applied where
//! balls actually reads it — the embedded arm's `Edge::default_actor`, which
//! prefers `$YOG_NAME` over `$USER` ([`crate::multiplex`]). An explicit `--as`
//! still wins, exactly as it would have over an argv-appended flag, and verbs
//! that take no `--as` are untouched. That is why nothing here parses argv.
//!
//! **Convergent, never authoritative.** The shim is a generated artifact (§5.2):
//! `ensure_shim` rewrites it whenever its content differs — a reinstalled yog at
//! a new path converges on the next start — and deleting the whole directory
//! loses nothing. It is *not* the `lernie-tool-<name>` external-tool slot: that
//! is lernie's JSON-stdin tool contract under `$LERNIE_HOME/tools/`, needs a
//! schema plus a `SKILL.md` plus a role `tools:` entry, and speaks nothing like
//! bl's argv (see §16.4's amended note).

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::cli_outbound::{Binary, Cli};
use crate::world::hatch::shell_quote;

/// The agent-tool namespace whose shim the start seeds (§16.7 W9): the `bl` an
/// agent's bash finds. Seeded at `Step::EnsureSeeded`, before any driver exists
/// to run one.
pub const BL: &str = "bl";

/// The `lernie` shim namespace (§16.7 W11). Converged by the multiplex arm on
/// the way into every `yog lernie` verb — it is `Fx::driver_target`, the single
/// path every lernie re-entry (the detached `advance` launch, the §6 successor
/// `execve`, the tool resolver's third hop) spawns verbatim, so it must exist
/// before the first driver runs and must carry the namespace word a bare yog
/// exe cannot.
pub const LERNIE: &str = "lernie";

/// The `bz` shim namespace (§16.7 W11): `Fx::adapter_target`, the provider
/// adapter a linked lernie spawns — the same single-path constraint as
/// [`LERNIE`], answered by the W10 `bz` arm. Converged beside it.
pub const BZ: &str = "bz";

/// The `bl-delivery` shim namespace (bl-2930): balls' delivery plugin sibling.
/// Not an agent-facing tool — it exists so the embedded `bl prime` can bind
/// the checkout's plugin chain to absolute paths that ARE yog: the `bl` arm
/// hands balls `world/tools/bl` as its executable, so `Edge::exe_dir` is this
/// directory and the seed's sibling rule (`exe_dir/<name>`) finds these shims.
pub const BL_DELIVERY: &str = "bl-delivery";

/// The `bl-tracker` shim namespace (bl-2930): balls' tracker plugin sibling,
/// bound by the same seed rule as [`BL_DELIVERY`].
pub const BL_TRACKER: &str = "bl-tracker";

/// The capability control's shim namespace (§8.6, VISION §4.11, bl-fec8): the
/// executable lernie's tool-control seam consults before every granted tool
/// invocation. Not an agent tool — no agent types it, and lernie spawns it with
/// no argv at all — but it belongs to this roster for the roster's own reason:
/// the authored `tool_control:` block names it by **absolute path**, so the
/// adjudicator cannot be shadowed by a host binary the way an unqualified name
/// could be, and it is regenerated on drift exactly like the tools beside it.
pub const TOOL_CONTROL: &str = crate::control::SUBCMD;

/// **yog's own shim namespace** (bl-3ff4): the `yog` an agent's bash finds, so
/// the §8.5 control boundary — every operator gesture, headlessly — is reachable
/// from inside the world. It is the odd entry twice over, and both oddities are
/// the point: its shim carries **no verb word** (yog is the argv surface, not a
/// namespace of it), and yog does not *spawn* it — the roster is simply what the
/// world seeds. Without it the world's `PATH` hands an agent every substrate
/// tool except the one that drives yog, and a clean room has no host `yog` to
/// fall through to; where a host one exists the fallthrough is worse than the
/// absence, silently resolving the operator's INSTALLED yog rather than the
/// build under drive (bl-d1af's defect class).
pub const YOG: &str = "yog";

/// The capability control shim's absolute path under a tools dir — the value the
/// authored template block carries. Pure; [`ensure_control`] is its effectful
/// half.
pub fn control_path(tools: &Path) -> PathBuf {
    tools.join(TOOL_CONTROL)
}

/// Converge the capability control shim alone, returning its path (§8.6). The
/// start flow needs both halves of this — the shim on disk *and* its path to
/// author into the template — before the first driver exists to be adjudicated.
pub fn ensure_control(tools: &Path) -> io::Result<PathBuf> {
    ensure_shim(tools, TOOL_CONTROL, &Cli::resolve(Binary::ToolControl))
}

/// The world's full tool roster — the single source of truth for which shims
/// the tools dir carries, consumed by [`ensure_tools`]. Each entry pairs the
/// shim's on-`PATH` name with the [`Binary`] whose resolution (namespace or
/// `*_BINARY` override) writes its body. The first three are the agent tools;
/// the next two are balls' sibling plugin binaries (bl-2930), carried so a
/// checkout primed by the embedded `bl` runs a plugin chain that is yog; then
/// the capability control (bl-fec8), which nothing types but everything is
/// adjudicated by; and last [`YOG`] itself (bl-3ff4), the boundary an agent
/// drives yog through.
pub const ROSTER: [(&str, Binary); 7] = [
    (BL, Binary::Bl),
    (LERNIE, Binary::Lernie),
    (BZ, Binary::Bz),
    (BL_DELIVERY, Binary::BlDelivery),
    (BL_TRACKER, Binary::BlTracker),
    (TOOL_CONTROL, Binary::ToolControl),
    (YOG, Binary::Yog),
];

/// Converge the whole world tools dir (§8.4, bl-44a5): every [`ROSTER`] shim,
/// each resolved exactly as yog's own spawns resolve it. The tools dir is a
/// generated artifact of **the world**, not of the start flow — the world's
/// `PATH` override names it unconditionally, so every place the world is
/// handed out must converge it. The escape hatches (`yog env` / `yog exec`)
/// call this before printing/spawning; before it existed, a pre-first-Start
/// hatch handed out a `PATH` fronted by an empty dir, and a bare `bl` fell
/// through to a host binary (or died in a clean room). Idempotent — one read
/// and no write per shim in the steady state, like [`ensure_shim`].
pub fn ensure_tools(tools: &Path) -> io::Result<()> {
    for (namespace, binary) in ROSTER {
        ensure_shim(tools, namespace, &Cli::resolve(binary))?;
    }
    Ok(())
}

/// Shim permissions: executable by all, writable only by the owner.
const SHIM_MODE: u32 = 0o755;

/// The shim's body: a `/bin/sh` re-exec of `exec_words` (yog's own executable
/// plus the namespace prefix, [`Cli::exec_words`]) with the caller's argv passed
/// through verbatim. Every word is [`shell_quote`]d, so a yog installed under a
/// path with spaces or quotes still execs. Pure.
pub fn shim_script(namespace: &str, exec_words: &[String]) -> String {
    let words: Vec<String> = exec_words.iter().map(|w| shell_quote(w)).collect();
    format!(
        "#!/bin/sh\n\
         # yog-generated: the world's `{namespace}`, dispatching to\n\
         # yog's embedded substrate against the nested world roots. Regenerated\n\
         # whenever it drifts; safe to delete.\n\
         exec {} \"$@\"\n",
        words.join(" "),
    )
}

/// Seed (or converge) `<tools>/<namespace>` as the re-exec shim of `cli`
/// (§16.7 W9), returning its path. Idempotent: an identical file on disk is left
/// untouched, so the common start does one read and no write; any drift (a
/// reinstalled yog, a hand-edit) is overwritten. The directory is created on the
/// way in — the general path with the tree absent, not a bootstrap branch (§3.4).
pub fn ensure_shim(tools: &Path, namespace: &str, cli: &Cli) -> io::Result<PathBuf> {
    let path = tools.join(namespace);
    let want = shim_script(namespace, &cli.exec_words());
    if fs::read_to_string(&path).is_ok_and(|have| have == want) {
        return Ok(path);
    }
    fs::create_dir_all(tools)?;
    fs::write(&path, &want)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(SHIM_MODE))?;
    Ok(path)
}

/// The world's `PATH` value (§16.2): the tools dir in **front** of the ambient
/// search path, so the world's shims win over any host binary of the same name.
/// Idempotent — a `PATH` already led by the tools dir is returned unchanged, so
/// re-composing the overrides (`yog env` inside a world shell, or the world
/// `Env` handed back to [`overrides`](super::overrides)) never stacks duplicate
/// entries. An absent or empty ambient `PATH` yields the tools dir alone. Pure.
pub fn prepend_path(tools: &Path, ambient: Option<String>) -> String {
    let dir = tools.to_string_lossy().into_owned();
    match ambient {
        Some(path) if !path.is_empty() => {
            if path.split(':').next() == Some(dir.as_str()) {
                path
            } else {
                format!("{dir}:{path}")
            }
        }
        _ => dir,
    }
}

#[cfg(test)]
mod tests;
