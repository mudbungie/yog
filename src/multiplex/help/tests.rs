//! The argv seat's help (bl-52ed): what each shape answers, what it refuses to
//! answer *for* a namespace, and the narrowness of the discovery probe. The
//! zero-side-effect half — that no command composes, spawns, parks or writes on
//! the way to its page — is proved per namespace in `tests/multiplex_bl.rs`
//! (the `bl` arm's shim converge) and `src/bz_host/tests.rs` (the wall gate),
//! each of which owns the env the effect would land in.

use super::*;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}

/// The top level is a command, in all three spellings, and its page is the
/// whole surface.
#[test]
fn the_top_level_answers_help_with_the_roster() {
    for spelling in ["--help", "-h", "help"] {
        let page = answer(&argv(&["yog", spelling])).unwrap_or_default();
        assert!(page.contains("Commands:"), "{spelling}: {page}");
    }
}

/// Higher-order in both directions: `yog help <command>` asks about a command
/// from the top, and `yog <command> --help` asks about it at the command. Same
/// question, same page.
#[test]
fn a_command_is_asked_about_from_either_end() {
    let subcmd = crate::world::hatch::EXEC_SUBCMD;
    let from_top = answer(&argv(&["yog", "help", subcmd]));
    let at_command = answer(&argv(&["yog", subcmd, "--help"]));
    assert_eq!(from_top, at_command);
    assert!(
        from_top.unwrap_or_default().contains("--cwd"),
        "the page is `exec`'s own"
    );
    assert_eq!(
        answer(&argv(&["yog", subcmd, "-h"])),
        at_command,
        "-h is the same ask"
    );
}

/// Every one of yog's own subcommands answers — including the two that used to
/// do their work first (`env` printed exports, `exec` tried to spawn a program
/// called `--help`), the two that never returned or refused (`headless`
/// parked; `tool-control` waited on stdin), and `tool-host`, which routed as a
/// namespace and rejected the flag as an argument (bl-4667). The roster is
/// derived from [`COMMANDS`] minus the [`SELF_ANSWERING`] set — the
/// authoritative definitions — so a later command cannot fall outside the
/// invariant silently, which is how `tool-host` regressed bl-52ed.
#[test]
fn every_yog_subcommand_answers_at_the_command() {
    let owed: Vec<&str> = COMMANDS
        .iter()
        .map(|row| row.verb)
        .filter(|verb| {
            !super::super::Namespace::from_arg(verb).is_some_and(super::super::Namespace::owns_argv)
        })
        .collect();
    assert!(
        owed.contains(&crate::wire::HOST_SUBCMD),
        "the regression's own verb is in the derived roster"
    );
    for verb in owed {
        let page = answer(&argv(&["yog", verb, "--help"])).unwrap_or_default();
        assert!(page.starts_with(&format!("yog {verb}")), "{verb}: {page}");
    }
}

/// A namespace's argv belongs to the tool, so the ask passes through to the arm
/// and the tool prints its own page — yog never answers over it.
#[test]
fn a_namespace_help_is_the_tools_own_and_falls_through() {
    // The roster is derived from the router's own table and its exhaustive
    // owns_argv classification (bl-4667), so a namespace added later is
    // judged here without this list being touched.
    for (word, namespace) in super::super::namespace::NAMESPACES {
        if namespace.owns_argv() {
            assert_eq!(answer(&argv(&["yog", word, "--help"])), None, "{word}");
        } else {
            let page = answer(&argv(&["yog", word, "--help"])).unwrap_or_default();
            assert!(page.starts_with(&format!("yog {word}")), "{word}: {page}");
        }
    }
    // Asked from the top, though, yog does have a page for the three a human
    // types — what the namespace is, and that its own `--help` is the tool's.
    for word in ["gesture", "litany", "bl", "bz"] {
        assert!(answer(&argv(&["yog", "help", word])).is_some(), "{word}");
    }
}

/// Not a help ask: no argv at all, a bare GUI launch, the `$EDITOR` shim, a
/// command run for real, and a word with no page.
#[test]
fn anything_that_is_not_a_help_ask_is_left_alone() {
    assert_eq!(answer(&[]), None);
    assert_eq!(answer(&argv(&["yog"])), None);
    assert_eq!(answer(&argv(&["yog", "--editor-apply", "/co"])), None);
    assert_eq!(answer(&argv(&["yog", "env"])), None);
    assert_eq!(answer(&argv(&["yog", "exec", "bash"])), None);
    assert_eq!(answer(&argv(&["yog", "something-else", "--help"])), None);
    assert_eq!(answer(&argv(&["yog", "something-else"])), None);
}

/// An unknown word after the top-level flag is not an error page: the roster
/// is the honest answer to "what is there?".
#[test]
fn help_about_an_unknown_command_falls_back_to_the_roster() {
    let page = answer(&argv(&["yog", "--help", "enhance"])).unwrap_or_default();
    assert!(page.contains("Commands:"), "{page}");
}

/// The table is the single source, so no page can name a word that does not
/// run: each row's usage line opens with the very verb its dispatcher routes
/// on, and every row carries a paragraph to print.
#[test]
fn every_page_opens_with_the_word_its_dispatcher_routes_on() {
    for row in COMMANDS {
        assert!(
            row.usage.starts_with(&format!("yog {}", row.verb)),
            "{}: {}",
            row.verb,
            row.usage
        );
        assert!(!row.detail.is_empty(), "{} has no page", row.verb);
    }
}

/// Exactly one row is answerable-but-unadvertised, and it is the machine seam
/// — the same rule the two balls plugin binaries live under. Everything else
/// on the roster is a word a human types.
#[test]
fn the_only_unlisted_command_is_the_machine_seam() {
    let unlisted: Vec<&str> = COMMANDS
        .iter()
        .filter(|row| row.summary.is_empty())
        .map(|row| row.verb)
        .collect();
    assert_eq!(unlisted, vec![crate::control::SUBCMD]);
}

/// **`serve` and `wire-certs` describe ONE mint, so they are read together**
/// (bl-6e0c). `serve`'s page said the listener was up *"only where an operator
/// has provisioned certificates … and silently absent otherwise"* long after
/// bl-ae05 moved the trigger into the engine's own boot
/// ([`ensure`](crate::wire::provision::ensure), aimed at
/// [`LOOPBACK`](crate::wire::provision::LOOPBACK)) and long after both faces
/// began *saying* a refusal — because `wire-certs`' page was the only one
/// anything ever read. Two pages over one authority drift apart unless a test
/// holds them to it: both must state the boot mint and its loopback aim, and
/// both must spell the explicit act as the const its dispatcher routes on, so a
/// rename cannot leave either page naming a word that no longer runs.
#[test]
fn the_serve_and_mint_pages_agree_on_who_provisions() {
    let page = |verb: &str| answer(&argv(&["yog", verb, "--help"])).unwrap_or_default();
    let mint = crate::wire::provision::verb::SUBCMD;
    for verb in [crate::boundary::SERVE_SUBCMD, mint] {
        let text = page(verb);
        assert!(
            text.contains("boot"),
            "{verb} does not name the boot mint: {text}"
        );
        assert!(
            text.contains("loopback"),
            "{verb} does not name its aim: {text}"
        );
        assert!(
            text.contains(mint),
            "{verb} does not name the explicit act: {text}"
        );
    }
    assert!(
        !page(crate::boundary::SERVE_SUBCMD).contains("silently"),
        "the retired claim — a listener absent without a word — is gone"
    );
}

/// The probe is recognized only when the whole argv *is* the flag (§8.5's
/// "the flag form counts only when the tail is exactly the flag"), so no
/// foreign crate's option grammar has to be restated to be sure a token is not
/// somebody's value.
#[test]
fn a_discovery_probe_is_the_whole_argv_or_it_is_not_one() {
    for flag in ["--help", "-h", "--version", "-V", "--skill"] {
        assert!(is_discovery(&argv(&[flag])), "{flag}");
    }
    for not in [
        vec![],
        vec!["list"],
        vec!["--system", "--help"],
        vec!["--help", "close"],
        vec!["--", "--help"],
        vec!["--helpful"],
    ] {
        assert!(!is_discovery(&argv(&not)), "{not:?}");
    }
}
