//! The litany arm's binding pieces, each driven directly: the clap parse and
//! its exit-code mapping, the `$EDITOR` hand-off, and the outcome/failure
//! plumbing ([`conclude`]/[`perform`], upstream's `perform` with `i32`).
//! [`run`]'s full verb path — preludes, the shim converge, the `Fx` build —
//! rides `tests/multiplex_litany.rs`, a test binary of its own that owns its
//! process environment (`LITANY_HOME`/`XDG_DATA_HOME`/`EDITOR`).
//! The `Exec` outcome's *returning* half rides `tests/exec_return.rs` for the
//! same reason one layer down: a returning exec is a process-global act, and
//! this binary has 2,600 peer threads (bl-419d).

use super::*;
use std::os::unix::ffi::OsStringExt as _;

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn parse_accepts_a_verb_and_maps_help_and_errors_to_exit_codes() {
    // A well-formed verb parses into litany's own surface.
    let cli = parse(&args(&["new", "/tmp/ws"])).unwrap();
    assert!(matches!(cli.command, cmd::Command::New(_)));
    // `--help` is clap's short-circuit: printed, exit 0 — never a verb.
    assert_eq!(parse(&args(&["--help"])).unwrap_err(), 0);
    // The door verb takes no arguments and is a verb like any other — a
    // program's `litany_tools` stub spawns exactly this argv (litany
    // `docs/DESIGN_CODE_EXECUTION.md` §2.8), and the arm builds one `Fx`, so
    // the injection it re-enters through is installed here as for `advance`.
    // `tests/litany_invoke.rs` drives the real shim end to end.
    assert!(matches!(
        parse(&args(&["invoke"])).unwrap().command,
        cmd::Command::Invoke(_)
    ));
    // An unknown verb and a bare argv are clap usage errors, exit 2.
    assert_eq!(parse(&args(&["no-such-verb"])).unwrap_err(), 2);
    assert_eq!(parse(&[]).unwrap_err(), 2);
}

#[test]
fn edit_with_maps_the_editor_exit_to_the_edit_result() {
    let dir = tempfile::TempDir::new().unwrap();
    // The editor is handed the checkout dir as `"$1"` through `sh -c`, so a
    // multi-word $EDITOR works; a zero exit is a completed edit.
    let ok = edit_with("test -d", dir.path());
    assert!(ok.is_ok());
    // A non-zero editor exit is a failed edit, message carrying the status.
    let err = edit_with("false", dir.path()).unwrap_err();
    assert!(err.to_string().contains("editor exited"));
}

#[test]
fn conclude_performs_success_and_prints_the_uniform_failure() {
    assert_eq!(conclude(Ok(Outcome::Quiet)), 0);
    assert_eq!(conclude(Err(cmd::Error::new("prompt", "boom"))), 1);
}

#[test]
fn perform_maps_each_outcome_to_its_exit() {
    // The one-product line prints and succeeds; quiet succeeds silently.
    assert_eq!(perform(Outcome::Line("a-branch".to_owned())), 0);
    assert_eq!(perform(Outcome::Quiet), 0);
    // The tool verb's exit code rides through within u8.
    assert_eq!(perform(Outcome::Code(7)), 7);
    // The successor exec: a successful execve never returns, so the arm past
    // it IS the failure path — a target that cannot exec reports and fails.
    //
    // **The target is refused ABOVE `do_exec`, on purpose** (bl-419d). std
    // rejects a command whose program, argv or cwd holds a NUL before it
    // touches the process at all, so this beat proves the arm's mapping while
    // spending neither of the two global effects a returning exec leaves — the
    // `SIGPIPE` reset (bl-3792) and the freed `environ` copy that reddened a
    // peer's spawn with this very error text. A real returning `execvp` has
    // exactly one lawful home in this repo and it is not a shared test binary:
    // `tests/exec_return.rs`, one `#[test]`, no peer threads.
    let mut cmd = crate::git_env::command(Path::new("/nonexistent/yog-successor"));
    cmd.arg(std::ffi::OsString::from_vec(b"a\0b".to_vec()));
    assert_eq!(perform(Outcome::Exec(cmd)), 1);
}

/// The injection reads no verb at all (bl-fd24): the driven agent is the
/// seam's own per-assembly fact since litany bl-ddaa, so a `prompt` driver —
/// whose verb mints its agent and whose argv names none — declares an
/// identical set to a resumed `advance` driver of the same agent. Before
/// this, the binding read `(workspace, agent)` off its own argv, the minting
/// verbs answered `None`, and the conversation's first driver could load but
/// never call.
#[test]
fn the_minting_verb_and_the_naming_verb_declare_one_set() {
    use ::litany::cmd::ToolInjection as _;
    let root = tempfile::TempDir::new().expect("tmp");
    crate::tool_host::loaded::add(
        root.path(),
        "home",
        "dulcet-mongoose",
        &[crate::tool_host::loaded::Entry {
            client: "laptop".to_owned(),
            tool: crate::registry::tools::Tool {
                name: "Bash".to_owned(),
                description: "run a command".to_owned(),
                input_schema: serde_json::json!({"type": "object"}),
                subject_cwd: false,
            },
        }],
    )
    .expect("seed the loaded set");
    let injection = crate::tool_host::Injection::new(
        root.path().to_path_buf(),
        root.path().join("no-front-door"),
        crate::tool_host::ask::Budget::default(),
        crate::tool_host::remote::patience(),
        std::sync::Arc::new(SystemClock),
    );
    let names: Vec<String> = injection
        .tools(Path::new("/w/home"), "dulcet-mongoose")
        .into_iter()
        .map(|t| t.name)
        .collect();
    assert_eq!(names, vec!["clients".to_owned(), "laptop_Bash".to_owned()]);
}
