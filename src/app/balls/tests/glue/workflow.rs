//! **The §9.4 workflow mark at the engine's chokepoint** (bl-b680), driven
//! against a recorder `litany`: what reaches the substrate is the
//! workspace-bound `litany workflow <ws> <agent> --config <name>` or `--clear`
//! — asserted off the §4.2 trail, which records the argv actually spawned —
//! and a name the world does not hold refuses before anything runs.

use super::{AGENT, fake_litany, model, world};
use crate::boundary::Action;
use crate::boundary::reply::Reply;
use crate::cli_outbound::Cli;
use crate::opslog;
use crate::test_support::engine;
use tempfile::tempdir;

fn act(w: &super::super::World, ts: &str, action: &Action) -> Result<Reply, String> {
    let bin = tempdir().unwrap();
    let (_c, m) = model(w);
    let litany = fake_litany(bin.path());
    let deps = m.boundary_deps(&litany, &Cli::new("/no/bl"));
    engine::act(&m, &deps, ts, action)
}

#[test]
fn the_set_and_the_clear_each_spawn_the_bound_litany_verb() {
    let w = world();
    let ws = w.ws_cobalt.display().to_string();
    for (config, tail) in [
        (Some("strict"), vec!["--config", "strict"]),
        (None, vec!["--clear"]),
    ] {
        let action = Action::Workflow {
            workspace: crate::naming::leaf(&w.ws_cobalt),
            agent: AGENT.into(),
            config: config.map(str::to_owned),
        };
        let Reply::Outcome(outcome) = act(&w, "TW", &action).unwrap() else {
            panic!("a verb answers an outcome");
        };
        assert!(outcome.ok());
        let ops = opslog::tail(&w.roots.yog_state, 4);
        let last = ops.last().expect("the verb is on the trail");
        let mut argv = vec!["workflow", ws.as_str(), AGENT];
        argv.extend(tail);
        assert_eq!(last.argv[1..], argv);
        assert_eq!(last.cwd, ws);
    }
}

/// The chokepoint's one resolution refuses an unknown workspace and an
/// unknown conversation by name, and no verb runs: the trail stays empty.
#[test]
fn an_unknown_workspace_or_conversation_refuses_before_the_verb_runs() {
    let w = world();
    let nowhere = act(
        &w,
        "TW",
        &Action::Workflow {
            workspace: "nowhere".into(),
            agent: AGENT.into(),
            config: Some("strict".into()),
        },
    )
    .expect_err("refused");
    assert!(
        nowhere.contains("unknown workspace \"nowhere\""),
        "{nowhere}"
    );
    let nobody = act(
        &w,
        "TW",
        &Action::Workflow {
            workspace: crate::naming::leaf(&w.ws_cobalt),
            agent: "grey-heron".into(),
            config: None,
        },
    )
    .expect_err("refused");
    assert!(nobody.contains("unknown conversation"), "{nobody}");
    assert!(
        opslog::tail(&w.roots.yog_state, 4).is_empty(),
        "nothing ran"
    );
}
