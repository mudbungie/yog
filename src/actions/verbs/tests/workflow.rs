//! The §9.4 workflow mark's argv (bl-b680): the set names its lineage with
//! `--config`, the clear says `--clear`, and both run in the workspace under
//! the conversation origin — asserted off the logged row, as every verb is.

use super::*;

#[test]
fn the_set_builds_argv_with_the_lineage_and_the_clear_with_the_flag() {
    let w = World::new("litany", OK_BODY);
    let ws = w.cwd.display().to_string();
    workflow(&w.bound(), w.state.path(), "TS", "a-1", Some("strict")).unwrap();
    let e = w.logged();
    assert_eq!(
        args_of(&e),
        vec!["workflow", &ws, "a-1", "--config", "strict"]
    );
    assert_eq!(e.cwd, ws);
    assert_eq!(e.origin, Origin::Conversation);

    let w = World::new("litany", OK_BODY);
    let ws = w.cwd.display().to_string();
    workflow(&w.bound(), w.state.path(), "TS", "a-1", None).unwrap();
    let e = w.logged();
    assert_eq!(args_of(&e), vec!["workflow", &ws, "a-1", "--clear"]);
}
