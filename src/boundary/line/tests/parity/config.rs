//! The **§9 config family's** line parity (bl-719a's carrier, bl-2410's sixth
//! member): every read of that family, through the line and back.
//!
//! Its own file beside `balls`/`inspector`/`policy`/`tools` on the seam those
//! four already draw — one family, one file — and because the parent reached
//! §12's cap when the sixth member arrived.

use super::rt;
use crate::boundary::config::{ConfigFile, Read};
use crate::boundary::{Gesture, Query};

/// The config family's reads (§8.5, bl-0164): the same verb as the write
/// beside them, spelled with nothing after the destination.
#[test]
fn the_config_familys_reads_round_trip() {
    for file in [
        ConfigFile::Brazen {
            workspace: "ws".to_owned(),
        },
        ConfigFile::LitanyModels,
        ConfigFile::Cadence,
        ConfigFile::LitanyWorkflow {
            name: "review".to_owned(),
        },
    ] {
        rt(Gesture::Ask(Query::Config(Read::File { file })));
    }
    // A lineage destination reads too (bl-dff8), in all three of its origins:
    // the same words the write takes, with nothing after them.
    for origin in [
        crate::config_edit::branch::edit::EditOrigin::Advance,
        crate::config_edit::branch::edit::EditOrigin::Orphan,
        crate::config_edit::branch::edit::EditOrigin::Fork {
            source: "base".to_owned(),
        },
    ] {
        rt(Gesture::Ask(Query::Config(Read::File {
            file: ConfigFile::Branch {
                workspace: "ws".to_owned(),
                lineage: "strict".to_owned(),
                origin,
                path: "workflow.yaml".to_owned(),
            },
        })));
    }
    rt(Gesture::Ask(Query::Config(Read::Marks {
        workspace: "ws".to_owned(),
    })));
    rt(Gesture::Ask(Query::Config(Read::Roles {
        workspace: "ws".to_owned(),
    })));
    rt(Gesture::Ask(Query::Config(Read::Providers {
        workspace: "ws".to_owned(),
    })));
    // The browse and the roster beside them (bl-dff8).
    rt(Gesture::Ask(Query::Config(Read::Lineages {
        workspace: "ws".to_owned(),
    })));
    rt(Gesture::Ask(Query::Config(Read::Models {
        workspace: "ws".to_owned(),
        provider: "acme".to_owned(),
    })));
}

/// **The §8.3 sign-in, both halves** (REMOTE §8.3, bl-c285): the act and its
/// lane, in this file because they are the sixth thing a provider row is
/// addressed by — `/providers` lists the rows, `/model` picks off one, and
/// these two sign one in and watch it. Both elide the workspace, which is the
/// seat's, and state the row, which is not.
#[test]
fn the_sign_in_and_its_lane_round_trip() {
    rt(Gesture::Act(crate::boundary::Action::Login {
        workspace: "ws".to_owned(),
        provider: "acme".to_owned(),
    }));
    rt(Gesture::Ask(Query::LoginTail {
        workspace: "ws".to_owned(),
        provider: "acme".to_owned(),
    }));
}

/// Neither takes a bare line: a sign-in with no row named is not a gesture, and
/// a lane with none is not a question. The refusal names the usage rather than
/// guessing at a row, exactly as `/models`' does.
#[test]
fn a_sign_in_with_no_provider_named_refuses_with_its_usage() {
    for verb in ["login", "login-tail"] {
        let refusal = crate::boundary::line::parse(&format!("/{verb}"), &super::super::ctx())
            .expect_err("a row is required");
        assert!(refusal.contains("usage"), "{verb}: {refusal}");
        assert!(
            crate::boundary::line::parse(&format!("/{verb} a b"), &super::super::ctx()).is_err(),
            "{verb}: one word, not a sentence"
        );
    }
}
