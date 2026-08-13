//! The self-multiplex spine (DESIGN §16.7 W12). yog's own executable is the
//! physical target of every embedded-tool spawn: a `lernie`/`bl`/`bz` leading
//! verb — `yog lernie <argv…>`, `yog bl <argv…>`, `yog bz <argv…>` — dispatches
//! here, to the arm that calls the embedded crate's entrypoint exactly as each
//! upstream's own thin bin does (all three filled: W8/W10/W11), and so do
//! balls' two sibling plugin binaries — `yog bl-delivery <op> <phase>`,
//! `yog bl-tracker <op> <phase>` (bl-2930, the U-balls-3 seam) — spawned by
//! the embedded balls' own plugin chain through the `world/tools/` shims a
//! `yog bl prime` binds. Everything else — no args, `--editor-apply`,
//! `yog env`/`yog exec`, the GUI — is not a namespace and falls through
//! ([`dispatch`] returns `None`).
//!
//! **How a wave fills an arm.** Each namespace has one function, `fn run(args:
//! &[String]) -> i32`. A wave lands by replacing exactly that one function's
//! body with the crate call — **W8 filled [`bl::run`]** (`balls::run(&edge,
//! args)`), **W10 filled [`bz::run`]** ([`crate::bz_host`], brazen's own `main`
//! in this process), and **W11 filled [`lernie::run`]** (lernie's own thin exec
//! binding: parse `cmd::Cli`, run `Command::preludes` + `Command::run`, perform
//! the `Outcome` including the successor `exec`). The routing, argv slicing,
//! and exit plumbing here did not change — each wave also flipped its
//! namespace's [`Binary::self_multiplexed`](crate::cli_outbound::Binary)
//! switch, so spawns target `yog <namespace>` and reach the filled arm. The
//! two edits are the whole of a wave's spine work.
//!
//! `main.rs` (coverage-excluded) stays a thin call: `dispatch(&argv)` returns
//! `Some(code)` (the process exits with it) or `None` (the GUI/hatch path,
//! unchanged). All routing logic lives here, under test.

/// Dispatch on the leading verb-namespace (`argv[1]`): `Some(exit_code)` when it
/// names a namespace (the caller exits with it), `None` for anything else — no
/// args, `--editor-apply`, `env`/`exec`, or the GUI, all unchanged (§16.7 W12).
/// `argv` is the whole process argv; the namespace's arm receives `argv[2..]`.
///
/// The W9 refusal (`prime`/`sync`/`install`, reserved exit 91) is **gone**
/// (bl-2930): U-balls-3 landed the plugin-binary lib seam upstream
/// (`delivery_bin::run` / `tracker::run`), the [`bl_delivery`]/[`bl_tracker`]
/// arms answer it, and the `bl` arm hands balls the world's own shim as its
/// executable — so a `prime` binds a plugin chain that IS yog, and the whole
/// verb surface runs embedded. W12's per-arm "not embedded" codes were already
/// dead; now no yog-reserved `bl` exit code exists at all.
pub fn dispatch(argv: &[String]) -> Option<i32> {
    // Help is answered before anything routes — and, since bl-52ed, before
    // `main.rs` reaches its own subcommand match, so no command composes a
    // world, spawns, parks or writes a shim on the way to its own page (§8.5's
    // higher-order rule at the argv surface, `help`). A namespace's `--help` is
    // the embedded tool's, so it falls through to the arm, which answers it
    // world-free. The top level is intercepted here rather than left to clap,
    // which knows only the window's own flags and would advertise nothing below.
    if let Some(page) = help::answer(argv) {
        println!("{page}");
        return Some(0);
    }
    let namespace = Namespace::from_arg(argv.get(1).map(String::as_str)?)?;
    Some(namespace.run(argv.get(2..).unwrap_or_default()))
}

/// The whole top-level surface, in one place: the window's own flags (rendered
/// by clap, so they are never restated here), then every leading word yog
/// answers to. Both come from [`help::COMMANDS`] — the same table every
/// per-command page is rendered from, whose `verb` is the const its dispatcher
/// routes on — so nothing here can drift from what runs. A row with no summary
/// is unadvertised (`tool-control`, a machine seam); balls' two plugin binaries
/// carry no row at all. The column is measured, not chosen: a line added
/// tomorrow aligns itself.
fn usage() -> String {
    use clap::CommandFactory;
    let rows: Vec<&crate::boundary::help::HelpRow> = help::COMMANDS
        .iter()
        .filter(|row| !row.summary.is_empty())
        .collect();
    let column = rows.iter().map(|row| row.usage.chars().count()).max();
    let listed: Vec<String> = rows
        .iter()
        .map(|row| {
            let gap = " ".repeat(
                column
                    .unwrap_or_default()
                    .saturating_sub(row.usage.chars().count())
                    + 2,
            );
            format!("  {}{gap}{}", row.usage, row.summary)
        })
        .collect();
    format!(
        "{}\nCommands:\n{}\n\nEvery command answers --help. `yog gesture --help` lists every \
         gesture; `yog gesture --help <command>` is one command's page.\n",
        crate::Args::command().render_help(),
        listed.join("\n"),
    )
}

/// Every namespace and its arm — the router's whole table (§16.7 W12,
/// bl-2930). What a word *means* is [`help::COMMANDS`]'s business; balls' two
/// plugin binaries are absent from it because balls' own plugin chain spawns
/// them and no operator types one.
const NAMESPACES: &[(&str, Namespace)] = &[
    ("lernie", Namespace::Lernie),
    ("bl", Namespace::Bl),
    ("bz", Namespace::Bz),
    ("bl-delivery", Namespace::BlDelivery),
    ("bl-tracker", Namespace::BlTracker),
    ("gesture", Namespace::Gesture),
];

/// The embedded-tool namespaces yog multiplexes to (§16.7 W12): the three
/// agent tools, plus balls' two sibling plugin binaries (bl-2930) — spawned
/// not by yog but by the embedded balls' own plugin chain, through the
/// `world/tools/` shims a `yog bl prime` binds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Namespace {
    Lernie,
    Bl,
    Bz,
    BlDelivery,
    BlTracker,
    Gesture,
}

impl Namespace {
    /// The namespace a leading verb names, or `None` — the signal that `arg` is
    /// not a multiplex target (the GUI/hatch path).
    fn from_arg(arg: &str) -> Option<Self> {
        NAMESPACES
            .iter()
            .find(|(word, _)| *word == arg)
            .map(|(_, namespace)| *namespace)
    }

    /// Route to the namespace's arm with the sliced verb args (§16.7 W12).
    fn run(self, args: &[String]) -> i32 {
        match self {
            Namespace::Lernie => lernie::run(args),
            Namespace::Bl => bl::run(args),
            Namespace::Bz => bz::run(args),
            Namespace::BlDelivery => bl_delivery::run(args),
            Namespace::BlTracker => bl_tracker::run(args),
            Namespace::Gesture => gesture::run(args),
        }
    }
}

/// The `gesture` arm (§8.5): the control boundary's deposit-and-wait sugar —
/// `yog gesture '<json>'` deposits into the composed world's gestures inbox
/// and waits for a consumer's reply ([`crate::boundary::sugar`]). The world
/// is composed here at the process edge, exactly as the GUI/headless paths
/// compose it (§16.2), so the sugar addresses the same nested state root.
mod gesture {
    use crate::ui_state::Clock;
    use std::time::Duration;

    /// 1200 × 50 ms — a 60 s answer budget before the timeout exit.
    const WAITS: u32 = 1200;
    const POLL: Duration = Duration::from_millis(50);

    pub(super) fn run(args: &[String]) -> i32 {
        let world = crate::world::compose(&crate::xdg::Env::from_env());
        // A *seed*, not an id: the clock second and the pid make the inbox
        // legible and time-ordered, and neither is unique across process
        // namespaces — the id itself is won from the world (bl-aa9f).
        let seed = format!(
            "{}-{}",
            crate::ui_state::SystemClock.stamp(),
            std::process::id()
        );
        crate::boundary::sugar::run(&world.yog_state_root(), args, &seed, WAITS, &mut || {
            std::thread::sleep(POLL);
        })
    }
}

/// The `lernie` arm — **filled by W11**: lernie's own thin exec binding, in
/// yog's process (see the module doc in `multiplex/lernie.rs`).
mod lernie;

mod bl;

/// The landing repair the `bl` arm runs on the way in (§16.3, bl-7e54): a
/// landing yog founded before balls' config home was nested carries a schedule
/// seeded from the operator's stale template, and balls re-seeds a landing only
/// when founding one. See the module doc.
mod landing;

/// The argv seat's help, read above the router (§8.5, bl-52ed) — the top-level
/// roster, every per-command page, and the discovery probe the namespace arms
/// answer world-free.
pub(crate) mod help;

/// The `bl-delivery` arm (bl-2930): balls' delivery plugin, in yog's process —
/// the upstream `pub` boundary ([`balls::delivery_bin::run`], U-balls-3) over
/// live env resolved here at the process edge, exactly as the shipped sibling
/// binary's `main` resolves it. Reached through the `world/tools/bl-delivery`
/// shim a `yog bl prime` binds into the checkout's `config/plugins/bin/`;
/// balls spawns it subprocess-uniform (§6) with the §7 wire on stdin.
mod bl_delivery {
    use std::env;
    use std::io;
    use std::path::{Path, PathBuf};

    pub(super) fn run(args: &[String]) -> i32 {
        let home = env::var("HOME").unwrap_or_default();
        let env = balls::delivery_bin::Env {
            plugin: env::var("BALLS_PLUGIN_NAME").ok(),
            xdg: balls::layout::Xdg::with(
                Path::new(&home),
                env::var("XDG_CONFIG_HOME").ok().as_deref(),
                env::var("XDG_STATE_HOME").ok().as_deref(),
            ),
            cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        };
        balls::delivery_bin::run(
            args,
            &mut io::stdin().lock(),
            &mut io::stdout().lock(),
            &env,
        )
    }
}

/// The `bl-tracker` arm (bl-2930): balls' tracker plugin, in yog's process —
/// the upstream `pub` boundary ([`balls::tracker::run`]) over live env
/// resolved here, exactly as the shipped `bl-tracker` binary's `main` does.
/// Bound and spawned the same way as [`bl_delivery`].
mod bl_tracker {
    use std::env;
    use std::io;
    use std::path::PathBuf;

    pub(super) fn run(args: &[String]) -> i32 {
        let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        let xdg = balls::layout::Xdg::with(
            &home,
            env::var("XDG_CONFIG_HOME").ok().as_deref(),
            env::var("XDG_STATE_HOME").ok().as_deref(),
        );
        let env = balls::tracker::Env { xdg };
        balls::tracker::run(
            args,
            &mut io::stdin().lock(),
            &mut io::stdout().lock(),
            &env,
        )
    }
}

/// The `bz` arm — **filled by W10**: [`crate::bz_host`] is `bz`'s own `main` in
/// yog's process, over the linked brazen's `native-host` shim.
mod bz {
    /// `yog bz <argv…>` IS `bz <argv…>`: brazen's route dispatch over the real
    /// process stdio and this process's own env — which is where the **wall**
    /// arrives (§16.2 as amended). An agent's bare `bz` is the world's shim
    /// re-entering here, and it inherits `YOG_WALL` from the loop that fired,
    /// so the config, sign-ins and cache it reaches are its workspace's own; a
    /// `bz` outside any workspace has no wall and is refused. Every route
    /// works, not just the two yog drives, because this is also the adapter a
    /// linked lernie will exec (§16.7 W11).
    pub(super) fn run(args: &[String]) -> i32 {
        crate::bz_host::run(
            args.to_vec(),
            &crate::xdg::Env::from_env(),
            crate::bz_host::Tty::probe(),
            &mut std::io::stdin().lock(),
            &mut std::io::stdout().lock(),
            &mut std::io::stderr().lock(),
        )
    }
}

#[cfg(test)]
mod tests;
