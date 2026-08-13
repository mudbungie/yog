use super::*;
use std::path::Path;

/// Every `shell_quote` shape a value can take, and its exact `eval`-safe form:
/// empty (`''`), plain, spaces, a single quote (the `'\''` dance), consecutive
/// and multiple quotes, and a value that is only quotes.
#[test]
fn shell_quote_is_eval_safe_across_shapes() {
    let cases: &[(&str, &str)] = &[
        ("", "''"),
        ("/d/yog/world/lernie", "'/d/yog/world/lernie'"),
        ("a b c", "'a b c'"),
        ("it's", "'it'\\''s'"),
        ("a'b'c", "'a'\\''b'\\''c'"),
        ("''", "''\\'''\\'''"),
        ("no'space's", "'no'\\''space'\\''s'"),
    ];
    for &(input, expect) in cases {
        assert_eq!(shell_quote(input), expect, "quoting {input:?}");
    }
}

#[test]
fn env_script_emits_one_quoted_export_per_override_in_order() {
    let overrides = vec![
        ("LERNIE_HOME".to_owned(), "/d/yog/world/lernie".to_owned()),
        (
            "XDG_STATE_HOME".to_owned(),
            "/weird dir/world/state".to_owned(),
        ),
    ];
    assert_eq!(
        env_script(&overrides),
        "export LERNIE_HOME='/d/yog/world/lernie'\n\
         export XDG_STATE_HOME='/weird dir/world/state'\n"
    );
}

#[test]
fn env_script_is_empty_for_no_overrides() {
    assert_eq!(env_script(&[]), "");
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn parse_exec_takes_bare_command() {
    let plan = parse_exec(&argv(&["bl"])).unwrap();
    assert_eq!(
        plan,
        ExecPlan {
            cwd: None,
            workspace: None,
            cmd: "bl".to_owned(),
            args: vec![],
        }
    );
}

#[test]
fn parse_exec_keeps_command_arguments() {
    let plan = parse_exec(&argv(&["bl", "list", "--json"])).unwrap();
    assert_eq!(
        plan,
        ExecPlan {
            cwd: None,
            workspace: None,
            cmd: "bl".to_owned(),
            args: vec!["list".to_owned(), "--json".to_owned()],
        }
    );
}

#[test]
fn parse_exec_reads_leading_cwd() {
    let plan = parse_exec(&argv(&["--cwd", "/proj", "bl", "close", "bl-1"])).unwrap();
    assert_eq!(
        plan,
        ExecPlan {
            cwd: Some(PathBuf::from("/proj")),
            workspace: None,
            cmd: "bl".to_owned(),
            args: vec!["close".to_owned(), "bl-1".to_owned()],
        }
    );
}

#[test]
fn parse_exec_passes_through_a_non_leading_cwd() {
    // `--cwd` only binds to yog when it leads; here it is the command's own flag.
    let plan = parse_exec(&argv(&["bl", "--cwd", "/x"])).unwrap();
    assert_eq!(
        plan,
        ExecPlan {
            cwd: None,
            workspace: None,
            cmd: "bl".to_owned(),
            args: vec!["--cwd".to_owned(), "/x".to_owned()],
        }
    );
}

#[test]
fn parse_exec_errors_on_empty_argv() {
    assert_eq!(parse_exec(&[]), Err(ExecError::MissingCommand));
}

#[test]
fn parse_exec_errors_when_cwd_has_no_directory() {
    assert_eq!(
        parse_exec(&argv(&["--cwd"])),
        Err(ExecError::MissingCwdValue)
    );
}

#[test]
fn parse_exec_errors_when_only_cwd_and_dir_given() {
    assert_eq!(
        parse_exec(&argv(&["--cwd", "/proj"])),
        Err(ExecError::MissingCommand)
    );
}

#[test]
fn exec_error_messages_are_human_readable() {
    assert_eq!(
        ExecError::MissingCwdValue.to_string(),
        "--cwd requires a directory argument"
    );
    assert!(
        ExecError::MissingCommand
            .to_string()
            .contains("no command given")
    );
}

/// bl-b589 — **the hatches take `--ws`, and that is the headless workspace
/// binding.** Before it, `yog env`/`yog exec` handed out the world and only the
/// world, so nothing a human or a teleoperator could type stood inside a
/// workspace's wall — which made the advertised `yog bz --login` refuse with
/// "no workspace in this environment" and left sign-in unreachable outside the
/// window.
#[test]
fn both_hatches_read_a_leading_workspace() {
    let plan = parse_exec(&argv(&["--ws", "/spheres/corp", "bz", "--login"])).unwrap();
    assert_eq!(plan.workspace.as_deref(), Some(Path::new("/spheres/corp")));
    assert_eq!(plan.cmd, "bz");
    assert_eq!(plan.args, vec!["--login".to_owned()]);
    assert_eq!(
        parse_env(&argv(&["--ws", "/spheres/corp"]))
            .unwrap()
            .workspace
            .as_deref(),
        Some(Path::new("/spheres/corp"))
    );
    // Bare, both hatches are exactly what they were: the world, no wall.
    assert_eq!(parse_env(&[]).unwrap().workspace, None);
    assert_eq!(parse_exec(&argv(&["bl"])).unwrap().workspace, None);
}

/// The leading flags are order-free, and only *leading*: once the command word
/// is reached everything belongs to the command, so a `--ws` of its own passes
/// through untouched rather than being eaten by the hatch.
#[test]
fn the_leading_flags_are_order_free_and_stop_at_the_command() {
    for order in [
        vec!["--cwd", "/proj", "--ws", "/ws", "bl"],
        vec!["--ws", "/ws", "--cwd", "/proj", "bl"],
    ] {
        let plan = parse_exec(&argv(&order)).unwrap();
        assert_eq!(plan.cwd.as_deref(), Some(Path::new("/proj")), "{order:?}");
        assert_eq!(
            plan.workspace.as_deref(),
            Some(Path::new("/ws")),
            "{order:?}"
        );
        assert_eq!(plan.cmd, "bl");
    }
    let passed = parse_exec(&argv(&[
        "--ws", "/mine", "yog", "gesture", "--ws", "/theirs",
    ]))
    .unwrap();
    assert_eq!(passed.workspace.as_deref(), Some(Path::new("/mine")));
    assert_eq!(passed.args, vec!["gesture", "--ws", "/theirs"]);
}

/// Each flag names its own missing value, and `env` refuses a word it has no
/// use for rather than silently handing out an environment the caller did not
/// ask for.
#[test]
fn a_hatch_refuses_what_it_cannot_read() {
    assert_eq!(parse_exec(&argv(&["--ws"])), Err(ExecError::MissingWsValue));
    assert_eq!(parse_env(&argv(&["--ws"])), Err(ExecError::MissingWsValue));
    assert_eq!(
        parse_env(&argv(&["bl"])),
        Err(ExecError::UnexpectedEnvArg("bl".to_owned()))
    );
    // `--cwd` is `exec`'s alone: `env` starts nothing, so it has no cwd to set.
    assert_eq!(
        parse_env(&argv(&["--cwd", "/proj"])),
        Err(ExecError::UnexpectedEnvArg("--cwd".to_owned()))
    );
}

/// The wall layer is what `--ws` buys, and naming no workspace layers nothing —
/// so a bare hatch still hands out a world with no wall, which is what keeps
/// brazen refusing rather than reaching the machine's own sign-ins (§16.2).
#[test]
fn the_workspace_layers_its_wall_over_the_world_and_nothing_else_does() {
    let ambient = crate::test_support::world_under(Path::new("/anchor"));
    let bare = overrides_for(&ambient, None);
    let bound = overrides_for(&ambient, Some(Path::new("/spheres/corp")));
    let wall_of = |pairs: &[(String, String)]| {
        pairs
            .iter()
            .find(|(k, _)| k == crate::world::wall::YOG_WALL)
            .map(|(_, v)| v.clone())
    };
    assert_eq!(wall_of(&bare), None, "no workspace named, no wall stood");
    let stood = wall_of(&bound).expect("the named workspace's wall");
    assert!(stood.ends_with("/walls/corp"), "{stood}");
    // Everything else the world overrides is untouched by the wall layer.
    for (key, value) in &bare {
        assert_eq!(
            bound.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            Some(value),
            "{key} changed"
        );
    }
}
