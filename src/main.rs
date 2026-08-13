use clap::Parser;
use std::sync::Arc;
use yog::cli_outbound::{Binary, Cli};
use yog::config_edit;
use yog::engine::Engine;
use yog::shell::ShellState;
use yog::ui_state::{Clock, SystemClock};
use yog::watch::{EguiRepaint, NoRepaint};
use yog::world::hatch;
use yog::xdg::Env;
use yog::{Args, shell};

/// §8.4 (bl-44a5): converge the world's tool shims before a hatch hands the
/// world out — the tools dir is a generated artifact of the world itself, not
/// of the start flow, and the world `PATH` names it unconditionally. Warn on
/// stderr and continue on failure (stdout stays the hatch's one product).
fn seed_world_tools(ambient: &Env) {
    let tools = yog::world::layout(ambient).tools;
    if let Err(e) = yog::world::tools::ensure_tools(&tools) {
        eprintln!("yog: seed world tools: {e}");
    }
}

fn main() -> eframe::Result<()> {
    // §9.3 shim mode: the `$EDITOR` lernie execs re-enters here BEFORE clap or
    // eframe. argv is `<yog> --editor-apply <checkout>`; `YOG_EDIT_SRC` (env)
    // carries the staging dir. Exit 0/non-zero — non-zero aborts lernie's
    // commit cleanly (see `config_edit::apply` for the invocation shape).
    let argv: Vec<String> = std::env::args().collect();
    if argv.get(1).map(String::as_str) == Some(config_edit::apply::EDITOR_APPLY_FLAG) {
        std::process::exit(config_edit::apply::run_shim(
            std::env::var("YOG_EDIT_SRC").ok(),
            argv.get(2).cloned(),
        ));
    }

    // §16.7 W12 self-multiplex, and — since bl-52ed — the argv seat's help,
    // which is why this stands ABOVE the subcommand match: a `lernie`/`bl`/`bz`
    // leading verb dispatches to that embedded crate's arm (`yog <namespace>
    // …`) and exits with its code, and `yog <command> --help` is answered from
    // the interface before any command below composes a world, spawns, parks or
    // writes a shim (§8.5's every-command-answers-help rule). Anything else (no
    // args, `--editor-apply`, `env`, `exec`, `headless`, `tool-control`, the GUI
    // below) is not a namespace and falls through unchanged. All routing lives
    // in `multiplex` (tested); main stays a thin call.
    if let Some(code) = yog::multiplex::dispatch(&argv) {
        std::process::exit(code);
    }
    // Read the ambient env once (§16.2): the world's fixed override set derives
    // from it and stands on every child spawn (§16.6 W2); both escape hatches
    // below join a human to that same world.
    let ambient = Env::from_env();
    let overrides = yog::world::overrides(&ambient);
    // §8.4 world escape hatches (`yog env` / `yog exec`): multi-call subcommands
    // beside `--editor-apply`, dispatched before clap and eframe — they need no
    // display, and clap must never see `env`/`exec` as unknown positionals.
    // Both converge the world's tool shims first (bl-44a5): the world `PATH`
    // they hand out is fronted by `world/tools/` unconditionally, so the dir
    // must be real before any Start has seeded it — otherwise a bare `bl`
    // fell through to a host binary (or died in a clean room, §16.7 W14). A
    // converge failure is warned, not fatal: the hatch still works for
    // commands that need no shim.
    match argv.get(1).map(String::as_str) {
        // `eval "$(yog env)"` drops the caller's shell into the world — and
        // `yog env --ws <workspace>` drops it into that workspace's **wall**
        // besides (§8.4 as amended, bl-b589), which is the supported headless
        // spelling for every wall-needing command, sign-in included.
        Some(hatch::ENV_SUBCMD) => match hatch::parse_env(argv.get(2..).unwrap_or_default()) {
            Ok(plan) => {
                seed_world_tools(&ambient);
                print!(
                    "{}",
                    hatch::env_script(&hatch::overrides_for(&ambient, plan.workspace.as_deref()))
                );
                return Ok(());
            }
            Err(e) => {
                eprintln!("yog {}: {e}", hatch::ENV_SUBCMD);
                std::process::exit(2);
            }
        },
        // `yog exec [--cwd DIR] <cmd…>` runs one command inside the world, its
        // exit faithfully yog's (a plan parse error is 2; a spawn failure 127).
        Some(hatch::EXEC_SUBCMD) => match hatch::parse_exec(argv.get(2..).unwrap_or_default()) {
            Ok(plan) => {
                seed_world_tools(&ambient);
                let cmd_args: Vec<&str> = plan.args.iter().map(String::as_str).collect();
                // `--ws` layers that workspace's wall over the world (bl-b589),
                // so `yog exec --ws <ws> bz --login …` signs in *inside* the
                // sphere and the credential lands there and nowhere else.
                let env = hatch::overrides_for(&ambient, plan.workspace.as_deref());
                match Cli::exec_in_world(&plan.cmd, &env, plan.cwd.as_deref(), &cmd_args) {
                    Ok(info) => std::process::exit(info.shell_code()),
                    Err(e) => {
                        eprintln!("yog {}: {e}", hatch::EXEC_SUBCMD);
                        std::process::exit(127);
                    }
                }
            }
            Err(e) => {
                eprintln!("yog {}: {e}", hatch::EXEC_SUBCMD);
                std::process::exit(2);
            }
        },
        // `yog tool-control` (§8.6): the capability control lernie's seam
        // consults before every granted tool invocation, spawned with no argv
        // beyond this word and speaking over the real stdio — so it binds here
        // at the process edge, exactly as the hatches above do.
        Some(yog::control::SUBCMD) => {
            let world = yog::world::compose(&ambient);
            let (mut i, mut o) = (std::io::stdin().lock(), std::io::stdout().lock());
            let ws = yog::control::workspace_of(&world);
            std::process::exit(yog::control::run(&mut i, &mut o, &world, &ws));
        }
        // `yog headless` (§8.5, VISION §4.8): the same engine with no window —
        // the world composed, the derivation worker and watch bridge up, and
        // the gestures-inbox consumer answering deposits — parked until a
        // signal ends the process (§4.1 state is write-through; nothing
        // pends at exit, exactly as the GUI's no-`on_exit` rule).
        Some(yog::boundary::HEADLESS_SUBCMD) => headless(&ambient, &overrides),
        _ => {}
    }
    // §16.4 (bl-3ff4): a window is the operator's own act. The world seeds a
    // `yog` shim so an agent's bash can drive the §8.5 boundary, and that shim
    // passes argv through verbatim — so this is where an agent seat asking for
    // a window is refused and pointed at the headless surface instead. It
    // stands below every namespace arm, hatch, `headless` and `tool-control`,
    // so it judges only argv that would really have painted.
    if let Some(refusal) = yog::world::seat::window_refusal(std::env::var("YOG_NAME").ok()) {
        eprintln!("{refusal}");
        std::process::exit(yog::world::seat::REFUSED);
    }
    let args = Args::parse();
    // Compose the nested world (§16.2): every read below derives through `world`
    // (so yog watches the nested clones/state/lernie-home) and every child spawns
    // with `overrides` standing (§16.6 W2), so reads and spawns agree.
    let world = yog::world::compose(&ambient);
    eframe::run_native(
        "yog",
        eframe::NativeOptions {
            // Size the first-launch window in logical points (winit applies the
            // display scale): the S0 surface needs the roster plus a real center,
            // so a default-tiny window never slivers the composer on HiDPI.
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1150.0, 760.0])
                .with_min_inner_size([420.0, 320.0])
                // The congeries mark (§11), computed rather than decoded — the
                // same orb table `assets/yog.svg` is emitted from.
                .with_icon(yog::theme::icon::icon_data())
                // Wayland app_id / X11 WM_CLASS. It must equal the desktop
                // entry's basename (`assets/yog.desktop`, StartupWMClass) or
                // the shell cannot match the running window to the installed
                // icon and falls back to a generic one.
                .with_app_id("yog"),
            ..Default::default()
        },
        Box::new(move |cc| {
            // The congeries visuals (§11): installed once, before first paint.
            yog::theme::apply(&cc.egui_ctx);
            // One time source for the whole window (§7.2): the derivation
            // worker's schedule and the shell's §7.3 banner grace both read it.
            let clock: Arc<dyn Clock> = Arc::new(SystemClock);
            // The engine — the same one `yog headless` boots. Everything below
            // it is what a *window* is and a windowless face is not.
            let engine = Engine::boot(
                &world,
                &overrides,
                args.workspace,
                Arc::clone(&clock),
                Arc::new(EguiRepaint(cc.egui_ctx.clone())),
            );
            // The §8.5 searcher: the window's own searches run here, never on
            // the frame (a search walks every transcript in the world). The
            // windowless face needs none — `yog gesture` and the consumer both
            // answer a search in place, already off-frame.
            let searcher = engine.model.searcher().spawn();
            // The shell's RAM surfaces, incl. the config editors folded from the
            // world env (§9) — their `bz`/`bl conf` runners nest too. A load
            // error here is fatal at bring-up only.
            let state = ShellState::new(&world, clock)?;
            Ok(Box::new(App {
                engine,
                _searcher: searcher,
                state,
                lernie: Cli::resolve_in_world(Binary::Lernie, &overrides),
                bl: Cli::resolve_in_world(Binary::Bl, &overrides),
                bz: Cli::resolve_in_world(Binary::Bz, &overrides),
            }))
        }),
    )
}

/// `yog headless` (§8.5, VISION §4.8): **the same engine with no window** —
/// one [`Engine::boot`], no face beside it, parked until a signal ends the
/// process (§4.1 state is write-through; nothing pends at exit, exactly as the
/// GUI's no-`on_exit` rule). The window's arm above and this one are the same
/// call with a different repaint hook, which is the whole of VISION V5.4's
/// "nothing here is a second implementation".
fn headless(ambient: &Env, overrides: &[(String, String)]) -> ! {
    seed_world_tools(ambient);
    let _engine = Engine::boot(
        &yog::world::compose(ambient),
        overrides,
        None,
        Arc::new(SystemClock),
        Arc::new(NoRepaint),
    );
    loop {
        std::thread::park();
    }
}

struct App {
    // The engine both faces run (VISION §5 V5): the model the frame renders
    // plus the derivation worker, watch bridge and gesture consumer it holds —
    // dropped on exit, which stops and joins each.
    engine: Engine,
    // The §8.5 searcher thread — the window's half of the search query.
    _searcher: yog::search::SearchThread,
    // Every RAM surface the shell owns: the action/start drafts, the inspector
    // ephemera, and the config editors (§3.5 — discarded on exit).
    state: ShellState,
    // The mutating-verb binaries: message/stop/scan/prompt on `lernie`,
    // close/unclaim/create/update on `bl` (§8.2), and `bz --login` — bz's one
    // interactive verb — streamed from the Login pane (§8.3). Ball *reads* are
    // in-process on the model's own `BlStore` (§16.7 W8); these drive the short
    // *actions*, which stay processes (balls' seal CAS + plugin chain).
    lernie: Cli,
    bl: Cli,
    bz: Cli,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // The frame's whole non-render duty (§7.2): take the latest snapshot the
        // worker published, adopt an external `ui.json`, hold the §6 ack. It
        // derives nothing, spawns nothing, and waits for nothing — the window
        // stays live through a storm the worker is still chewing on (bl-ee0a).
        self.engine.model.refresh();
        // Poll floor (I4): wake at least every cheap-sweep interval even absent
        // interaction, so a published snapshot never waits on a mouse move. The
        // live cadence's period, off the snapshot (bl-3381).
        ctx.request_repaint_after(self.engine.model.cadence().cheap_sweep);
        shell::render(
            ctx,
            &mut self.engine.model,
            &mut self.state,
            &self.lernie,
            &self.bl,
            &self.bz,
        );
    }
    // No `on_exit` hook: §4.1 state is write-through at the gesture, so there
    // is nothing pending to flush at close — and nothing a SIGTERM (`pkill`,
    // which never reaches `on_exit`) could take with it (bl-b54e).
}
