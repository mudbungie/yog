//! **The birth role's one channel** (§8.1, bl-9ced): `Prepared::role` reaching
//! the detached child as `litany prompt --role <name>`, and spelling nothing
//! when the start named none.
//!
//! Its own file rather than a case in [`super::lineage`], which both files'
//! sizes would have made a shave: the lineage answers *which config commit
//! governs* and is derived from the ball's tags, while the role answers *which
//! role is resolved out of that commit* and is derived from nothing — it is the
//! seat's own field, set between the prepare and the fire. Two facts, one
//! commit, and the beats do not share a fixture.

use super::prompt::{fire, make_fifo, prepared, workspace};
use super::{World, write_exec};
use crate::cli_outbound::Cli;
use crate::start::execute_prompt;

/// The flag rides when the start names a role and is absent when it does not —
/// an omitted flag is litany's own `worker`, never a word yog writes, which is
/// the discipline `--config` already carries one line above it in the argv.
#[test]
fn the_fire_carries_a_named_role_and_omits_an_unnamed_one() {
    for (role, want) in [
        (Some("planner".to_owned()), vec!["--role", "planner"]),
        (None, vec![]),
    ] {
        let w = World::new();
        let fifo = w.bin.path().join("report");
        make_fifo(&fifo);
        let body = format!(
            "#!/bin/sh\nprintf '%s\\037%s' \"$4\" \"$5\" > '{}'\n",
            fifo.display()
        );
        let litany = Cli::new(write_exec(w.bin.path(), "litany", &body));
        let ws = workspace(&w);
        execute_prompt(
            &litany,
            w.state.path(),
            "TS",
            &crate::start::Fire {
                prepared: crate::start::Prepared {
                    role,
                    ..prepared("cobalt-gecko", None)
                },
                ..fire(&ws, "cobalt-gecko", None, "do it")
            },
            &[],
            &super::rng(),
        )
        .unwrap();
        // argv is `prompt --name <conv> [--role <name>] <ws> <goal>`, and the
        // §4.2 row records the same list the spawn was built from.
        let logged = &w.ops()[0].argv;
        assert_eq!(logged[4..4 + want.len()], want[..], "role {want:?}");
        assert_eq!(
            logged[logged.len() - 2..],
            [ws.to_string_lossy().into_owned(), "do it".to_owned()],
            "the workspace and the goal still trail",
        );
        let recorded = std::fs::read_to_string(&fifo).unwrap();
        let seen: Vec<&str> = recorded.split('\u{1f}').collect();
        let expect: Vec<String> = if want.is_empty() {
            vec![ws.to_string_lossy().into_owned(), "do it".to_owned()]
        } else {
            want.iter().map(|s| (*s).to_owned()).collect()
        };
        assert_eq!(seen, expect);
    }
}

/// **Plan mode, whole**: both config-resolution fields set at once, in the
/// order litany reads them. `--config` leads because the commit has to be
/// chosen before a role can be resolved out of it, and the pair is what a seat
/// deposits when it starts a planning conversation on a lineage of its own.
#[test]
fn a_lineage_and_a_role_ride_together_in_that_order() {
    let w = World::new();
    let ws = workspace(&w);
    let litany = Cli::new(write_exec(w.bin.path(), "litany", "#!/bin/sh\nexit 0\n"));
    execute_prompt(
        &litany,
        w.state.path(),
        "TS",
        &crate::start::Fire {
            prepared: crate::start::Prepared {
                lineage: Some("plan".to_owned()),
                role: Some("planner".to_owned()),
                ..prepared("cobalt-gecko", None)
            },
            ..fire(&ws, "cobalt-gecko", None, "do it")
        },
        &[],
        &super::rng(),
    )
    .unwrap();
    let logged = &w.ops()[0].argv;
    assert_eq!(
        logged[4..8],
        ["--config", "plan", "--role", "planner"],
        "{logged:?}"
    );
}
