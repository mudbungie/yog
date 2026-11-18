//! **A sign-in on a registered machine's row** (REMOTE §5.7, bl-7467): the act
//! reads the flow off *that machine's* catalog, asked through the real
//! mailbox, and spawns the same child a bare row gets — whose command line
//! carries the qualified name for the `bz` arm to route.

use std::path::PathBuf;

use serde_json::json;
use tempfile::tempdir;

use super::{deps_with, script, settle, workspace};
use crate::boundary::Action;
use crate::boundary::dispatch::{Caller, Deps, dispatch};
use crate::boundary::reply::Reply;
use crate::config_edit::brazen::CATALOG;
use crate::test_support::foot::{mailbox, parked, printed, seat, serve};
use crate::ui_state::UiState;

fn sign_in(deps: &Deps, provider: &str) -> Result<Reply, String> {
    let mut ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let act = Action::Login {
        workspace: crate::naming::leaf(&workspace()),
        provider: provider.to_owned(),
    };
    dispatch(deps, &mut ui, "100", &act)
}

#[test]
fn the_flow_of_a_far_row_is_read_off_that_machines_own_catalog() {
    let dir = tempdir().expect("tmp");
    // The child echoes the command line it was fired with.
    let bz = script(
        dir.path(),
        "bz",
        "#!/bin/sh\nprintf '%s\\n' \"$*\" 1>&2\nexit 0\n",
    );
    let mail = mailbox();
    let deps = deps_with(dir.path(), &bz);
    let deps = Deps {
        caller: Caller {
            mailbox: mail.clone(),
            ..deps.caller.clone()
        },
        ..deps
    };
    seat(
        &deps.state_root,
        "laptop",
        &crate::naming::leaf(&workspace()),
        &["bz"],
    );

    // Nobody is waiting for work: the flow cannot be read, so nothing starts.
    let asleep = sign_in(&deps, "sub@laptop").expect_err("refused");
    assert!(asleep.contains("\"laptop\" is not waiting"), "{asleep}");

    let catalog = r#"{"providers":[{"name":"sub","auth":"oauth2","device":"rfc8628"},
                                   {"name":"web","auth":"oauth2"}]}"#;
    let (foot, asked) = serve(&mail, "laptop", vec![Some(printed(catalog)); 2]);
    for (provider, line) in [
        ("sub@laptop", "--login --provider sub@laptop"),
        ("web@laptop", "--login --provider web@laptop --browser"),
    ] {
        parked(&mail, "laptop");
        assert!(sign_in(&deps, provider).is_ok());
        assert_eq!(asked.recv().expect("asked"), json!({ "args": CATALOG }));
        let said = settle(&deps, provider);
        assert_eq!(said.lines.first().map(|l| l.text.as_str()), Some(line));
    }
    foot.join().expect("foot");
}
