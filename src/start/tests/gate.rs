//! The §8.1 provider gate (bl-1fd0): the pure read over brazen's `credential`
//! column, and the three states the start pane's first rung has.

use crate::config_edit::brazen::provider_rows;
use crate::start::{StartGate, WallCredit};

/// A bare wall's real table, verbatim from `bz --list-providers --json` against
/// an empty config home: five keyed rows `missing`, `ollama` and `claude-code`
/// keyless, and the one built-in oauth row with nothing signed in. **This is
/// the wall the ruling was filed from**, which is why the fixture is the whole
/// listing rather than a hand-made pair of rows — the rung either appears here
/// or it appears nowhere.
const BARE_WALL: &str = r#"{"providers":[
    {"auth":"api_key","credential":"missing","name":"anthropic","protocol":"anthropic_messages"},
    {"auth":"bearer","credential":"missing","name":"openai","protocol":"openai_chat"},
    {"auth":"none","credential":"not required","name":"ollama","protocol":"ollama_chat"},
    {"auth":"none","credential":"not required","name":"claude-code","protocol":"claude_code"},
    {"auth":"oauth2","credential":"missing","name":"openai-chatgpt","protocol":"openai_responses"}
]}"#;

/// The same wall after one browser sign-in: the oauth row reads `stored`.
const SIGNED_WALL: &str = r#"{"providers":[
    {"auth":"api_key","credential":"missing","name":"anthropic","protocol":"anthropic_messages"},
    {"auth":"none","credential":"not required","name":"ollama","protocol":"ollama_chat"},
    {"auth":"oauth2","credential":"stored","name":"openai-chatgpt","protocol":"openai_responses"}
]}"#;

#[test]
fn the_credential_column_is_projected_verbatim() {
    let rows = provider_rows(BARE_WALL);
    assert_eq!(
        rows.iter()
            .map(|r| r.credential.clone())
            .collect::<Vec<_>>(),
        [
            "missing",
            "missing",
            "not required",
            "not required",
            "missing"
        ]
    );
}

#[test]
fn a_credential_spelling_this_build_cannot_read_is_not_a_refusal() {
    // An absent column folds to an empty string, and brazen's `inline` and
    // `ambient` are spellings the ruling's own enumeration never listed. None
    // of the three is `missing`, so none of them refuses: no surface here
    // refuses on the strength of a question that went unanswered.
    for column in ["", "\"ambient\"", "\"inline\"", "\"a-newer-spelling\""] {
        let json = if column.is_empty() {
            r#"{"providers":[{"name":"x","auth":"oauth2"}]}"#.to_owned()
        } else {
            format!(r#"{{"providers":[{{"name":"x","auth":"oauth2","credential":{column}}}]}}"#)
        };
        let credit = WallCredit::read(&provider_rows(&json));
        assert!(credit.credentialed, "credential {column:?} must not refuse");
        assert_eq!(StartGate::read(None, credit), StartGate::Ready);
    }
}

#[test]
fn a_signed_wall_is_ready_and_the_pane_adds_nothing() {
    let credit = WallCredit::read(&provider_rows(SIGNED_WALL));
    assert_eq!(
        credit,
        WallCredit {
            credentialed: true,
            keyless: true
        },
        "the keyless rows are still there — a credential elsewhere is what readies it"
    );
    let gate = StartGate::read(None, credit);
    assert_eq!(gate, StartGate::Ready);
    assert_eq!(gate.note(), None, "today's flow, byte for byte");
    assert_eq!(gate.refusal(), None);
    assert!(!gate.roster());
}

/// The ruling's literal predicate — *"any row `stored` or `not required`"* —
/// calls this wall usable, because brazen merges `ollama` and `claude-code`
/// under every config there can be. The rung has to appear here or it appears
/// nowhere: this IS the wall the first goal was wasted on.
#[test]
fn the_bare_wall_the_ruling_was_filed_from_refuses_and_says_why() {
    let credit = WallCredit::read(&provider_rows(BARE_WALL));
    assert_eq!(
        credit,
        WallCredit {
            credentialed: false,
            keyless: true
        }
    );
    let gate = StartGate::read(None, credit);
    assert!(gate.roster(), "the sign-in roster is the remedy");
    let why = gate.refusal().expect("Send says the blocker");
    assert!(why.contains("reaches no model"), "{why}");
    assert!(why.contains("your draft is kept"), "{why}");

    let note = gate.note().expect("and the rung says it above the box");
    assert!(note.starts_with(&why), "the rung leads with the same words");
    assert!(
        note.contains("keyless rows are reached only by an explicit provider"),
        "and then names what the operator can see but must not count on:\n{note}"
    );
}

/// The keyless clause is a fact about the wall, not decoration: a table with no
/// keyless row says only the first half.
#[test]
fn a_wall_with_no_keyless_row_says_only_the_refusal() {
    let rows = provider_rows(
        r#"{"providers":[{"auth":"api_key","credential":"missing","name":"anthropic"}]}"#,
    );
    let gate = StartGate::read(None, WallCredit::read(&rows));
    assert_eq!(gate.note(), gate.refusal());
}

/// A wall brazen could not be asked about has no rows, and a wall with no rows
/// has nothing to route to — the same refusal, reached with no branch of its
/// own (the general path over an empty input).
#[test]
fn an_unanswerable_wall_is_the_empty_table_not_a_case() {
    let credit = WallCredit::read(&[]);
    assert_eq!(credit, WallCredit::default());
    assert!(StartGate::read(None, credit).refusal().is_some());
}

/// bl-61bf's seam. This box reads its own wall's brazen; for a workspace hosted
/// by a §8.2 entry that is the wrong table, so the gate answers with the fact
/// it cannot answer rather than with the local rows.
#[test]
fn a_remote_workspace_is_an_unknown_never_a_refusal() {
    let doomed = WallCredit::default();
    let gate = StartGate::read(Some("box-two".to_owned()), doomed);
    assert_eq!(gate, StartGate::Unknown("box-two".to_owned()));
    assert_eq!(
        gate.refusal(),
        None,
        "an unread wall is not a wall known to be empty"
    );
    assert!(
        !gate.roster(),
        "and the local roster belongs to another wall"
    );
    let note = gate.note().expect("said honestly");
    assert!(note.contains("\"box-two\""), "{note}");
    assert!(note.contains("cannot be read from here yet"), "{note}");
}
