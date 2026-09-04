//! The world's **agent tools** — `<yog-data-root>/world/tools/` (DESIGN §16.4,
//! §16.7 W9). This module owns that directory end to end: the shim files yog
//! seeds into it and the `PATH` entry that makes them the tools an agent finds.
//!
//! **The surface an agent actually uses is bash.** §16.4's premise — "binaries
//! matter only as the plane on which litany's worker agents drive tools by
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
//!    `litany prompt`, its driver, the driver's `bash` tool, the shim — inherits
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
//! loses nothing. Convergence is not unconditional, though: a rewrite is only
//! ever an improvement if the new target is one that can be exec'd, so
//! [`ensure_shim`] refuses a resolution that is not an absolute path (bl-f558)
//! rather than replacing a working shim with a broken one. It is *not* the `litany-tool-<name>` external-tool slot: that
//! is litany's JSON-stdin tool contract under `$LITANY_HOME/tools/`, needs a
//! schema plus a `SKILL.md` plus a role `tools:` entry, and speaks nothing like
//! bl's argv (see §16.4's amended note).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::cli_outbound::{Binary, Cli};
use crate::world::hatch::shell_quote;

/// The agent-tool namespace whose shim the start seeds (§16.7 W9): the `bl` an
/// agent's bash finds. Seeded at `Step::EnsureSeeded`, before any driver exists
/// to run one.
pub const BL: &str = "bl";

/// The `litany` shim namespace (§16.7 W11). Converged by the multiplex arm on
/// the way into every `yog litany` verb — it is `Fx::driver_target`, the single
/// path every litany re-entry (the detached `advance` launch, the §6 successor
/// `execve`, the tool resolver's third hop) spawns verbatim, so it must exist
/// before the first driver runs and must carry the namespace word a bare yog
/// exe cannot.
pub const LITANY: &str = "litany";

/// The `bz` shim namespace (§16.7 W11): `Fx::adapter_target`, the provider
/// adapter a linked litany spawns — the same single-path constraint as
/// [`LITANY`], answered by the W10 `bz` arm. Converged beside it.
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
/// executable litany's tool-control seam consults before every granted tool
/// invocation. Not an agent tool — no agent types it, and litany spawns it with
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
    (LITANY, Binary::Litany),
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

/// [`ensure_tools`] over an ambient env, **warned rather than fatal** — the one
/// call every face that hands the world out makes before it does (the two §8.4
/// hatches and the §8.5 windowless face). Here rather than once per caller
/// because it is one policy: stdout is the hatch's one product, so a converge
/// failure is said on stderr and the caller carries on — the world still works
/// for every command that needs no shim.
pub fn seed(ambient: &crate::xdg::Env) {
    if let Err(e) = ensure_tools(&crate::world::layout(ambient).tools) {
        eprintln!("yog: seed world tools: {e}");
    }
}

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
///
/// **A shim's target must be an absolute path, and a resolution that is not one
/// refuses the write** (bl-f558). This is the durable half of the self-exe
/// invariant ([`crate::cli_outbound::self_exe`]): when yog cannot say which
/// file it is, [`Cli::resolve`] falls back to the bare PATH *name* of the tool,
/// and that name is a catastrophe to persist here rather than a degradation —
/// the world's `PATH` is fronted by this very directory ([`prepend_path`]), so
/// `exec 'bl' "$@"` re-resolves to the shim itself and spins, and for a
/// namespace a host binary DOES answer it silently runs the operator's
/// installed tool instead of yog's (§16.4's (b), bl-d1af's defect class). A
/// convergent artifact's honest answer to "I do not know what to write" is to
/// leave the last good file alone and say so: the caller surfaces it — `yog
/// env`/`yog exec` warn on stderr, and a Start fails with the reason rather
/// than seeding an adjudicator that cannot run.
///
/// **The write itself is a child's** ([`crate::git_env::write_exec`], bl-e6c9).
/// This is the engine's own ETXTBSY window and not a test's: yog seeds a shim
/// and then execs it, and yog forks from every thread, so an `fs::write` here
/// held a descriptor any peer fork could copy into the exec that follows —
/// measured at 7 failures over 1,120 runs of bl-fd28's stress filter with these
/// beats folded in, and zero once the descriptor moved. A retry on ETXTBSY was
/// rejected: a hazard must not become a production loop.
pub fn ensure_shim(tools: &Path, namespace: &str, cli: &Cli) -> io::Result<PathBuf> {
    let path = tools.join(namespace);
    let words = cli.exec_words();
    let target = Path::new(words.first().map_or("", String::as_str));
    if !target.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "world tool `{namespace}`: refusing to write a shim naming \
                 `{}` — a shim target must be an absolute path, and yog could \
                 not resolve its own executable",
                target.display()
            ),
        ));
    }
    let want = shim_script(namespace, &words);
    if fs::read_to_string(&path).is_ok_and(|have| have == want) {
        return Ok(path);
    }
    fs::create_dir_all(tools)?;
    crate::git_env::write_exec(&path, &want)?;
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
