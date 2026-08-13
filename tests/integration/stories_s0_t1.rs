//! STORIES **S0-T1** bootstrap-bare-start: an empty world, one Enter → the world
//! materializes. `lernie prime`, `lernie new <names-root>/home` — the §3.1
//! default name, taken without asking, so the first Enter meets no name picker —
//! and a detached `lernie prompt` carrying the typed text **verbatim** (bl-6920:
//! identity rides `--name`, never the payload), `YOG_NAME=home`, cwd `~`; the
//! pre-submit view-model already carried the greyed name prediction (STORIES
//! S0.2, DESIGN §3.1/§3.3/§3.4/§8.1).

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::binding::{names_root, workspace_path};
use yog::cli_outbound::Cli;
use yog::names::{DEFAULT_NAME, SplitMix64};
use yog::start::{self, DETACHED_EXIT, Deps, Payload, StartInputs};

#[test]
fn s0_t1_empty_world_one_enter_materializes_the_conversation() {
    let (bin, state) = (tempdir().unwrap(), tempdir().unwrap());
    let (yog, balls, home) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let lernie_rec = Recorder::new(bin.path(), "lernie").authoring_workspaces();
    let bl_rec = Recorder::new(bin.path(), "bl");
    let deps = Deps {
        bl: Cli::new(bl_rec.path()),
        lernie: Cli::new(lernie_rec.path()),
        state_root: state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
        // No answer from brazen: the §9.2 birth-template gate judges nothing.
    };
    let inputs = StartInputs {
        // The empty world's target: the fixed default name (§3.1), a constant —
        // not a config, not a mint, and nothing the operator was asked for.
        workspace: workspace_path(yog.path(), DEFAULT_NAME),
        payload: Payload::Bare,
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    // Pre-submit: the greyed name prediction, a pure read (nothing spawned yet).
    let composer = start::preview(&inputs, &mut SplitMix64::from_seed(7));
    assert!(composer.preview.starts_with("will be named "));
    assert_eq!(
        bl_rec.invocations().len(),
        0,
        "the preview spawns nothing (I7)"
    );

    // Enter: seed + new, then the detached prompt with the typed text.
    let prepared = start::prepare(&deps, &inputs, "T0").unwrap();
    let name = prepared.name.clone();
    assert_eq!(
        name, DEFAULT_NAME,
        "the bootstrap names without asking (§3.1)"
    );
    assert_eq!(prepared.cwd, home.path(), "bare cwd is ~");
    let ws = workspace_path(yog.path(), &name);
    assert_eq!(prepared.workspace, ws);
    // The conversation mint re-derives at fire off a fresh generator on the same
    // held seed the preview used — so the greyed prediction is the fired name (§3.3).
    let minted = start::execute_prompt(
        &deps.lernie,
        state.path(),
        "T0",
        &prepared,
        "make me a plan",
        &[],
        &mut SplitMix64::from_seed(7),
    )
    .unwrap();

    // The full argv sequence: prime, new <names-root>/home, prompt — the pinned
    // template already grants the worker role's whole tool pool (§8.1,
    // bl-7fc8), so nothing advances config/default a second time.
    let inv = lernie_rec.wait(3);
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
    // lernie-stored name fact (bl-49cb).
    assert_eq!(
        minted, conversation,
        "the name the fire returns is the name --name carries — one mint, one channel (§3.3)"
    );
    assert_eq!(
        inv[2].argv[4], "make me a plan",
        "the typed text fires verbatim — no identity line, no mutation (bl-6920)"
    );
    assert_eq!(
        composer.preview,
        format!("will be named {conversation}"),
        "the pre-submit view-model already predicted this exact name (S0-T1)"
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
    assert_eq!(ops[2].cwd, home.path().display().to_string());
    assert_eq!(
        ops[2].argv[5], "make me a plan",
        "the logged goal is verbatim"
    );
}
