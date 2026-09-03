//! STORIES **S0-T1** bootstrap-bare-start: an empty world, one Enter → the world
//! materializes. `litany prime`, `litany new <names-root>/home` — the §3.1
//! default name, taken without asking, so the first Enter meets no name picker —
//! and a detached `litany prompt` carrying the typed text **verbatim** (bl-6920:
//! identity rides `--name`, never the payload), `YOG_NAME=home`, cwd `~`
//! (DESIGN §3.1/§3.3/§3.4/§8.1). The pre-submit name *prediction* is the seat's
//! since bl-7cc8 — no reply carries one — so what is asserted here is the mint
//! the fire actually makes.

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use litany::mint::SplitMix64;
use tempfile::tempdir;
use yog::binding::{names_root, workspace_path};
use yog::cli_outbound::Cli;
use yog::names::DEFAULT_NAME;
use yog::start::{self, DETACHED_EXIT, Deps, Payload, StartInputs};

#[test]
fn s0_t1_empty_world_one_enter_materializes_the_conversation() {
    let (bin, state) = (tempdir().unwrap(), tempdir().unwrap());
    let (yog, balls, home) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let litany_rec = Recorder::new(bin.path(), "litany").authoring_workspaces();
    let bl_rec = Recorder::new(bin.path(), "bl");
    let deps = Deps {
        bl: Cli::new(bl_rec.path()),
        litany: Cli::new(litany_rec.path()),
        state_root: state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
        // No answer from brazen: the §9.2 birth-template gate judges nothing.
    };
    let inputs = StartInputs {
        // The empty world's target: the fixed default name (§3.1), a constant —
        // not a config, not a mint, and nothing the operator was asked for.
        workspace: workspace_path(yog.path(), DEFAULT_NAME),
        repo: None,
        payload: Payload::Bare,
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    // Enter: seed + new, then the detached prompt with the typed text.
    let prepared = start::prepare(&deps, &inputs, "T0").unwrap();
    let name = prepared.workspace.clone();
    assert_eq!(
        name, DEFAULT_NAME,
        "the bootstrap names without asking (§3.1)"
    );
    assert_eq!(prepared.binding, None, "the bare rung binds no work target");
    // §3.1: the workspace's name IS its address (bl-f5f6), so the path is the
    // one derivation off it rather than a second field.
    let ws = workspace_path(yog.path(), &name);
    // The conversation mint is drawn at fire, off the held seed (§3.3).
    let minted = start::execute_prompt(
        &deps.litany,
        state.path(),
        "T0",
        &start::Fire {
            workspace: ws.clone(),
            prepared: prepared.clone(),
            goal: "make me a plan".to_owned(),
        },
        &[],
        &SplitMix64::from_seed(7),
    )
    .unwrap();

    // The full argv sequence: prime, new <names-root>/home, prompt — the pinned
    // template already grants the worker role's whole tool pool (§8.1,
    // bl-7fc8), so nothing advances config/default a second time.
    let inv = litany_rec.wait(3);
    assert_eq!(inv[0].argv, ["prime"]);
    assert_eq!(inv[1].argv, ["new", ws.to_string_lossy().as_ref()]);
    assert!(
        ws.starts_with(names_root(yog.path())),
        "the workspace is under the flat names root"
    );
    assert_eq!(inv[2].argv[0], "prompt");
    assert_eq!(inv[2].argv[1], "--name");
    assert_eq!(inv[2].argv[3], ws.to_string_lossy());
    let conversation = inv[2].argv[2].clone();
    // The fire hands the minted name back — the §3.4 focus claim's only handle:
    // the started root has no agent id until the detached driver writes it, so
    // this name is what the view-model holds until the roster carries the
    // litany-stored name fact (bl-49cb).
    assert_eq!(
        minted, conversation,
        "the name the fire returns is the name --name carries — one mint, one channel (§3.3)"
    );
    assert_eq!(
        inv[2].argv[4], "make me a plan",
        "the typed text fires verbatim — no identity line, no mutation (bl-6920)"
    );
    // The workspace enters the harness channel and nothing else (§3.3, bl-df65):
    // its path is never in the goal text, only in the workspace argument and the
    // spawn's env.
    assert!(!inv[2].argv[4].contains(ws.to_string_lossy().as_ref()));
    assert!(!inv[2].argv[4].contains("workspace"));
    assert_eq!(
        inv[2].env.get("YOG_NAME"),
        Some(&"home".to_owned()),
        "YOG_NAME layered (§8), the default name verbatim"
    );
    assert!(
        bl_rec.invocations().is_empty(),
        "the bare rung mutates no ball"
    );

    // The ops trail is complete: prime, new, and the detached-prompt spawn line.
    let ops = yog::opslog::tail(state.path(), 16);
    assert_eq!(ops.len(), 3);
    assert_eq!(ops[2].exit, DETACHED_EXIT);
    assert_eq!(ops[2].cwd, ws.display().to_string());
    assert_eq!(
        ops[2].argv[5], "make me a plan",
        "the logged goal is verbatim"
    );
}
