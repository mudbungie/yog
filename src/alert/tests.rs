//! The §6 escalation's two rules: what a queue row becomes, and what counts as
//! having *arrived*.

use super::{Alert, Announced, announce, of_queue};
use crate::attention::AttentionKind;
use crate::boundary::answer::queue::QueueRow;
use crate::git_tree::AgentState;

fn row(ws: &str, display: &str, signals: Vec<AttentionKind>) -> QueueRow {
    QueueRow {
        workspace: ws.to_string(),
        agent: "20260802T120000Z-root".to_string(),
        display: display.to_string(),
        state: AgentState::Quiescent,
        uncertain: false,
        signals,
        preview: "please ping".to_string(),
        age_secs: 42,
        pending: 0,
        held: None,
        failure: None,
    }
}

fn alert(summary: &str, body: &str) -> Alert {
    Alert {
        summary: summary.to_string(),
        body: body.to_string(),
    }
}

/// A row becomes the sentence the desktop shows: the wall it is in and the
/// conversation it is, then every firing rule in badge order.
#[test]
fn a_queue_row_names_its_workspace_its_conversation_and_its_rules() {
    let alerts = of_queue(&[
        row("cobalt", "ochre-tern", vec![AttentionKind::Stopped]),
        row(
            "slate",
            "jade-vole",
            vec![AttentionKind::Notify, AttentionKind::Mail],
        ),
    ]);
    assert_eq!(
        alerts,
        vec![
            alert("cobalt · ochre-tern", "came to rest — your turn"),
            alert(
                "slate · jade-vole",
                "raised a notify mark; has mail queued and no driver taking it"
            ),
        ]
    );
}

/// A row with no firing signal is not announced at all — there is nothing to
/// say — and the summary is the row's own two words, the wall's §3.1 name and
/// the conversation's §3.3 display, with nothing derived in between (bl-22ab:
/// the row carries the name, so the escalation no longer shortens a path).
#[test]
fn a_signal_less_row_says_nothing_and_a_named_row_says_its_two_words() {
    assert!(of_queue(&[row("cobalt", "ochre-tern", Vec::new())]).is_empty());
    let alerts = of_queue(&[row("cobalt", "ochre-tern", vec![AttentionKind::Held])]);
    assert_eq!(
        alerts,
        vec![alert(
            "cobalt · ochre-tern",
            "parked a tool invocation for your answer"
        )]
    );
}

/// A window that has just opened announces nothing: everything already waiting
/// was waiting before it existed, so the first fold is the baseline.
#[test]
fn the_first_fold_is_the_baseline_and_announces_nothing() {
    let mut announced = Announced::default();
    let waiting = vec![alert("cobalt · ochre-tern", "came to rest — your turn")];
    assert!(announced.arrivals(waiting.clone()).is_empty());
    // …and the very same ask, still unanswered, is still not an arrival.
    assert!(announced.arrivals(waiting).is_empty());
}

/// What arrives is what the operator has not already been told: a new
/// conversation, or the same one now saying something different. An
/// acknowledged row leaves the queue and, when it re-arms, is new again.
#[test]
fn only_a_sentence_the_operator_has_not_heard_is_an_arrival() {
    let rest = alert("cobalt · ochre-tern", "came to rest — your turn");
    let mail = alert(
        "cobalt · jade-vole",
        "has mail queued and no driver taking it",
    );
    let both = alert(
        "cobalt · ochre-tern",
        "came to rest — your turn; exhausted its budget",
    );
    let mut announced = Announced::default();
    assert!(announced.arrivals(vec![rest.clone()]).is_empty());

    // A second conversation joins: only it is announced.
    assert_eq!(
        announced.arrivals(vec![rest.clone(), mail.clone()]),
        vec![mail.clone()]
    );
    // A second rule on the first: a changed sentence is a new thing to say.
    assert_eq!(
        announced.arrivals(vec![both.clone(), mail.clone()]),
        vec![both]
    );
    // Acknowledged — the row leaves the queue — then re-arms: new again.
    assert!(announced.arrivals(vec![mail.clone()]).is_empty());
    assert_eq!(announced.arrivals(vec![rest.clone(), mail]), vec![rest]);
}

/// The baseline advances even when the caller announces nothing (the window had
/// focus, or the knob is off). A signal that landed while you were looking must
/// not be announced later as though it had just arrived.
#[test]
fn a_discarded_arrival_is_not_re_announced_when_the_window_is_buried_again() {
    let rest = alert("cobalt · ochre-tern", "came to rest — your turn");
    let mut announced = Announced::default();
    announced.arrivals(Vec::new());
    // Arrived once (the caller may or may not have announced it) …
    assert_eq!(announced.arrivals(vec![rest.clone()]), vec![rest.clone()]);
    // … and never again while it says the same thing.
    assert!(announced.arrivals(vec![rest]).is_empty());
}

/// The rule wording has one home, and every rule has one: the six §6 signals
/// are six distinct clauses, so an alert naming two of them reads as two facts.
#[test]
fn every_signal_words_itself_distinctly() {
    let all = [
        AttentionKind::Notify,
        AttentionKind::Stopped,
        AttentionKind::Budget,
        AttentionKind::Conflicted,
        AttentionKind::Mail,
        AttentionKind::Held,
    ];
    let said: std::collections::BTreeSet<&str> = all.iter().map(|k| k.says()).collect();
    assert_eq!(said.len(), all.len(), "six rules, six sentences");
    assert!(said.iter().all(|s| !s.is_empty()));
}

/// The two gates, over the one fold: focus and the §4.1 knob each suppress the
/// *announcing* while the baseline advances regardless — so news the operator
/// already had is never replayed the moment the window is buried or the knob
/// is switched on.
#[test]
fn focus_and_the_knob_each_silence_the_desktop_without_saving_the_news_up() {
    let waiting = [row("cobalt", "ochre-tern", vec![AttentionKind::Stopped])];
    let arrival = alert("cobalt · ochre-tern", "came to rest — your turn");

    // Buried and armed: the ordinary path. Baseline first, then the arrival.
    let mut armed = Announced::default();
    assert!(announce(&mut armed, &[], false, true).is_empty());
    assert_eq!(announce(&mut armed, &waiting, false, true), vec![arrival]);
    assert!(announce(&mut armed, &waiting, false, true).is_empty());

    // Focused: silent, and the ask is absorbed — burying the window later says
    // nothing, because the operator was looking straight at it.
    let mut looking = Announced::default();
    assert!(announce(&mut looking, &[], true, true).is_empty());
    assert!(announce(&mut looking, &waiting, true, true).is_empty());
    assert!(announce(&mut looking, &waiting, false, true).is_empty());

    // Knob off: the same, and switching it on does not replay the backlog.
    let mut off = Announced::default();
    assert!(announce(&mut off, &[], false, false).is_empty());
    assert!(announce(&mut off, &waiting, false, false).is_empty());
    assert!(announce(&mut off, &waiting, false, true).is_empty());
}
