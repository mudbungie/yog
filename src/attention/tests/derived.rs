//! **The §6 signals yog derives from its own reads**, and the words they
//! are said in: `refused` (bl-b43b — the latest model call's failure, read off
//! the response tail the §3.5 classifier already opened), `truncated` (bl-ebef
//! — the `length` finish on that same tail) and `flagged`
//! (bl-6f2f — the signal-out verb's ops row). Every other signal is a
//! `refs/litany/*` mark or the inbox listing, which is the seam this file is
//! split from [`super::signals`] on (DESIGN §12).

use super::{acked, agent, nothing};
use crate::attention::*;
use crate::git_tree::AgentState;
use crate::ui_state::SeenKind;

/// An auth-shaped failure sentence: what makes `Agent::refused()` — the
/// reading, not a second stored flag — answer true (bl-9b88).
const AUTH_SHAPED: &str = r#"{"type":"error","status":401,"message":"Unauthorized"}"#;

/// **Rule 7** (bl-6f2f): a raised flag is attention, in its own word, and it
/// acknowledges exactly as an oid-backed signal does — the stamp is the
/// watermark, so `/seen` quiets the flag you answered and a later one fires
/// again. This is the join the defect was missing: the verb wrote its ops row
/// and §6 read no ops row, so the monitor's floor grant signalled into silence.
#[test]
fn a_raised_flag_is_attention_and_its_stamp_is_the_watermark() {
    let mut ag = agent("a");
    ag.flagged = Some(crate::monitor::Flag {
        at: "7".into(),
        reason: "please look at this one".into(),
    });
    let att = attention(&ag, &[], "ws", &nothing);
    assert!(att.flagged && att.any());
    assert_eq!(att.kinds(), vec![AttentionKind::Flagged]);
    assert_eq!(
        evidence(&ag),
        vec![(SeenKind::Flag, "7".to_string())],
        "the raising row's stamp is what an acknowledgement records"
    );
    // Acknowledged, it is quiet…
    assert!(!attention(&ag, &[], "ws", &acked(SeenKind::Flag, "7")).flagged);
    // …and a *later* flag is a later stamp, so it asks again.
    ag.flagged = Some(crate::monitor::Flag {
        at: "8".into(),
        reason: "and again".into(),
    });
    assert!(attention(&ag, &[], "ws", &acked(SeenKind::Flag, "7")).flagged);
    // A watermark of another kind never quiets it.
    assert!(attention(&ag, &[], "ws", &acked(SeenKind::Stopped, "8")).flagged);
}

/// The word carries what the operator is being asked for — a look, and a
/// reason to read.
#[test]
fn the_flagged_word_says_a_human_should_look() {
    let said = AttentionKind::Flagged.says();
    assert!(said.contains("flagged"), "{said}");
    assert!(said.contains("reason"), "{said}");
}

/// **Rule 2's rest, said in the word that is true of it** (bl-b43b). A
/// conversation refused at the provider rung comes to rest `Stopped` exactly
/// as an operator's own `/stop` does — the badge set is frozen at four — so
/// `stopped` would tell the operator they did a thing they did not do.
#[test]
fn a_refused_rest_earns_the_refused_word_and_not_stoppeds() {
    let mut ag = agent("a");
    ag.state = AgentState::Stopped;
    let mine = attention(&ag, &[], "ws", &nothing);
    assert_eq!(mine.kinds(), vec![AttentionKind::Stopped]);

    ag.failure = Some(AUTH_SHAPED.to_owned());
    let theirs = attention(&ag, &[], "ws", &nothing);
    assert_eq!(theirs.kinds(), vec![AttentionKind::Refused]);

    // One firing, not two: the refinement changes the word, never the count,
    // so a refused conversation is not asked about twice.
    assert_eq!(mine.any(), theirs.any());
    assert!(theirs.stopped, "it is still rule 2 that fired");
}

/// The word carries the remedy, because the remedy is not a gesture on the
/// conversation at all — which is what the desktop escalation says out loud.
#[test]
fn the_refused_word_names_the_act() {
    let said = AttentionKind::Refused.says();
    assert!(said.contains("refused at the provider"), "{said}");
    assert!(said.contains("sign a provider in"), "{said}");
}

/// The refinement is **gated on the rest firing**: an acknowledged tip stirs
/// nothing, refused or not, so a refusal cannot smuggle a signal past the
/// watermark.
#[test]
fn an_acked_refused_rest_still_stirs_nothing() {
    let mut ag = agent("a");
    ag.state = AgentState::Stopped;
    ag.failure = Some(AUTH_SHAPED.to_owned());
    assert!(
        attention(&ag, &[], "ws", &acked(SeenKind::Stopped, "tip-a"))
            .kinds()
            .is_empty()
    );
}

/// **Three conditions must not read as one row** (bl-511d). A conversation
/// that finished, one that stopped without finishing and one parked on an
/// answer all put your turn on the queue — rule 2 is unchanged and the tokens
/// are unchanged — but the sentence has to tell them apart, and it did not: a
/// birth that died in its first step said *"came to rest — your turn"*, which
/// is the same words a clean turn-end says.
#[test]
fn a_wounded_rest_does_not_say_it_came_to_rest() {
    let clean = crate::attention::row_says(&[AttentionKind::Stopped], AgentState::Quiescent);
    let wounded = crate::attention::row_says(&[AttentionKind::Stopped], AgentState::Stopped);
    assert_eq!(clean, "came to rest — your turn");
    assert_eq!(wounded, "stopped without finishing — your turn");

    // Every other clause is a fact about the signal alone and reads the same
    // either way — the state refines rule 2's word and nothing else's.
    for state in [AgentState::Quiescent, AgentState::Stopped] {
        assert_eq!(
            crate::attention::row_says(&[AttentionKind::Mail], state),
            AttentionKind::Mail.says()
        );
    }
    // And the clauses join in signal order, one row, one sentence.
    assert_eq!(
        crate::attention::row_says(
            &[AttentionKind::Stopped, AttentionKind::Held],
            AgentState::Stopped
        ),
        format!(
            "stopped without finishing — your turn; {}",
            AttentionKind::Held.says()
        )
    );
}

/// **Nothing but an answer clears a park** (§6 rule 6), so the clause names
/// the gesture that gives one. Every other rule states a condition an operator
/// can act on from the row; this one used to state a condition and leave the
/// verb to be known.
#[test]
fn the_held_word_names_the_verb_that_releases_it() {
    let said = AttentionKind::Held.says();
    assert!(said.contains("/answer"), "{said}");
    assert!(said.contains("pass"), "{said}");
    assert!(said.contains("refuse"), "{said}");
    // The tool is not repeated into the sentence: `held.tool` is on the row.
    assert!(!said.contains("bash"), "{said}");
}

/// **Rule 2's rest, said in the other true word** (bl-ebef). litany 0.0.11
/// fails a turn cut off at the request's output cap
/// (`Error::OutputTruncated`): the staging sink is never sealed, so nothing is
/// committed and no tool call runs. It settles `Stopped` like every other
/// wound, so `stopped` told the operator their conversation had finished and
/// handed them a turn — the exact sentence the upstream ball measured while a
/// learning loop staged nothing across nine calls.
#[test]
fn a_truncated_rest_earns_the_truncated_word_and_not_stoppeds() {
    let mut ag = agent("a");
    ag.state = AgentState::Stopped;
    assert_eq!(
        attention(&ag, &[], "ws", &nothing).kinds(),
        vec![AttentionKind::Stopped]
    );

    ag.truncated = true;
    let cut = attention(&ag, &[], "ws", &nothing);
    assert_eq!(cut.kinds(), vec![AttentionKind::Truncated]);
    assert!(cut.stopped, "it is still rule 2 that fired");

    // A refusal outranks it, and the pair is mutually exclusive on disk anyway
    // — a truncation frames COMPLETE and so carries no failure sentence, while
    // a failure carries no `finish` to read a reason off. One word either way,
    // never two firings.
    ag.failure = Some(AUTH_SHAPED.to_owned());
    assert_eq!(
        attention(&ag, &[], "ws", &nothing).kinds(),
        vec![AttentionKind::Refused]
    );
}

/// The word carries the remedy litany's own error names — the config edit, or
/// a smaller ask — and says outright that nothing was committed, because the
/// operator's first question about a cut-off turn is what survived it.
#[test]
fn the_truncated_word_names_the_remedy_and_says_nothing_landed() {
    let said = AttentionKind::Truncated.says();
    assert!(said.contains("cut off at its output cap"), "{said}");
    assert!(said.contains("nothing was committed"), "{said}");
    assert!(said.contains("max_output_tokens"), "{said}");
}
