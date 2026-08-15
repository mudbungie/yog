//! The §11 inspector family's decode arms (§8.5, REMOTE §9 step 2, bl-7067) —
//! the reads bl-6233 landed and bl-13f9 extended, split off [`super`] on the seam
//! `codec/query/inspector` already draws: these are the replies addressed at a
//! *conversation* rather than a workspace, and each one's rows are read by the
//! `wire` module that spells them, beside its own type.
//!
//! Nothing is read here but the envelope. The bodies belong to
//! `transcript::wire`, `steps_view::wire`, `files_view::wire`, `rail::wire`,
//! `inboxview::wire` and `workdiff::wire` — the same modules that write them,
//! for the same reason they write them: those rows' shape *is* each module's
//! vocabulary.

use serde_json::{Map, Value};

use super::super::Reply;
use crate::boundary::codec::fields::{opt_str_of, opt_val, str_of, strings_of};

/// The §11 answers, plus the work diff that shares their shape. `None`
/// when the kind is not one of them.
pub(super) fn decode(kind: &str, o: &Map<String, Value>) -> Option<Result<Reply, String>> {
    Some(match kind {
        "transcript" => crate::transcript::wire::decode::transcript(o).map(Reply::Transcript),
        "steps" => crate::steps_view::wire::decode::steps(o).map(Reply::Steps),
        "step" => crate::steps_view::wire::decode::detail(o).map(Reply::Step),
        "files" => files(o),
        "rail" => crate::rail::wire::rail_of(o).map(Reply::Rail),
        "governing" => governing(o),
        "inbox" => crate::inboxview::wire::entries_of(o).map(Reply::Inbox),
        "agent" => super::super::agent::view_of(o).map(Reply::Agent),
        "work-diff" => work_diff(o),
        _ => return None,
    })
}

fn files(o: &Map<String, Value>) -> Result<Reply, String> {
    Ok(Reply::Files {
        view: crate::files_view::wire::view_of(o)?,
        preview: opt_val(o, "preview", crate::files_view::wire::preview_of)?,
    })
}

/// The config-frozen-at answer (bl-13f9). `branch` is the lineage whose tip the
/// governing commit still *is*, and its absence is the ordinary frozen case —
/// the field is optional here for the same reason it is `Option` on the type.
fn governing(o: &Map<String, Value>) -> Result<Reply, String> {
    Ok(Reply::Governing(
        crate::config_edit::branch::GoverningConfig {
            oid: str_of(o, "oid")?,
            short_oid: str_of(o, "short_oid")?,
            branch_name_if_tip_of_one: opt_str_of(o, "branch")?,
            files: strings_of(o, "files")?,
        },
    ))
}

fn work_diff(o: &Map<String, Value>) -> Result<Reply, String> {
    Ok(Reply::WorkDiff {
        attempts: crate::workdiff::wire::attempts_of(o)?,
        patch: opt_val(o, "patch", crate::files_view::wire::preview_of)?,
    })
}
