//! Reading `<row>@<client>` against the workspace's registered clients, and
//! the seam's decision on a whole command line (REMOTE §5.7).

use super::super::*;
use super::names;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn a_bare_row_is_not_a_qualification() {
    assert_eq!(split("anthropic", &names(&["laptop"])), None);
    assert_eq!(split("anthropic", &[]), None);
}

#[test]
fn a_qualified_row_names_its_row_and_its_client() {
    assert_eq!(
        split("claude_sub@laptop", &names(&["phone", "laptop"])),
        Some(("claude_sub".to_owned(), "laptop".to_owned()))
    );
}

/// A machine registered elsewhere is absent, so the name is just a row the
/// linked brazen has never heard of — and a joiner with no row in front of it
/// qualifies nothing.
#[test]
fn a_suffix_no_registered_client_bears_is_a_bare_row() {
    assert_eq!(split("sub@desktop", &names(&["laptop"])), None);
    assert_eq!(split("@laptop", &names(&["laptop"])), None);
    assert_eq!(split("sublaptop", &names(&["laptop"])), None);
}

#[test]
fn where_two_registered_names_fit_the_longer_wins() {
    assert_eq!(
        split("sub@home@laptop", &names(&["laptop", "home@laptop"])),
        Some(("sub".to_owned(), "home@laptop".to_owned()))
    );
}

/// A world whose state root holds `laptop` registered in `alba`, read from
/// inside `wall`'s workspace.
pub(super) fn world(root: &Path, wall: &str) -> Env {
    let env = Env::from_pairs([
        ("XDG_STATE_HOME", root.to_string_lossy().into_owned()),
        (crate::world::wall::YOG_WALL, format!("/walls/{wall}")),
    ]);
    let laptop = Client::parse("laptop").expect("client");
    crate::registry::register(&env.yog_state_root(), &laptop, "alba").expect("register");
    env
}

#[test]
fn the_seam_routes_only_a_call_naming_a_row_on_a_client_registered_here() {
    let root = TempDir::new().expect("tmp");
    let here = world(root.path(), "alba");
    let call = names(&["--json", "--provider", "sub@laptop"]);
    let far = of(&call, &here).expect("routed");
    assert_eq!(far.client, "laptop");
    assert_eq!(far.argv, names(&["--json", "--provider", "sub"]));
    assert_eq!(far.state_root, here.yog_state_root());
    // Registered in another workspace: absent, so the linked brazen's.
    assert!(of(&call, &world(root.path(), "brine")).is_none());
    assert!(far.events, "the data plane answers in canonical events");
    // No wall, no row, no value, and a route that names no row.
    assert!(of(&call, &here.without(crate::world::wall::YOG_WALL)).is_none());
    assert!(of(&names(&["--json"]), &here).is_none());
    assert!(of(&names(&["--json", "--provider"]), &here).is_none());
    assert!(of(&names(&["--list-providers", "--json"]), &here).is_none());
    assert!(of(&names(&["--json", "--provider", "sub"]), &here).is_none());
}

/// A model listing and a sign-in name a row too, so they take the same leg
/// (bl-7467) — with every other word of the command line passed as typed.
#[test]
fn a_listing_and_a_sign_in_naming_a_far_row_are_routed_too() {
    let root = TempDir::new().expect("tmp");
    let here = world(root.path(), "alba");
    for line in [
        &["--list-models", "--provider", "sub@laptop", "--json"][..],
        &["--login", "--provider", "sub@laptop", "--browser"][..],
    ] {
        let far = of(&names(line), &here).expect("routed");
        assert_eq!(far.client, "laptop");
        assert_eq!(far.argv[2], "sub");
        assert_eq!(
            (far.argv[0].as_str(), far.argv[3].as_str()),
            (line[0], line[3])
        );
        assert!(!far.events);
    }
}
