//! Enrollment's act: what it mints, what it seats, what it answers, and what it
//! keeps (REMOTE §1.4 as amended, §4.2). The ways it refuses are [`refusals`],
//! split at §12's budget on the seam every other family here is cut on; the QR
//! envelope's measured size is [`envelope`].

mod envelope;
mod refusals;

use super::*;
use crate::boundary::dispatch::Deps;
use crate::boundary::tests::snapshot;
use crate::boundary::{Action, Gesture};
use crate::cli_outbound::Cli;
use crate::registry::Grade;
use rustls::pki_types::CertificateDer;
use rustls::pki_types::pem::PemObject;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

/// The address an operator states — a name and a port, which is what makes it
/// something a device can be handed (§8).
pub(super) const STATED: &str = "engine.invalid:7737";

/// A hermetic world whose wire directory is provisioned at [`STATED`], plus the
/// state root the registration and the trail land in. The mint is the real
/// recipe (`openssl`), because what this act answers with is exactly what that
/// recipe writes.
pub(super) fn provisioned(tmp: &TempDir) -> (Deps, PathBuf) {
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    provision::mint(&dir, STATED, false).expect("the operator's own mint");
    let state_root = tmp.path().join("state-root");
    (deps(&world, &state_root), dir)
}

pub(super) fn deps(world: &crate::xdg::Env, state_root: &Path) -> Deps {
    Deps {
        litany: Cli::new("/no/such/litany"),
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: world.clone(),
        snapshot: Arc::new(snapshot(
            Path::new("/names/alba"),
            "alba",
            Vec::new(),
            Vec::new(),
        )),
        caller: crate::boundary::dispatch::Caller::default(),
    }
}

pub(super) fn request(name: &str, grade: Grade) -> Request {
    Request {
        workspace: "alba".to_owned(),
        name: name.to_owned(),
        grade,
    }
}

pub(super) fn enrolled(reply: Reply) -> Enrolled {
    match reply {
        Reply::Enrolled(enrolled) => enrolled,
        other => panic!("not an enrollment: {other:?}"),
    }
}

/// The act, whole: a leaf minted on this box's CA, the client seated in the
/// named workspace, the material answered, and the trail carrying a row.
#[test]
fn an_enrollment_mints_registers_and_answers() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
    let answer = enrolled(enroll(&deps, "7", &request("phone-1", Grade::Operator)).expect("act"));

    assert_eq!(answer.name, "phone-1");
    assert_eq!(answer.grade, Grade::Operator);
    assert_eq!(answer.address, STATED);
    assert_eq!(answer.ca, read(&dir.join(material::ANCHORS)).expect("ca"));
    assert_eq!(answer.cert, read(&dir.join("phone-1.pem")).expect("leaf"));
    assert!(answer.key.contains("PRIVATE KEY"), "a key, in PEM");

    // The registration is the file, and its existence is the fact (§4.1).
    let client = Client::parse("phone-1").expect("identity");
    assert!(
        crate::registry::registered(&deps.state_root, &client).contains("alba"),
        "seated in the workspace the gesture named"
    );
    // The act is on the trail, and the trail names no material (§4.2).
    let trail = crate::opslog::tail(&deps.state_root, 8);
    let row = trail.first().expect("one row");
    assert_eq!(
        row.argv,
        vec![crate::opslog::YOG_STEP.to_owned(), STEP.into()]
    );
    assert_eq!(row.exit, 0);
    assert!(row.stdout.is_empty() && row.stderr.is_empty());
}

/// **The key is gone from this box the moment the answer is built**, and the
/// certificate is not — it is public material, and its presence is the guard
/// that refuses a second enrollment under one name.
#[test]
fn the_key_is_absent_server_side_and_the_certificate_is_not() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
    enroll(&deps, "7", &request("phone-1", Grade::Operator)).expect("act");

    assert!(!dir.join("phone-1.key").exists(), "shredded");
    assert!(dir.join("phone-1.pem").is_file(), "kept, and public");

    let again = enroll(&deps, "8", &request("phone-1", Grade::Operator)).expect_err("refused");
    assert!(again.contains("already holds"), "{again}");
    assert!(again.contains("re-issuing distrusts nothing"), "{again}");
}

/// **The address is the one a client dials** (§8): the same fact the seat's own
/// material read answers with, never a second spelling.
#[test]
fn the_address_field_matches_what_a_client_dials() {
    let tmp = tempdir().expect("tmp");
    let (deps, dir) = provisioned(&tmp);
    let answer = enrolled(enroll(&deps, "7", &request("phone-1", Grade::Operator)).expect("act"));
    let dialled = material::read_dir(&dir, material::Role::Client)
        .expect("readable")
        .expect("provisioned")
        .address;
    assert_eq!(answer.address, dialled);
}

/// The grade is minted into the subject by this box's own CA (§4.2), so what
/// the device presents later reads back as the grade that was asked for.
#[test]
fn a_foot_is_minted_as_a_foot_and_reads_back_as_one() {
    let tmp = tempdir().expect("tmp");
    let (deps, _) = provisioned(&tmp);
    for (name, grade) in [("laptop", Grade::Operator), ("builder", Grade::Foot)] {
        let answer = enrolled(enroll(&deps, "7", &request(name, grade)).expect("act"));
        assert_eq!(answer.grade, grade);
        let der = CertificateDer::from_pem_slice(answer.cert.as_bytes()).expect("pem");
        assert_eq!(crate::registry::leaf::grade(&der), grade, "{name}");
        assert_eq!(
            crate::registry::leaf::common_name(&der).as_deref(),
            Some(name)
        );
    }
}

/// **A foot may not enroll, and no code here says so** (REMOTE §4.2): the foot
/// set is enumerated, an act is not in it, and the raise at `answer_as` is what
/// refuses. This is the assertion that the structure holds — if a later edit
/// ever admitted an act into that set, this is what reddens.
#[test]
fn a_foot_grade_caller_is_refused_structurally() {
    let gesture = Gesture::Act(Action::Enroll(request("phone-1", Grade::Operator)));
    assert!(
        !Grade::Foot.admits(&gesture),
        "an act is outside the foot set: advertise, invocations, complete"
    );
    assert!(Grade::Operator.admits(&gesture));
}

/// **Through the chokepoint**, which is the only path any face takes: the
/// workspace name resolves once ahead of the table (REMOTE §8), the arm runs,
/// and what comes back is the receipt. A gesture naming a workspace this caller
/// cannot see never reaches the executor at all.
#[test]
fn the_chokepoint_routes_it_and_refuses_an_unknown_workspace() {
    let tmp = tempdir().expect("tmp");
    let (deps, _) = provisioned(&tmp);
    let act = |name: &str, ws: &str| {
        crate::boundary::dispatch::dispatch(
            &deps,
            &mut crate::ui_state::UiState::open(tmp.path().join("ui.json")),
            "7",
            &Action::Enroll(Request {
                workspace: ws.to_owned(),
                name: name.to_owned(),
                grade: Grade::Operator,
            }),
        )
    };
    let answer = enrolled(act("phone-1", "alba").expect("routed"));
    assert_eq!(answer.name, "phone-1");

    let refusal = act("phone-2", "elsewhere").expect_err("unresolvable");
    assert!(refusal.contains("elsewhere"), "{refusal}");
}
