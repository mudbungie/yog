//! Dispatch routing, argv slicing, and per-namespace exit plumbing (§16.7 W12).
//! Every branch of [`dispatch`]/[`Namespace::from_arg`]/[`Namespace::run`] is
//! pinned here; each arm's own proof rides the cheapest verb that reaches its
//! embedded crate (all three arms are filled — W8, W10, W11).

use super::*;

/// Build a process-argv vector (`argv[0]` is the program) from string parts.
fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn no_leading_verb_is_not_a_namespace() {
    // Bare `yog` (GUI launch) and an empty argv both fall through to `None`.
    assert_eq!(dispatch(&argv(&["yog"])), None);
    assert_eq!(dispatch(&[]), None);
}

#[test]
fn a_non_namespace_verb_falls_through() {
    // The hatches and any GUI arg are not namespaces — unchanged behavior.
    assert_eq!(dispatch(&argv(&["yog", "--editor-apply", "/co"])), None);
    assert_eq!(dispatch(&argv(&["yog", "env"])), None);
    assert_eq!(dispatch(&argv(&["yog", "exec", "bash"])), None);
    assert_eq!(dispatch(&argv(&["yog", "something-else"])), None);
}

#[test]
fn the_lernie_arm_runs_the_embedded_lernie_crate() {
    // W11: `yog lernie …` IS lernie's thin exec binding. `--help` is clap's
    // own short-circuit — it prints the shared `cmd::Cli` surface and exits 0
    // before any prelude, world write, or Fx build, so it proves the crate
    // parse is wired (the W12 stub would have returned its 90). An unknown
    // verb is clap's usage error (2) — lernie's own exit, not yog's.
    assert_eq!(dispatch(&argv(&["yog", "lernie", "--help"])), Some(0));
    assert_eq!(dispatch(&argv(&["yog", "lernie", "no-such-verb"])), Some(2));
    // The full verb path (preludes, shim converge, Fx, Outcome) is driven by
    // `tests/multiplex_lernie.rs`, which owns its process env.
}

// The `bl` arm's own proof lives in `tests/multiplex_bl.rs` (bl-2930): the arm
// converges the world's tool shims on the way into every verb (its `prime`
// binds them as balls' siblings), so any verb here would write under the
// ambient `$XDG_DATA_HOME` — an integration binary that owns its process env
// is the hermetic home, exactly the W11 lernie-arm precedent. That binary also
// carries the bl-52ed proof that a *discovery probe* writes nothing at all,
// which is only meaningful against an anchor the test owns.

#[test]
fn the_bz_arm_reaches_the_embedded_brazen_crate_and_asks_it_for_a_wall() {
    // W10: `yog bz …` IS `bz …` — brazen's own code, in this process. Since
    // the blast-radius ruling it is brazen *inside a workspace*:
    // this arm folds the wall out of its own process env (§16.2), and a test
    // runner's env names none, so the answer is the wall refusal — not the 92
    // an unfilled arm would return, and not the machine's own brazen state.
    // Reaching brazen's wall gate at all is the proof the arm is wired.
    assert_eq!(
        dispatch(&argv(&["yog", "bz", "--list-providers"])),
        Some(crate::bz_host::NO_WALL_CODE)
    );
    // bl-52ed: a discovery probe is the gate's one exemption — it reads
    // brazen's interface, never the world, so it answers with no wall at all.
    assert_eq!(dispatch(&argv(&["yog", "bz", "--version"])), Some(0));
}

#[test]
fn a_namespace_with_no_verb_args_dispatches_with_an_empty_slice() {
    // `yog <ns>` alone: `argv[2..]` is an empty slice, the arm still runs and
    // returns a code (the slicing never indexes out of bounds). Bare `yog
    // lernie` is clap's missing-subcommand usage error (2) — the crate's own
    // exit. (Bare `yog bl` — balls' no-command usage exit, same shape — is
    // proved in `tests/multiplex_bl.rs`; see the note above.)
    assert_eq!(dispatch(&argv(&["yog", "lernie"])), Some(2));
}

#[test]
fn from_arg_maps_names_and_rejects_the_rest() {
    assert_eq!(Namespace::from_arg("lernie"), Some(Namespace::Lernie));
    assert_eq!(Namespace::from_arg("bl"), Some(Namespace::Bl));
    assert_eq!(Namespace::from_arg("bz"), Some(Namespace::Bz));
    assert_eq!(
        Namespace::from_arg("bl-delivery"),
        Some(Namespace::BlDelivery)
    );
    assert_eq!(
        Namespace::from_arg("bl-tracker"),
        Some(Namespace::BlTracker)
    );
    assert_eq!(Namespace::from_arg("gui"), None);
    assert_eq!(Namespace::from_arg(""), None);
}

/// bl-2930 — the plugin arms answer the §6 self-describe exactly as the
/// shipped sibling binaries do: `protocol` prints the self-description and
/// exits 0, before any env or stdin is needed. Reaching each upstream boundary
/// (`delivery_bin::run` / `tracker::run`) is the proof the arm is filled; the
/// hook paths run end-to-end in `tests/multiplex_bl.rs`, where a real
/// `claim` spawns them through the bound shims.
#[test]
fn the_plugin_arms_answer_protocol() {
    assert_eq!(
        dispatch(&argv(&["yog", "bl-delivery", "protocol"])),
        Some(0)
    );
    assert_eq!(dispatch(&argv(&["yog", "bl-tracker", "protocol"])), Some(0));
}

/// §16.7 W9 identity: the embedded `bl`'s default actor is `$YOG_NAME` when the
/// harness stamped one (§3.3), else balls' own `$USER`, else nothing (balls
/// falls back to `"unknown"`). An empty name reads as absent.
#[test]
fn the_default_actor_prefers_the_harness_stamped_workspace_name() {
    let name = |n: Option<&str>, u: Option<&str>| {
        bl::default_actor(n.map(str::to_owned), u.map(str::to_owned))
    };
    assert_eq!(
        name(Some("cobalt-gecko"), Some("mark")).as_deref(),
        Some("cobalt-gecko")
    );
    assert_eq!(name(None, Some("mark")).as_deref(), Some("mark"));
    assert_eq!(name(Some(""), Some("mark")).as_deref(), Some("mark"));
    assert_eq!(name(None, None), None);
    assert_eq!(
        name(Some("cobalt-gecko"), None).as_deref(),
        Some("cobalt-gecko")
    );
}

/// The top level is a command, so it answers `--help` (§8.5's rule applied to
/// the argv surface) — before clap, which knows only the window's own flags.
#[test]
fn the_top_level_answers_help_in_every_spelling() {
    for spelling in ["--help", "-h", "help"] {
        assert_eq!(dispatch(&argv(&["yog", spelling])), Some(0), "{spelling}");
    }
}

/// What it says is the surface itself: the windowless face, both hatches, the
/// boundary's own verb, and every namespace a human is meant to type — each
/// named by the const its dispatcher routes on, so the page cannot drift from
/// what runs. The two plugin binaries are not advertised: balls' own chain
/// spawns them, and no operator types one.
#[test]
fn the_top_level_page_states_the_whole_surface() {
    let page = usage();
    for word in [
        crate::boundary::SERVE_SUBCMD,
        crate::world::hatch::ENV_SUBCMD,
        crate::world::hatch::EXEC_SUBCMD,
        "gesture",
        "lernie",
        "bl",
        "bz",
    ] {
        assert!(page.contains(&format!("yog {word}")), "{word} is unlisted");
    }
    assert!(!page.contains("bl-delivery"), "plugin arms are not typed");
    assert!(!page.contains("bl-tracker"));
    // It says how to go deeper, since one line per command is not a manual.
    assert!(page.contains("--help"));
    // And it still carries the window's own flags, rendered by clap rather
    // than restated here.
    assert!(page.contains("--workspace"));
}

/// The namespace table is the router: every advertised word routes, and the
/// unadvertised ones still do.
#[test]
fn every_table_word_routes_to_its_arm() {
    for (word, namespace) in namespace::NAMESPACES {
        assert_eq!(Namespace::from_arg(word), Some(*namespace), "{word}");
    }
    assert_eq!(Namespace::from_arg("nope"), None);
}
