//! The one room both intakes open onto.

use super::*;
use crate::boundary::consumer::ConsumerCtx;
use crate::cli_outbound::Cli;
use crate::ui_state::SystemClock;
use crate::wire::server::Answerer;
use serde_json::json;
use std::path::PathBuf;
use tempfile::tempdir;

/// The in-world caller as a peer — operator grade, which every intake without
/// a certificate is (REMOTE §4.2).
fn local() -> crate::registry::Peer {
    crate::registry::Peer {
        client: crate::registry::Client::local(),
        grade: crate::registry::Grade::Operator,
    }
}

fn intake(state_root: &std::path::Path) -> Intake {
    let snap = Arc::new(crate::app::Snapshot::empty(0));
    Intake::new(Arc::new(ConsumerCtx {
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::test_support::no_world(),
        litany: Cli::new("/no/such/litany"),
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        ui_path: state_root.join("ui.json"),
        cell: crate::state::new_snapshot_cell(snap),
        presence: crate::registry::presence::Presence::default(),
        mailbox: crate::registry::mailbox::Mailbox::default(),
        clock: Arc::new(SystemClock),
    }))
}

/// A request becomes a one-frame reply stream — today's every answer.
#[test]
fn a_request_is_one_reply_frame() {
    let root = tempdir().expect("tmp");
    let stream: Vec<_> = intake(root.path())
        .answer(&local(), json!({"op": "workspaces"}))
        .collect();
    assert_eq!(stream.len(), 1);
    assert_eq!(stream[0]["kind"], "workspaces");
    assert_eq!(stream[0]["ok"], true);
}

/// **The wire adds no verb** (REMOTE §3): a request the codec does not know is
/// refused by the codec, in-band, exactly as a deposited one is.
#[test]
fn an_unknown_verb_refuses_in_band() {
    let root = tempdir().expect("tmp");
    let stream: Vec<_> = intake(root.path())
        .answer(&local(), json!({"op": "teleport"}))
        .collect();
    assert_eq!(stream.len(), 1);
    assert_eq!(stream[0]["ok"], false);
    assert!(
        stream[0]["error"]
            .as_str()
            .unwrap_or_default()
            .contains("teleport"),
        "names the offender: {}",
        stream[0]
    );
}
