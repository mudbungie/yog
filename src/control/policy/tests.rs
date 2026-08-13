//! The standing policy: absence is the shipped defaults, and every key is an
//! override of exactly one thing.

use super::*;
use tempfile::tempdir;

#[test]
fn nothing_declared_is_the_shipped_state() {
    let shipped = Policy::default();
    assert!(!shipped.confinement_required);
    // The table answers exactly what the shipped one does, class for class.
    for (effect, want) in [
        (Effect::Read, Ruling::Pass),
        (Effect::TargetWrite, Ruling::Pass),
        (Effect::Process, Ruling::Pass),
        (Effect::OpenWorld, Ruling::Pass),
        (Effect::Destructive, Ruling::Refuse),
        (Effect::Secret, Ruling::Refuse),
    ] {
        assert_eq!(shipped.ruling(effect), want, "{}", effect.word());
    }
    assert_eq!(shipped.rows().len(), DEFAULT.len());
    assert_eq!(shipped.secret_fragments().len(), SECRET_FRAGMENTS.len());
    // An empty file and no file at all are one state.
    assert_eq!(Policy::parse(""), shipped);
    assert_eq!(Policy::parse("# nothing but a comment\n"), shipped);
}

#[test]
fn a_table_row_overrides_one_class_and_leaves_the_rest() {
    let policy = Policy::parse("table:\n  open-world: refuse\n");
    assert_eq!(policy.ruling(Effect::OpenWorld), Ruling::Refuse);
    assert_eq!(policy.ruling(Effect::Read), Ruling::Pass);
    assert_eq!(policy.ruling(Effect::Destructive), Ruling::Refuse);
    // The parked default the shipped table used to carry is exactly one row —
    // an operator who wants to answer for every fetch writes it and nothing
    // else moves.
    let parked = Policy::parse("table:\n  open-world: hold\n");
    assert_eq!(parked.ruling(Effect::OpenWorld), Ruling::Hold);
    assert_eq!(parked.ruling(Effect::Process), Ruling::Pass);
}

#[test]
fn the_last_statement_of_a_class_wins() {
    let policy = Policy::parse("table:\n  open-world: refuse\n  open-world: pass\n");
    assert_eq!(policy.ruling(Effect::OpenWorld), Ruling::Pass);
}

#[test]
fn an_operator_rule_leads_the_shipped_ones() {
    let policy = Policy::parse("rules:\n  python: target-write\n  git push: read\n");
    let rows = policy.rows();
    assert_eq!(rows.first().map(|r| r.program.as_str()), Some("python"));
    assert_eq!(rows.first().map(|r| r.reach), Some(Reach::Fixed(TARGET)));
    // Qualifying words ride the key, so a row can be narrower than a program.
    let git = rows.get(1).expect("second operator row");
    assert_eq!(git.program, "git");
    assert_eq!(git.words, vec!["push".to_owned()]);
    // The shipped rows are still there, after them.
    assert_eq!(rows.len(), DEFAULT.len() + 2);
    assert!(rows.iter().any(|r| r.program == "curl"));
}

/// The class name in the policy file, checked against the one the sentences
/// speak — `target write` said as one token.
const TARGET: Effect = Effect::TargetWrite;

#[test]
fn secrets_are_additive_only() {
    let policy = Policy::parse("secrets:\n  - .kube\n  - vault-token\n");
    let fragments = policy.secret_fragments();
    assert!(fragments.iter().any(|f| f == ".kube"));
    assert!(fragments.iter().any(|f| f == "vault-token"));
    // Every shipped fragment survives — a workspace may widen, never narrow.
    for shipped in SECRET_FRAGMENTS {
        assert!(fragments.iter().any(|f| f == shipped), "{shipped}");
    }
}

#[test]
fn confinement_is_required_only_when_it_says_so() {
    assert!(Policy::parse("confinement: required\n").confinement_required);
    assert!(!Policy::parse("confinement: optional\n").confinement_required);
    assert!(!Policy::parse("confinement:\n").confinement_required);
}

#[test]
fn unreadable_lines_contribute_nothing_and_never_panic() {
    let policy = Policy::parse(concat!(
        "gibberish\n",
        "table:\n",
        "  nonesuch: refuse\n",
        "  read: nonesuch\n",
        "rules:\n",
        "  : read\n",
        "  curl: nonesuch\n",
        "unknown-block:\n",
        "  anything: at all\n",
    ));
    assert_eq!(policy, Policy::default());
}

#[test]
fn a_comment_and_a_blank_line_do_not_end_a_block() {
    let policy = Policy::parse("rules:\n  # why we do this\n\n  python: read\n");
    assert_eq!(
        policy.rows().first().map(|r| r.program.as_str()),
        Some("python")
    );
}

#[test]
fn a_workspace_with_no_repo_reads_the_shipped_state() {
    let dir = tempdir().unwrap();
    assert_eq!(Policy::read(dir.path()), Policy::default());
}
