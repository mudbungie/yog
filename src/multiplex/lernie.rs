//! The `lernie` arm — **filled by W11** (DESIGN §16.7): lernie's own thin exec
//! binding (`src/bin/lernie/main.rs` upstream), reproduced in yog's process.
//! `yog lernie <argv…>` IS `lernie <argv…>`: parse the shared `cmd::Cli`, run
//! the §2.9 binding preludes the parsed verb names (`Command::preludes` —
//! pgid leadership, the SIGTERM stop handler), build the [`Fx`] injections,
//! invoke `Command::run`, and perform the returned [`Outcome`] — print the one
//! product, `exec` the successor, or map the tool exit code. Process semantics
//! are identical to the host binary's (§16.5: linking changes what code yog
//! calls, never the concurrency model): same pgid leadership, same flock lease
//! and `LERNIE_LOCK_FD` adopt, same `execve` baton — the process image at every
//! hop is yog.
//!
//! **The re-entry targets are the world's shims, not a bare exe.**
//! `Fx::driver_target` and `Fx::adapter_target` are single paths lernie spawns
//! *verbatim* (`<driver_target> advance …`, `<adapter_target> generate …`), so
//! yog's own executable cannot stand there — bare, it would drop the namespace
//! word and the argv would fall through to the GUI. The arm instead converges
//! the world's `lernie` and `bz` shims ([`tools::ensure_shim`], the W9 `bl`
//! pattern: a `/bin/sh` re-exec of yog under the namespace, generated from the
//! same [`Cli::exec_words`] yog's own spawns use) and names those. Ensured here,
//! on the way into every verb, so the very first `yog lernie prompt` — however
//! reached — already has valid re-entry targets; a byproduct is that the world's
//! `PATH` now carries `lernie` and `bz` beside `bl`, each answering from the
//! embedded crate.

use std::io;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ::lernie::cmd::{self, Command, Fx, Outcome, prelude};
use clap::Parser as _;

use crate::cli_outbound::{Binary, Cli};
use crate::tool_host;
use crate::ui_state::SystemClock;
use crate::world::tools;

/// One `lernie` invocation, exactly as the upstream exec binding performs it.
/// The host environment is read here, at the process boundary (`$EDITOR`, the
/// world layout's ambient anchor), never inside the surface — lernie's own
/// §3.4 rule ("process effects stay at the binding"), kept by the arm that
/// replaces the binding. Folding the world into that environment is the same
/// rule's other half ([`crate::world::inhabit`], bl-81c9): the binding is the
/// only place that can put a linked lernie in the nested world, because every
/// root lernie resolves comes from its own `getenv`.
pub(super) fn run(args: &[String]) -> i32 {
    let cli = match parse(args) {
        Ok(cli) => cli,
        Err(code) => return code,
    };
    // The process STANDS in the world before lernie resolves anything
    // (bl-81c9, §16.2): `LERNIE_HOME` is read by the linked lernie's own
    // `getenv` (`harness_root::resolve`) and there is no injection seam for it
    // — so a bare `yog lernie prime` seeded the operator's ambient harness
    // root instead of the world's. It also puts the tool injection below and
    // every tool subprocess lernie fires on the same `<world>/state`. Placed
    // after the parse, so a `--help`/`--version` probe — which clap answers
    // above — still touches no world.
    crate::world::inhabit();
    // The §2.9 binding preludes, before the surface is invoked. Which ones the
    // verb needs is lernie's own tested map (`Command::preludes`); performing
    // them is this binding's act.
    cli.command.preludes().iter().for_each(|p| p());
    let (driver_target, adapter_target) = match targets() {
        Ok(t) => t,
        Err(e) => {
            let _ = writeln!(io::stderr(), "yog lernie: seed world tool shims: {e}");
            return 1;
        }
    };
    // `$EDITOR` resolved once, at the binding (lernie's `cli::edit_in_editor`
    // reads it at spawn time; same value, earlier read).
    let editor_cmd = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let editor = move |dir: &Path| edit_with(&editor_cmd, dir);
    // **yog's tool injection** (REMOTE §5, bl-c907), built here for the same
    // reason `$EDITOR` is: the state root is a process fact, read at the
    // binding and never inside the surface. It resolves to the very directory
    // the engine writes — the world hands `XDG_STATE_HOME` down to every child
    // (§16.2), so parent and child fold to one `<world>/state/yog`.
    let injection = tool_host::Injection::new(
        crate::xdg::Env::from_env().yog_state_root(),
        tool_host::ask::Budget::default(),
        tool_host::remote::patience(),
        Arc::new(SystemClock),
        driving(&cli.command),
    );
    // Stdio is locked for the whole verb (the `tool` verb writes raw bytes into
    // it) and released before the outcome is performed — holding the lock
    // across the product `writeln!` would deadlock.
    let result = {
        let mut stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();
        let mut stderr = io::stderr().lock();
        let mut fx = Fx {
            driver_target,
            adapter_target: Some(adapter_target),
            editor: &editor,
            tool_stdin: &mut stdin,
            tool_stdout: &mut stdout,
            tool_stderr: &mut stderr,
            stop: prelude::stop_flag(),
            tool_injection: Some(&injection),
        };
        cli.command.run(&mut fx)
    };
    conclude(result)
}

/// Parse a `lernie` argv (the args after the namespace word) into the shared
/// clap surface, `Err` carrying the process exit code: clap prints its own
/// help/usage/error rendering (`--help` is exit 0, an unknown verb its usage
/// error), exactly as the upstream binding's `Cli::parse` would.
fn parse(args: &[String]) -> Result<cmd::Cli, i32> {
    let argv = std::iter::once("lernie".to_owned()).chain(args.iter().cloned());
    cmd::Cli::try_parse_from(argv).map_err(|e| {
        let _ = e.print();
        e.exit_code()
    })
}

/// The `(workspace name, agent id)` this invocation drives, when its verb names
/// one (REMOTE §5, bl-c907). `advance` is the driver verb and the only one that
/// names an existing agent; `prompt` and `dispatch` *mint* their agent, and an
/// agent that does not exist yet has loaded nothing — so `None` is the honest
/// answer rather than a case. It feeds
/// [`Injection::tools`](crate::tool_host::Injection), which the seam gives no
/// per-call context (lernie's `docs/DESIGN_TOOL_INJECTION.md` §7: the injection
/// is per-process, not per-agent).
fn driving(command: &Command) -> Option<(String, String)> {
    match command {
        Command::Advance(args) => Some((crate::naming::leaf(&args.workspace), args.agent.clone())),
        _ => None,
    }
}

/// Converge the world's `lernie`/`bz` re-exec shims and return them as the
/// `(driver_target, adapter_target)` pair (§16.7 W11; module doc). The tools
/// dir derives from the ambient anchor (`$XDG_DATA_HOME/yog/world/tools`,
/// §16.2 — the anchor is never a world override, so every process in the
/// driver chain resolves the same dir); the shim content derives from the same
/// [`Cli`] resolution yog's own spawns use, `*_BINARY` test seams included.
fn targets() -> io::Result<(PathBuf, PathBuf)> {
    let dir = crate::world::layout(&crate::xdg::Env::from_env()).tools;
    Ok((
        tools::ensure_shim(&dir, tools::LERNIE, &Cli::resolve(Binary::Lernie))?,
        tools::ensure_shim(&dir, tools::BZ, &Cli::resolve(Binary::Bz))?,
    ))
}

/// The `lernie config` `$EDITOR` hand-off, verbatim from the upstream binding:
/// `$EDITOR` may carry arguments, so it runs through `sh -c`, and a non-zero
/// editor exit is a failed edit. yog's own §9.3 config drive rides this exact
/// seam — the `EDITOR` it stands on the spawn is `<yog> --editor-apply`, so
/// the editor spawned here re-enters yog's apply shim.
fn edit_with(editor: &str, dir: &Path) -> io::Result<()> {
    let status = crate::git_env::command(Path::new("sh"))
        .arg("-c")
        .arg(format!("exec {editor} \"$1\""))
        .arg("sh")
        .arg(dir)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("editor exited with {status}")))
    }
}

/// Map a verb's result to the process exit code: perform an [`Outcome`], or
/// print the uniform failure (`lernie <verb-prefix>: <error>`) and fail — the
/// upstream binding's tail, with `i32` in place of `ExitCode`.
fn conclude(result: Result<Outcome, cmd::Error>) -> i32 {
    match result {
        Ok(outcome) => perform(outcome),
        Err(e) => {
            let _ = writeln!(io::stderr(), "{e}");
            1
        }
    }
}

/// Perform a verb's [`Outcome`]: print the one product, do nothing, `exec` the
/// successor (a successful `execve` never returns — reaching past it is the
/// failure path), or map the tool exit code (rides within `u8`, POSIX).
fn perform(outcome: Outcome) -> i32 {
    match outcome {
        Outcome::Line(line) => {
            let _ = writeln!(io::stdout(), "{line}");
            0
        }
        Outcome::Quiet => 0,
        Outcome::Exec(mut command) => {
            use std::os::unix::process::CommandExt as _;
            let _ = writeln!(
                io::stderr(),
                "lernie advance: exec successor: {}",
                command.exec()
            );
            1
        }
        Outcome::Code(code) => i32::from(code),
    }
}

#[cfg(test)]
mod tests;
