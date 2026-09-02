//! The §11 inspector family's decode arms (§8.5, REMOTE §9 step 2, bl-7067) —
//! the reads bl-6233 landed and bl-13f9 extended, split off [`super`] on the seam
//! `codec/query/inspector` already draws: these are the replies addressed at a
//! *conversation* rather than a workspace, and each one's rows are read by the
//! `wire` module that spells them, beside its own type.
//!
//! Nothing is read here but the envelope. The bodies belong to
//! `transcript::wire`, `steps_view::wire`, `files_view::wire`, `rail::wire`,
//! `inboxview::wire`, `workdiff::wire` and `science::wire` — the same modules
//! that write them,
//! for the same reason they write them: those rows' shape *is* each module's
//! vocabulary.

use serde_json::{Map, Value};

use super::super::Reply;
use crate::boundary::codec::fields::{opt_str_of, opt_val, str_of, strings_of, usize_of};

/// The §11 answers, plus the work diff that shares their shape. `None`
/// when the kind is not one of them.
pub(super) fn decode(kind: &str, o: &Map<String, Value>) -> Option<Result<Reply, String>> {
    Some(match kind {
        "transcript" => crate::transcript::wire::decode::transcript(o).map(Reply::Transcript),
        // One frame of the live tail (bl-73e7) — the fold read back by the
        // module that wrote it, so the lane needs no reader of its own.
        super::super::encode::FOLLOW => follow(o),
        "steps" => crate::steps_view::wire::decode::steps(o).map(Reply::Steps),
        "step" => crate::steps_view::wire::decode::detail(o).map(Reply::Step),
        "files" => files(o),
        "rail" => crate::rail::wire::rail_of(o).map(Reply::Rail),
        "governing" => governing(o),
        "inbox" => crate::inboxview::wire::entries_of(o).map(Reply::Inbox),
        "agent" => super::super::agent::view_of(o).map(Reply::Agent),
        "work-diff" => work_diff(o),
        "science" => crate::science::wire::rows_of(o).map(Reply::Science),
        _ => return None,
    })
}

/// One follow frame: the fold, under the one key the encoder writes it at. A
/// frame with no `stream` object at all is a codec that has drifted, not an
/// empty tail — an empty tail is an empty object, which reads as
/// [`Stream::default`](crate::git_tree::Stream).
fn follow(o: &Map<String, Value>) -> Result<Reply, String> {
    let body = o.get("stream").ok_or("follow: missing stream")?;
    let body = body.as_object().ok_or("follow: stream is not an object")?;
    crate::git_tree::stream_wire::stream_of(body).map(Reply::Follow)
}

fn files(o: &Map<String, Value>) -> Result<Reply, String> {
    Ok(Reply::Files {
        view: crate::files_view::wire::view_of(o)?,
        preview: opt_val(o, "preview", crate::files_view::wire::preview_of)?,
        working_dir: crate::files_view::wire::working_dir_of(o)?,
    })
}

/// The which-config-governs answer (bl-13f9; follow-the-tip, bl-e654).
/// `follows` names the lineage whose tip the conversation resolves — the
/// ordinary case — and its **absence is the held one**, which is why the
/// enum is rebuilt off that key alone: `diverged_lineages` is read only where
/// it can be anything but zero, so the two fields cannot decode to a state the
/// encoder could not have written.
fn governing(o: &Map<String, Value>) -> Result<Reply, String> {
    use crate::config_edit::branch::Governance;
    let governance = match opt_str_of(o, "follows")? {
        Some(branch) => Governance::Follows(branch),
        None => Governance::Held {
            diverged_lineages: usize_of(o, "diverged_lineages")?,
        },
    };
    Ok(Reply::Governing(
        crate::config_edit::branch::GoverningConfig {
            oid: str_of(o, "oid")?,
            short_oid: str_of(o, "short_oid")?,
            governance,
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
