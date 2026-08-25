//! **Where a workspace lives on THIS box, if anywhere** (REMOTE §8.2,
//! bl-e349) — the three answers [`AppModel::start_path`] gives, and the
//! §8.2 fact underneath them: an entry's workspace has no local path at all.

use super::super::tests::Harness;
use crate::boundary::reply::{Reply, Workspaces, WsRow};
use crate::wire::channel::Channel;
use crate::wire::channels::Channels;
use crate::wire::entries::Entry;
use crate::wire::link::LinkEnd;

/// A provisioned entry naming `leaf` here and there — the ordinary
/// provisioning, where the two names agree.
fn entry(leaf: &str) -> Entry {
    Entry {
        leaf: leaf.to_owned(),
        workspace: leaf.to_owned(),
        channel: Ok(crate::wire::material::Material {
            address: "127.0.0.1:7737".to_owned(),
            anchors: std::path::PathBuf::new(),
            chain: std::path::PathBuf::new(),
            key: std::path::PathBuf::new(),
        }),
    }
}

/// One row as a host answers it — enough to put the name in the union roster.
fn listing(name: &str) -> Reply {
    Reply::Workspaces(Workspaces {
        rows: vec![WsRow {
            workspace: name.to_owned(),
            kind: crate::binding::WorkspaceKind::Foreign,
            attention: 0,
            agents: 0,
            running: false,
            pinned: None,
            config_tip: None,
        }],
        stale: None,
        growth: None,
    })
}

/// The asker's pass on one channel, minus the socket: declare the roster,
/// answer it, declare again so the answer lands.
fn answered(set: &mut Channels, end: &mut LinkEnd, reply: &Reply) {
    set.roster();
    set.settle();
    for question in end.standing() {
        end.publish(&question, Ok(reply.clone()));
    }
    set.roster();
    set.settle();
}

/// The whole of it in one drive: a name an entry holds has **no** local path, a
/// name this box enumerates has its own, and a name nobody holds is the place a
/// `Prepare` would found it — the §3.1 names root, which is what the §11 raise
/// and the bootstrap `home` have always resolved to.
#[test]
fn only_a_workspace_this_box_could_hold_has_a_path_here() {
    let h = Harness::new();
    let (_c, mut rig) = h.model();
    let (local, mut local_end) = crate::wire::link::pair();
    let (remote, mut remote_end) = crate::wire::link::pair();
    let mut set = Channels::held(local, vec![Channel::entry(&entry("cobalt"), remote)]);
    answered(&mut set, &mut remote_end, &listing("cobalt"));
    answered(&mut set, &mut local_end, &listing("ws"));
    rig.model.adopt_wire(set);

    assert_eq!(
        rig.model.hosting_entry("cobalt").as_deref(),
        Some("cobalt"),
        "the union paints which engine holds it"
    );
    assert_eq!(
        rig.model.start_path("cobalt"),
        None,
        "and a workspace held there has no directory on this box at all"
    );
    assert_eq!(
        rig.model.start_path("ws"),
        Some(h.ws.clone()),
        "an enumerated name is its own path"
    );
    assert_eq!(
        rig.model.hosting_entry("fresh"),
        None,
        "a name no channel holds is nobody's yet"
    );
    assert_eq!(
        rig.model.start_path("fresh"),
        Some(crate::binding::names_root(&h.roots.yog_data).join("fresh")),
        "so it is the place a Prepare would found it — the §3.1 names root"
    );
}

/// **The bounce this ball is about** (bl-e349): with a workspace an entry hosts
/// focused, the start's target is that workspace — not the §3.1 bootstrap
/// `home` the old path-first resolution substituted for it, which is a *local*
/// name and founded a local wall.
#[test]
fn a_remote_focus_aims_the_start_at_itself_and_never_at_home() {
    let h = Harness::new();
    let (_c, mut rig) = h.model();
    let (local, _local_end) = crate::wire::link::pair();
    let (remote, mut remote_end) = crate::wire::link::pair();
    let mut set = Channels::held(local, vec![Channel::entry(&entry("cobalt"), remote)]);
    answered(&mut set, &mut remote_end, &listing("cobalt"));
    rig.model.adopt_wire(set);
    rig.model.focus_workspace("cobalt");

    assert_eq!(
        rig.model.focused_workspace(),
        None,
        "the premise: no local path, exactly as REMOTE §8.2 rules"
    );
    assert_eq!(
        rig.model.start_workspace_name(),
        "cobalt",
        "and the start is still addressed at what the operator is looking at"
    );
    assert_eq!(
        crate::naming::leaf(&rig.model.start_bare_inputs().workspace),
        "cobalt",
        "so the act the composer's Enter posts names it (Action::Prepare routes by that name)"
    );
    assert_eq!(
        rig.model.start_path("cobalt"),
        None,
        "and founds nothing here"
    );
}

/// The other side of the same rule, unchanged: nothing focused is the empty
/// world, and only there does the start take the §3.1 default.
#[test]
fn nothing_focused_is_the_only_state_that_takes_the_default_name() {
    let h = Harness::pristine();
    let (_c, rig) = h.model();
    assert_eq!(rig.model.focused_ws_name(), None);
    assert_eq!(rig.model.start_workspace_name(), "home");
    assert_eq!(
        rig.model.start_workspace(),
        crate::binding::names_root(&h.roots.yog_data).join("home"),
        "the bootstrap's own path, which EnsureWorkspace founds"
    );
}
