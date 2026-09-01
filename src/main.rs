use clap::Parser;
use yog::Args;
use yog::cli_outbound::Cli;
use yog::config_edit;
use yog::engine::Engine;
use yog::world::hatch;
use yog::xdg::Env;

/// **The yog server** (REMOTE §12, bl-7942). Bare `yog` boots the engine and
/// parks until a §8.5 stop — what `yog serve` used to be, and the whole binary
/// now that the window is the seat crate's. Everything above that arm is an
/// edge-bound multi-call face: the `$EDITOR` shim, the §16.7 namespace
/// multiplex, the two world hatches, the capability control, and the wire
/// mint. There is no third answer and no display stack in the process.
fn main() {
    // §9.3 shim mode: the `$EDITOR` litany execs re-enters here BEFORE clap.
    // argv is `<yog> --editor-apply <checkout>`; `YOG_EDIT_SRC` (env) carries
    // the staging dir. Exit 0/non-zero — non-zero aborts litany's commit
    // cleanly (see `config_edit::apply` for the invocation shape).
    let argv: Vec<String> = std::env::args().collect();
    if argv.get(1).map(String::as_str) == Some(config_edit::apply::EDITOR_APPLY_FLAG) {
        std::process::exit(config_edit::apply::run_shim(
            std::env::var("YOG_EDIT_SRC").ok(),
            argv.get(2).cloned(),
        ));
    }

    // §16.7 W12 self-multiplex, and — since bl-52ed — the argv seat's help,
    // which is why this stands ABOVE the subcommand match: a `litany`/`bl`/`bz`
    // leading verb dispatches to that embedded crate's arm (`yog <namespace>
    // …`) and exits with its code, and `yog <command> --help` is answered from
    // the interface before any command below composes a world, spawns, parks or
    // writes a shim (§8.5's every-command-answers-help rule). Anything else (no
    // args, `--editor-apply`, `env`, `exec`, `tool-control`, `wire-certs`, the
    // boot below) is not a namespace and falls through unchanged. All routing
    // lives in `multiplex` (tested); main stays a thin call.
    if let Some(code) = yog::multiplex::dispatch(&argv) {
        std::process::exit(code);
    }
    // Read the ambient env once (§16.2): the world's fixed override set derives
    // from it and stands on every child spawn (§16.6 W2); both escape hatches
    // below join a human to that same world.
    let ambient = Env::from_env();
    let overrides = yog::world::overrides(&ambient);
    // §8.4 world escape hatches (`yog env` / `yog exec`): multi-call subcommands
    // beside `--editor-apply`, dispatched before the boot. Both converge the
    // world's tool shims first (bl-44a5): the world `PATH` they hand out is
    // fronted by `world/tools/` unconditionally, so the dir must be real before
    // any Start has seeded it — otherwise a bare `bl` fell through to a host
    // binary (or died in a clean room, §16.7 W14). A converge failure is
    // warned, not fatal: the hatch still works for commands that need no shim.
    match argv.get(1).map(String::as_str) {
        // `eval "$(yog env)"` drops the caller's shell into the world — and
        // `yog env --ws <workspace>` drops it into that workspace's **wall**
        // besides (§8.4 as amended, bl-b589), which is the supported spelling
        // for every wall-needing command, sign-in included.
        Some(hatch::ENV_SUBCMD) => match hatch::parse_env(argv.get(2..).unwrap_or_default()) {
            Ok(plan) => {
                yog::world::tools::seed(&ambient);
                print!(
                    "{}",
                    hatch::env_script(&hatch::overrides_for(&ambient, plan.workspace.as_deref()))
                );
                return;
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
                yog::world::tools::seed(&ambient);
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
        // `yog tool-control` (§8.6): the capability control litany's seam
        // consults before every granted tool invocation, spawned with no argv
        // beyond this word and speaking over the real stdio — so it binds here
        // at the process edge, exactly as the hatches above do.
        Some(yog::control::SUBCMD) => {
            let world = yog::world::compose(&ambient);
            let (mut i, mut o) = (std::io::stdin().lock(), std::io::stdout().lock());
            let ws = yog::control::workspace_of(&world);
            std::process::exit(yog::control::run(&mut i, &mut o, &world, &ws));
        }
        // `yog wire-certs` (REMOTE §8, bl-ae05): the operator's explicit mint —
        // a server another machine dials by name, or a rotation. The boot's own
        // mint covers this box aimed at loopback, so this is the act for
        // everything else, and it is the same recipe reached by a verb.
        Some(yog::wire::provision::verb::SUBCMD) => {
            wire_certs(&ambient, argv.get(2..).unwrap_or_default());
        }
        _ => {}
    }
    // The binary's own two flags (`--version`, and the usage error an unknown
    // one earns). `--help` never reaches here — `multiplex::help` answers the
    // whole surface above, where clap knows only these two — and `Args` itself
    // carries no field since the window's `--workspace` went with the window.
    // It stands ABOVE the seat guard so a version read is never refused: it
    // asks the binary about itself and touches no world.
    let Args {} = Args::parse();
    // §16.4 (bl-3ff4, retargeted bl-7942): booting the engine is the operator's
    // own act. The world seeds a `yog` shim so an agent's bash can drive the
    // §8.5 boundary, and that shim passes argv through verbatim — so a bare
    // `yog` from inside an agent would found a SECOND engine on this world,
    // which is the instance-coordination shape DESIGN §14 rejects. It stands
    // below every namespace arm, hatch and `tool-control`, so it judges only
    // argv that would really have booted.
    if let Some(refusal) = yog::world::seat::boot_refusal(std::env::var("YOG_NAME").ok()) {
        eprintln!("{refusal}");
        std::process::exit(yog::world::seat::REFUSED);
    }
    // The whole face is `Engine::serve` and therefore tested (bl-269a) — this
    // is the one call, which is all a coverage-excluded file should ever hold
    // of a face.
    Engine::serve(&ambient, &overrides);
}

/// `yog wire-certs` (REMOTE §8, §8.2; bl-ae05, bl-64a7): the operator's explicit
/// mint — the recipe the engine's boot performs, reached by a verb — or, under
/// `WIRE_LEAF`, one extra client leaf, at `WIRE_FOOT`'s grade. The six
/// environment readings happen here because the process edge is where every
/// environment read in this crate happens (the xdg discipline); they fold into
/// `verb::plan`, which is pure.
///
/// `tail` is argv past the verb, and the verb reads none of it: `verb::stray`
/// judges it so a setting spelled after the command refuses instead of
/// vanishing into a default mint (bl-a0dd).
fn wire_certs(ambient: &Env, tail: &[String]) -> ! {
    use yog::wire::provision::verb;
    if let Some(refusal) = verb::stray(tail) {
        eprintln!("{refusal}");
        std::process::exit(2);
    }
    let read = |key: &str| std::env::var(key).ok();
    let world = yog::world::compose(ambient);
    std::process::exit(verb::perform(&verb::plan(
        &world,
        read(verb::READS[0]),
        read(verb::READS[1]),
        read(verb::READS[2]),
        read(verb::READS[3]),
        read(verb::READS[4]),
        read(verb::READS[5]),
    )));
}
