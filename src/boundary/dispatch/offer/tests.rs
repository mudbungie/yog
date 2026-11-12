//! The offer's four acts: what may be offered, what the rows say, what may be
//! approved, and what a decline leaves behind.

use super::*;
use crate::boundary::tests::snapshot;
use crate::cli_outbound::Cli;
use crate::registry::offer::claim::claim;
use crate::test_support::wire::Device;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

const TS: &str = "1000";

/// A world whose wire is stated at a host, and the state root beside it.
fn provisioned(tmp: &TempDir, address: &str) -> (Deps, PathBuf) {
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    provision::mint(&dir, address, &[], false).expect("the operator's own mint");
    let deps = Deps {
        litany: Cli::new("/no/such/litany"),
        bl: Cli::new("/no/such/bl"),
        state_root: tmp.path().join("state-root"),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        yog_binary: PathBuf::from("/no/such/yog"),
        world,
        snapshot: Arc::new(snapshot(
            Path::new("/names/alba"),
            "alba",
            Vec::new(),
            Vec::new(),
        )),
        caller: crate::boundary::dispatch::Caller::default(),
    };
    (deps, dir)
}

fn offer(name: &str, grade: Grade) -> Verb {
    Verb::Mint {
        workspace: "alba".to_owned(),
        name: name.to_owned(),
        grade,
    }
}

fn named(name: &str) -> (String, String) {
    ("alba".to_owned(), name.to_owned())
}

fn approve(name: &str) -> Verb {
    let (workspace, name) = named(name);
    Verb::Approve { workspace, name }
}

fn decline(name: &str) -> Verb {
    let (workspace, name) = named(name);
    Verb::Decline { workspace, name }
}

fn rows(deps: &Deps, workspace: &str) -> Vec<Row> {
    let list = Verb::List {
        workspace: workspace.to_owned(),
    };
    match dispatch(deps, TS, &list).expect("listed") {
        Answer::Offers(rows) => rows,
        other => panic!("not a listing: {other:?}"),
    }
}

/// The code out of an `offered` answer's URL.
fn minted(deps: &Deps, name: &str, grade: Grade) -> String {
    match dispatch(deps, TS, &offer(name, grade)).expect("offered") {
        Answer::Offered { url, expires } => {
            assert_eq!(expires, 1000 + offer::LIFE);
            let code = url
                .strip_prefix("https://engine.invalid/yog/register/")
                .expect("the first stated host, and the door's path");
            assert_eq!(code.len(), offer::CODE_LEN);
            code.to_owned()
        }
        other => panic!("not an offer: {other:?}"),
    }
}

/// The offer's whole life: minted open, claimed with a word, proved by the
/// device speaking, approved into a registration — and gone from the list.
#[test]
fn an_offer_is_minted_claimed_proved_and_approved() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp, "engine.invalid:7737");
    let root = deps.state_root.clone();
    let phone = Client::parse("phone").expect("identity");
    let code = minted(&deps, "phone", Grade::Foot);
    let open = rows(&deps, "alba");
    assert_eq!(
        (open[0].state, open[0].word.clone(), open[0].grade),
        (State::Open, None, Grade::Foot)
    );
    assert!(
        rows(&deps, "elsewhere").is_empty(),
        "another workspace's list"
    );
    let unproved = dispatch(&deps, TS, &approve("phone")).expect_err("still open");
    assert!(unproved.contains("not proved"), "{unproved}");

    claim(&root, &dir, 1001, &code, &Device::new().body()).expect("claimed");
    let claimed = rows(&deps, "alba");
    assert_eq!(claimed[0].state, State::Claimed);
    assert_eq!(
        claimed[0].word.as_deref().map(str::len),
        Some(offer::WORD_LEN)
    );
    assert!(
        dispatch(&deps, TS, &approve("phone")).is_err(),
        "claimed is not proved"
    );
    // A stamp from before the claim is somebody else's: only speaking since
    // then proves this leaf's holder.
    registry::seen::mark(&root, &phone, 1000);
    assert_eq!(rows(&deps, "alba")[0].state, State::Claimed);
    registry::seen::mark(&root, &phone, 1001);
    assert_eq!(rows(&deps, "alba")[0].state, State::Proved);

    assert_eq!(dispatch(&deps, TS, &approve("phone")), Ok(Answer::Approved));
    assert!(registry::registered(&root, &phone).contains("alba"));
    assert!(
        !store::held(&root, &phone),
        "the hold ends with the approval"
    );
    assert!(rows(&deps, "alba").is_empty());
    let trail = std::fs::read_to_string(root.join("ops.jsonl")).expect("the trail");
    for step in ["offer", "claim", "approve"] {
        assert!(trail.contains(&format!("\"{step}\"")), "{step}: {trail}");
    }
    // The name is now a seated client's, and is not offered a second time.
    assert!(dispatch(&deps, TS, &offer("phone", Grade::Foot)).is_err());
}

/// **A name is offerable only while nothing was ever issued under it.** Each
/// refusal is ahead of the code, so no code exists for an offer nobody could
/// safely claim.
#[test]
fn a_name_that_is_not_fresh_is_not_offered() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp, "engine.invalid:7737");
    let root = deps.state_root.clone();
    let refused = |name: &str| dispatch(&deps, TS, &offer(name, Grade::Operator)).expect_err(name);

    assert!(refused("a/b").contains("unusable client identity"));
    // This box's own seats: founding a workspace seats them by name.
    assert!(refused("yog-client").contains("own seat"));
    assert!(refused(registry::WINDOW).contains("own seat"));
    // A name registered by hand, with no leaf here to say so.
    let carried = Client::parse("carried").expect("identity");
    registry::register(&root, &carried, "alba").expect("registered");
    assert!(refused("carried").contains("registered already"));
    // A name whose leaf `wire-certs` already minted.
    provision::issue(&dir, "laptop", Grade::Operator).expect("a hand-carried leaf");
    assert!(refused("laptop").contains("already holds"));
    assert!(
        rows(&deps, "alba").is_empty(),
        "nothing was minted for any of them"
    );

    // And a box that could not sign refuses before a code exists.
    std::fs::remove_file(dir.join(provision::CA_KEY)).expect("rm");
    assert!(refused("phone").contains(provision::CA_KEY));
}

/// An address that names no host a device could reach mints no offer: a `:0`
/// is a request only the listener learns the answer to.
#[test]
fn a_box_with_no_stated_host_offers_nothing() {
    let tmp = tempdir().expect("tmp");
    let (deps, _dir) = provisioned(&tmp, "127.0.0.1:0");
    let refusal = dispatch(&deps, TS, &offer("phone", Grade::Operator)).expect_err("no host");
    assert!(refusal.contains("WIRE_HOST"), "{refusal}");
}

/// **A decline voids an open offer and spends a claimed one's name.** The
/// certificate a claim issued cannot be recalled, so its hold is kept for
/// good, and a later offer under that name — whose approval would seat the
/// declined holder too — is refused.
#[test]
fn a_decline_frees_an_open_name_and_spends_a_claimed_one() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp, "engine.invalid:7737");
    let root = deps.state_root.clone();
    let phone = Client::parse("phone").expect("identity");
    let missing = dispatch(&deps, TS, &decline("phone")).expect_err("nothing to decline");
    assert!(
        missing.contains("no pending offer for phone in alba"),
        "{missing}"
    );

    let stale = minted(&deps, "phone", Grade::Operator);
    assert_eq!(dispatch(&deps, TS, &decline("phone")), Ok(Answer::Declined));
    assert!(rows(&deps, "alba").is_empty());
    assert_eq!(
        claim(&root, &dir, 1001, &stale, &Device::new().body()),
        None
    );

    // Free again: the same name is offered, and this time claimed first.
    let code = minted(&deps, "phone", Grade::Operator);
    claim(&root, &dir, 1001, &code, &Device::new().body()).expect("claimed");
    assert_eq!(dispatch(&deps, TS, &decline("phone")), Ok(Answer::Declined));
    assert!(
        rows(&deps, "alba").is_empty(),
        "a declined offer is not pending"
    );
    assert!(store::held(&root, &phone), "its leaf is held for good");
    assert!(
        dir.join("phone.pem").is_file(),
        "and still says the name is spent"
    );
    assert!(dispatch(&deps, TS, &offer("phone", Grade::Operator)).is_err());
    assert!(
        dispatch(&deps, TS, &approve("phone")).is_err(),
        "nothing left to approve"
    );
}

/// A store that cannot be written refuses the mint and the decline, each
/// naming the file — never a code or a receipt for something not recorded.
#[test]
fn an_unwritable_store_refuses_the_act() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp, "engine.invalid:7737");
    let root = deps.state_root.clone();
    let code = minted(&deps, "phone", Grade::Operator);
    claim(&root, &dir, 1001, &code, &Device::new().body()).expect("claimed");
    let held = root.join(store::DIR).join("held");
    std::fs::remove_file(held.join("phone")).expect("rm");
    std::fs::create_dir(held.join("phone")).expect("a directory where the record goes");
    assert!(
        rows(&deps, "alba").is_empty(),
        "an unreadable record is no offer"
    );

    let blocked = tempdir().expect("tmp");
    let (deps, _dir) = provisioned(&blocked, "engine.invalid:7737");
    std::fs::create_dir_all(&deps.state_root).expect("root");
    std::fs::write(deps.state_root.join(store::DIR), "a file").expect("block");
    let refusal = dispatch(&deps, TS, &offer("phone", Grade::Operator)).expect_err("unwritable");
    assert!(refusal.contains("phone"), "{refusal}");
}

/// **Through the chokepoint**, which is the only path any face takes, and
/// **operator grade only without a line here saying so** (REMOTE §4.2): the
/// workspace resolves once ahead of the table, the arm runs, and none of the
/// four is in the set a foot may say.
#[test]
fn the_chokepoint_routes_it_and_a_foot_may_say_none_of_the_four() {
    use crate::boundary::reply::Reply;
    use crate::boundary::{Action, Gesture};
    let tmp = tempdir().expect("tmp");
    let (deps, _dir) = provisioned(&tmp, "engine.invalid:7737");
    let act = |verb: Verb| {
        crate::boundary::dispatch::dispatch(
            &deps,
            &mut crate::ui_state::UiState::open(tmp.path().join("ui.json")),
            TS,
            &Action::Offer(verb),
        )
    };
    let offered = act(offer("phone", Grade::Operator)).expect("routed");
    assert!(
        matches!(offered, Reply::Offer(Answer::Offered { .. })),
        "{offered:?}"
    );
    let elsewhere = Verb::List {
        workspace: "elsewhere".to_owned(),
    };
    assert!(
        act(elsewhere.clone())
            .expect_err("unresolvable")
            .contains("elsewhere")
    );

    for verb in [
        offer("phone", Grade::Foot),
        elsewhere,
        approve("phone"),
        decline("phone"),
    ] {
        let gesture = Gesture::Act(Action::Offer(verb));
        assert!(!Grade::Foot.admits(&gesture), "{gesture:?}");
        assert!(Grade::Operator.admits(&gesture));
    }
}
