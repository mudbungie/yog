//! The **query** half of the envelope codec (§8.5), cut from the action half
//! at §12's per-file budget on the family seam [`super::config`] already
//! established: [`super`] holds the action roster and the shared field
//! readers, this holds every populating read's spelling.
//!
//! Both directions stay exhaustive over [`Query`], so the §4.8 compile gate is
//! unchanged — a query variant added tomorrow does not build until it is
//! spelled here.
//!
//! **`config` and `marks` are shared tokens, not query-exclusive (bl-0164).**
//! Both ops answer either family — a `text`/`mode` field present is the
//! write, absent is the read — so [`read`] recognizes them only in their
//! fieldless shape and falls through (`Ok(None)`) otherwise, letting
//! [`super::config::decode_action`] answer the write. The line reads this
//! same discriminant off an empty tail, so a seat cannot spell one meaning
//! at the envelope and the other at the line.

use serde_json::{Map, Value, json};

use super::fields::opt_str_of;
use super::start::opt_field;
use super::{obj, str_of, usize_of};
use crate::boundary::Query;
use crate::boundary::config::Read;

/// The §11 inspector family's own spelling (bl-6233, bl-13f9) — the queries
/// addressed at a conversation rather than a workspace.
mod inspector;

/// Encode one query to its envelope. Total over [`Query`].
pub(super) fn encode(query: &Query) -> Value {
    match query {
        Query::Workspaces => json!({ "op": "workspaces" }),
        Query::Conversations { workspace } => {
            json!({ "op": "conversations", "workspace": workspace })
        }
        Query::Balls => json!({ "op": "balls" }),
        // The same facts one workspace deep (bl-b4b5) — the address is the
        // whole difference, so the envelope says only that.
        Query::WorkspaceBalls { workspace } => {
            json!({ "op": WORKSPACE_BALLS, "workspace": workspace })
        }
        Query::WorkDiff { workspace, file } => {
            let mut map = obj(&[("op", "work-diff")]);
            map.insert("workspace".to_owned(), json!(workspace));
            if let Some(file) = file {
                let mut f = obj(&[]);
                f.insert("ball".to_owned(), json!(file.ball));
                // Absent for the ordinary claim attempt (bl-c2bd): only a fan
                // candidate's patch needs the row named one level deeper.
                if let Some(handle) = &file.handle {
                    f.insert("handle".to_owned(), json!(handle));
                }
                f.insert("path".to_owned(), json!(file.path));
                map.insert("file".to_owned(), Value::Object(f));
            }
            Value::Object(map)
        }
        // The §11 inspector family (bl-6233): one address, written once —
        // what differs between them rides beside it, never instead of it.
        Query::Transcript { workspace, agent } => inspector::at("transcript", workspace, agent),
        // The follow lane's read (bl-73e7): the same address, the same fold,
        // and a cadence the envelope says nothing about — how many frames an
        // answer becomes is the intake's, never the question's.
        Query::Follow { workspace, agent } => inspector::at(inspector::FOLLOW, workspace, agent),
        Query::Steps { workspace, agent } => inspector::at("steps", workspace, agent),
        Query::Rail { workspace, agent } => inspector::at("rail", workspace, agent),
        Query::Agent { workspace, agent } => inspector::at("agent", workspace, agent),
        Query::Inbox { workspace, agent } => inspector::at("inbox", workspace, agent),
        Query::Step {
            workspace,
            agent,
            seq,
        } => inspector::step(workspace, agent, seq),
        Query::Files {
            workspace,
            agent,
            path,
            at,
        } => inspector::files(workspace, agent, path.as_ref(), at.as_ref()),
        Query::Governing {
            workspace,
            agent,
            at,
        } => inspector::governing(workspace, agent, at.as_ref()),
        // The projection over the work-diff's attempts (§3.9, bl-40ab): one
        // address and nothing else, because the join takes no parameter — what
        // it answers about is every attempt the workspace holds.
        Query::Science { workspace } => json!({ "op": SCIENCE, "workspace": workspace }),
        Query::Board => json!({ "op": "board" }),
        Query::Attention => json!({ "op": "attention" }),
        Query::Ops { max } => json!({ "op": "ops", "max": max }),
        Query::Search { text } => json!({ "op": "search", "text": text }),
        Query::Help { verb } => {
            let mut map = obj(&[("op", "help")]);
            opt_field(&mut map, "verb", verb.as_ref());
            Value::Object(map)
        }
        // The §9 config family (bl-719a), each member spelled below. Its
        // destination read shares its WRITE's op: a `text`/`mode` field is what
        // makes an envelope a write, so a read is spelled by leaving it out,
        // never by a second op token (§8.5, bl-0164).
        Query::Config(read) => encode_config(read),
        Query::Clients { workspace } => {
            json!({ "op": "clients", "workspace": workspace })
        }
        // The routing leg's two reads (bl-024b). The follow-class one names
        // nothing at all: the queue it drains is the intake's own.
        Query::Invocations => json!({ "op": INVOCATIONS }),
        Query::Capture { invocation } => {
            json!({ "op": CAPTURE, "invocation": invocation })
        }
    }
}

/// The §9 family's five spellings, off the one carrier the roster holds
/// (bl-719a) — here rather than as five rows of the match above for the reason
/// `codec::balls::encode` is: a family whose grammar is already one subject
/// spells itself in one place, and the roster names the family once.
fn encode_config(read: &crate::boundary::config::Read) -> Value {
    use crate::boundary::config::Read;
    match read {
        Read::File { file } => {
            json!({ "op": "config", "target": super::config::encode_file(file) })
        }
        Read::Marks { workspace } => json!({ "op": "marks", "workspace": workspace }),
        Read::Providers { workspace } => json!({ "op": "providers", "workspace": workspace }),
        Read::Lineages { workspace } => json!({ "op": "lineages", "workspace": workspace }),
        Read::Models {
            workspace,
            provider,
        } => json!({ "op": "models", "workspace": workspace, "provider": provider }),
    }
}

/// The §11 balls section's own read (bl-b4b5), named once for both directions.
const WORKSPACE_BALLS: &str = "workspace-balls";

/// The §3.9 projection's op token (bl-40ab), named once for both directions.
pub(super) const SCIENCE: &str = "science";

/// The routing leg's read tokens, named once for the encoder and the arm.
pub(super) const INVOCATIONS: &str = "invocations";
pub(super) const CAPTURE: &str = "capture";

/// Decode `op` as a query, or `None` when it names none — the signal
/// [`super::decode`] chains on before it refuses an unknown op, exactly as it
/// chains on the config family's own reader. The two shapes are separated so
/// the reader below can `?` its field refusals: "not a query" and "a query
/// with a bad field" are different answers, and only the second is an error.
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Option<Result<Query, String>> {
    match read(op, o) {
        Ok(query) => query.map(Ok),
        Err(reason) => Some(Err(reason)),
    }
}

/// The query table itself: `Ok(None)` is "some other family's op".
fn read(op: &str, o: &Map<String, Value>) -> Result<Option<Query>, String> {
    // The conversation-addressed family reads first, in its own table
    // (bl-6233); an op it does not claim falls through to this one unchanged.
    if let Some(query) = inspector::read(op, o)? {
        return Ok(Some(query));
    }
    Ok(Some(match op {
        "workspaces" => Query::Workspaces,
        "conversations" => Query::Conversations {
            workspace: str_of(o, "workspace")?,
        },
        "balls" => Query::Balls,
        WORKSPACE_BALLS => Query::WorkspaceBalls {
            workspace: str_of(o, "workspace")?,
        },
        "work-diff" => Query::WorkDiff {
            workspace: str_of(o, "workspace")?,
            file: work_file(o)?,
        },
        SCIENCE => Query::Science {
            workspace: str_of(o, "workspace")?,
        },
        "board" => Query::Board,
        "attention" => Query::Attention,
        "ops" => Query::Ops {
            max: usize_of(o, "max")?,
        },
        "search" => Query::Search {
            text: str_of(o, "text")?,
        },
        // Strict here too: help *about* something must name a gesture, so the
        // answer is total and no seat renders an empty page.
        "help" => Query::Help {
            verb: match opt_str_of(o, "verb")? {
                Some(verb) if !crate::boundary::help::known(&verb) => {
                    return Err(format!("help: unknown verb {verb:?}"));
                }
                other => other,
            },
        },
        // Read-shaped only (bl-0164): present without the write's own field,
        // else `Ok(None)` falls through to `config::decode_action`'s write.
        "config" if !o.contains_key("text") => Query::Config(Read::File {
            file: super::config::decode_file(o.get("target").ok_or("config: missing target")?)?,
        }),
        "marks" if !o.contains_key("branch") => Query::Config(Read::Marks {
            workspace: str_of(o, "workspace")?,
        }),
        "providers" => Query::Config(Read::Providers {
            workspace: str_of(o, "workspace")?,
        }),
        // REMOTE §5's roster (bl-4e08): who is registered here, who is live,
        // and what each advertises.
        "clients" => Query::Clients {
            workspace: str_of(o, "workspace")?,
        },
        // The tool host's follow-class read, and the asker's poll (bl-024b).
        INVOCATIONS => Query::Invocations,
        CAPTURE => Query::Capture {
            invocation: str_of(o, "invocation")?,
        },
        "lineages" => Query::Config(Read::Lineages {
            workspace: str_of(o, "workspace")?,
        }),
        // The provider is required, with no default: a roster is a question
        // *about* one row, and guessing the row would answer about another
        // provider's models entirely.
        "models" => Query::Config(Read::Models {
            workspace: str_of(o, "workspace")?,
            provider: str_of(o, "provider")?,
        }),
        _ => return Ok(None),
    }))
}

/// The optional `file` object of a work-diff query — both of its fields
/// required once it is present, because a patch read that guessed either half
/// would open the wrong file.
fn work_file(obj: &Map<String, Value>) -> Result<Option<crate::workdiff::WorkFile>, String> {
    let Some(value) = obj.get("file") else {
        return Ok(None);
    };
    let file = value.as_object().ok_or("file: not a JSON object")?;
    Ok(Some(crate::workdiff::WorkFile {
        ball: str_of(file, "ball")?,
        handle: opt_str_of(file, "handle")?,
        path: str_of(file, "path")?,
    }))
}
