//! The §9 config family's halves of the [`codec`](super) (bl-3f46, bl-0164):
//! the family's op envelopes in both directions — the apply, the §16.3 marks
//! amendment, the §9.4 model pick and tuning pair, the §9.6 settle, and the
//! six reads beside them. Split from the top-level codec per §12's line
//! budget; the **destination** those envelopes carry in `target` is
//! [`file`]'s, split off in turn at the same cap (bl-edfc). Every encoder here
//! is matched by a decoder and every variant round-trips (the §8.5 parity
//! tests).
//!
//! Strict, like the rest of the codec: an unknown destination token, a missing
//! parameter, a `fork` with no source each refuse by name. A config apply
//! rewrites a file that governs every model call in a workspace — a guessed
//! destination is the last thing it may do.

use crate::boundary::Action;
use crate::boundary::config::Write;
use crate::proposals::{Settle, Verdict};
use serde_json::{Map, Value, json};

use super::fields::{bool_of, opt_str_of};
use super::str_of;
use crate::model_pick::{LEVELS, Tuning};

mod file;
pub(super) use file::{decode_file, encode_file};

/// Read one of the family's three ops, or `None` when the token is not one —
/// which is what keeps the unknown-op refusal in a single place upstream.
/// **Write-shaped only**: [`super::query`] tries a query reading first
/// (`config`/`marks` with no `text`/`mode` field is the §8.5 read, bl-0164)
/// and this is its fallback, so by the time either op reaches here it always
/// carries the field that makes it a write.
pub(super) fn decode_action(op: &str, o: &Map<String, Value>) -> Option<Result<Action, String>> {
    match op {
        "config" => Some(decode_apply(o)),
        "marks" => Some(decode_marks(o)),
        "model" => Some(decode_pick(o)),
        PROPOSAL => Some(decode_settle(o)),
        EFFORT => Some(decode_effort(o)),
        PRIORITY => Some(decode_priority(o)),
        _ => None,
    }
}

fn decode_apply(o: &Map<String, Value>) -> Result<Action, String> {
    Ok(write(Write::Apply {
        file: decode_file(o.get("target").ok_or("config: missing target")?)?,
        text: str_of(o, "text")?,
    }))
}

fn decode_marks(o: &Map<String, Value>) -> Result<Action, String> {
    let branch = str_of(o, "branch")?;
    if !crate::world::marks::lawful(&branch) {
        return Err(format!("marks: {}", crate::world::marks::REFUSAL));
    }
    Ok(write(Write::Marks {
        workspace: str_of(o, "workspace")?,
        branch,
    }))
}

/// The §9.4 tuning pair's two op words (bl-23bd) — the operator's vocabulary,
/// which is also litany's config key and also the slash verb, so one word
/// serves the line, the wire and the file.
pub(super) const EFFORT: &str = "effort";
pub(super) const PRIORITY: &str = "priority";

/// One tuning gesture's envelope. Two ops off one carrier: `effort` carries the
/// level or `null` for off, `priority` carries the boolean — each field the
/// shape its own arm has, rather than a shared `value` that would have to be
/// two types.
pub(super) fn encode_tuning(tuning: &Tuning) -> Value {
    match tuning {
        Tuning::Effort {
            workspace,
            role,
            level,
        } => json!({ "op": EFFORT, "workspace": workspace, "role": role,
                     "level": level.map(|l| l.as_str()) }),
        Tuning::Priority {
            workspace,
            role,
            on,
        } => json!({ "op": PRIORITY, "workspace": workspace, "role": role, "on": on }),
    }
}

/// `/effort`'s level, read back **strictly** — the [`decode_marks`] shape, and
/// for its reason: the vocabulary is closed, so a word outside it is a codec
/// that has drifted rather than an operator's typo, and answering it in band
/// costs one sentence. `null` is `off`, and absence is the same reading: the
/// encoder writes the key always, and a peer that omits it has said the one
/// thing an absent optional can honestly mean.
fn decode_effort(o: &Map<String, Value>) -> Result<Action, String> {
    let level = match opt_str_of(o, "level")? {
        None => None,
        Some(word) => Some(
            crate::model_pick::Effort::parse(&word)
                .ok_or_else(|| format!("{EFFORT}: level must be one of {LEVELS}, got {word:?}"))?,
        ),
    };
    Ok(write(Write::Tune(Tuning::Effort {
        workspace: str_of(o, "workspace")?,
        role: str_of(o, "role")?,
        level,
    })))
}

fn decode_priority(o: &Map<String, Value>) -> Result<Action, String> {
    Ok(write(Write::Tune(Tuning::Priority {
        workspace: str_of(o, "workspace")?,
        role: str_of(o, "role")?,
        on: bool_of(o, "on")?,
    })))
}

fn decode_pick(o: &Map<String, Value>) -> Result<Action, String> {
    Ok(write(Write::Pick {
        workspace: str_of(o, "workspace")?,
        role: str_of(o, "role")?,
        provider: str_of(o, "provider")?,
        model: str_of(o, "model")?,
    }))
}

/// The §9.6 settle's op word (bl-dd88) — singular, where the read's is plural:
/// one word tells an act from a listing at a glance, and the pair is the
/// operator's own vocabulary as well as litany's verb.
pub(super) const PROPOSAL: &str = "proposal";

/// One settle's envelope. The verdict is a **word** and not a boolean: accept
/// and reject are two acts, not two values of one, and the day a third settling
/// exists a boolean would have to become one anyway.
pub(super) fn encode_settle(settle: &Settle) -> Value {
    json!({ "op": PROPOSAL, "workspace": settle.workspace,
            "id": settle.id, "verdict": settle.verdict.word() })
}

/// Read one back, **strictly** on the verdict — the [`decode_effort`] shape and
/// its reason: the vocabulary is closed, so a word outside it is a codec that
/// has drifted rather than an operator's typo.
fn decode_settle(o: &Map<String, Value>) -> Result<Action, String> {
    let word = str_of(o, "verdict")?;
    let verdict = Verdict::parse(&word)
        .ok_or_else(|| format!("{PROPOSAL}: verdict must be one of accept|reject, got {word:?}"))?;
    Ok(write(Write::Proposal(Settle {
        workspace: str_of(o, "workspace")?,
        id: str_of(o, "id")?,
        verdict,
    })))
}

/// The §9 write family's five spellings, off the one carrier the roster holds
/// (bl-dd88) — here rather than as five rows of `codec`'s own match, for the
/// reason `encode_config` is: a family whose grammar is already one subject
/// spells itself in one place.
pub(super) fn encode_write(w: &Write) -> Value {
    match w {
        Write::Apply { file, text } => {
            json!({ "op": "config", "target": encode_file(file), "text": text })
        }
        Write::Marks { workspace, branch } => {
            json!({ "op": "marks", "workspace": workspace, "branch": branch })
        }
        Write::Pick {
            workspace,
            role,
            provider,
            model,
        } => json!({ "op": "model", "workspace": workspace,
                     "role": role, "provider": provider, "model": model }),
        Write::Tune(tuning) => encode_tuning(tuning),
        Write::Proposal(settle) => encode_settle(settle),
    }
}

/// One §9 write, in the variant that carries them all.
fn write(w: Write) -> Action {
    Action::Config(w)
}

#[cfg(test)]
pub(crate) mod tests;

/// The §9 family's six READ spellings, off the one carrier the roster holds
/// (bl-719a) — here beside the write family's (bl-dd88), rather than in the
/// query roster, for the reason `codec::balls::encode` is: a family whose
/// grammar is already one subject spells itself in one place, and the roster
/// names the family once on each side.
pub(super) fn encode_read(read: &crate::boundary::config::Read) -> Value {
    use crate::boundary::config::Read;
    match read {
        Read::File { file } => {
            json!({ "op": "config", "target": encode_file(file) })
        }
        Read::Marks { workspace } => json!({ "op": "marks", "workspace": workspace }),
        Read::Providers { workspace } => json!({ "op": "providers", "workspace": workspace }),
        Read::Roles { workspace } => json!({ "op": "roles", "workspace": workspace }),
        Read::Lineages { workspace } => json!({ "op": "lineages", "workspace": workspace }),
        Read::Models {
            workspace,
            provider,
        } => json!({ "op": "models", "workspace": workspace, "provider": provider }),
        // The id is **absent** rather than null for the bare listing (bl-dd88):
        // absence is the fact — nothing was named — and a null would be a
        // second spelling of it.
        Read::Proposals { workspace, id } => match id {
            Some(id) => json!({ "op": PROPOSALS, "workspace": workspace, "id": id }),
            None => json!({ "op": PROPOSALS, "workspace": workspace }),
        },
    }
}

/// The §9.6 staged-proposal read's op token (bl-dd88), named once for both
/// directions. Plural, because the read is the listing and naming one is a
/// depth of it — the settle is the singular `proposal`, and the two words tell
/// a read from an act at a glance.
pub(super) const PROPOSALS: &str = "proposals";
