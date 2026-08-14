//! Unit tests for the inbox-composer derivations (§11 inbox-composer,
//! bl-929d): the queue projection, the derived fold-line height, and the
//! structurally-triggered snap.

use super::{ComposerRam, SNAP_SECS, SnapState, rows};
use crate::actions::DraftKey;
use crate::inboxview::{Deposit, InboxEntry};
use crate::nav::convs::Titles;
use std::collections::HashSet;

fn near(is: f32, want: f32) -> bool {
    (is - want).abs() < 0.5
}

fn entry(name: &str, from: &str, at: &str, body: &str) -> InboxEntry {
    InboxEntry {
        name: name.into(),
        raw: body.as_bytes().to_vec(),
        deposit: Deposit {
            sender: Some(from.into()),
            deposited_at: Some(at.into()),
            body: body.into(),
            ..Deposit::default()
        },
    }
}

#[test]
fn rows_project_the_listing_in_order_keyed_by_inbox_path() {
    let pending = vec![
        entry("user-001.md", "user", "t0", "first line\nrest"),
        entry("p1-002.md", "p1", "t1", "steer"),
    ];
    let out = rows("c-1", &pending, &Titles::default(), &HashSet::new());
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].key, "inbox/c-1/user-001.md");
    assert_eq!(out[0].header, "✉ user · t0");
    assert_eq!(out[0].preview, "first line");
    assert_eq!(out[0].body, "first line\nrest");
    assert!(!out[0].expanded, "folded is the auto-state");
    assert_eq!(out[1].key, "inbox/c-1/p1-002.md");
    assert_eq!(out[1].header, "✉ p1 · t1");
}

#[test]
fn a_fold_override_flips_exactly_its_row_open() {
    let pending = vec![
        entry("user-001.md", "user", "t0", "a"),
        entry("user-002.md", "user", "t1", "b"),
    ];
    let folds: HashSet<String> = ["inbox/c-1/user-002.md".to_string()].into();
    let out = rows("c-1", &pending, &Titles::default(), &folds);
    assert!(!out[0].expanded);
    assert!(out[1].expanded);
}

#[test]
fn an_empty_body_previews_empty_rather_than_panicking() {
    let out = rows(
        "c-1",
        &[entry("u-001.md", "u", "t", "")],
        &Titles::default(),
        &HashSet::new(),
    );
    assert_eq!(out[0].preview, "");
}

/// The §11 role stripe (bl-3acb): a pending row wears the same byte-derived
/// role identity its delivered transcript row will — one mapping
/// ([`crate::theme::message_role`]), two seats.
#[test]
fn a_pending_row_wears_the_role_its_deposit_asserts() {
    use crate::theme::Role;
    let mut ended = entry("kid-003.md", "kid", "t2", "");
    ended.deposit.epitaph = Some(crate::inboxview::Epitaph::Stopped);
    let mut nameless = entry("x-004.md", "x", "t3", "hi");
    nameless.deposit.sender = None;
    let pending = vec![
        entry("user-001.md", "user", "t0", "mine"),
        entry("p1-002.md", "p1", "t1", "theirs"),
        ended,
        nameless,
    ];
    let out = rows("c-1", &pending, &Titles::default(), &HashSet::new());
    let roles: Vec<Role> = out.iter().map(|r| r.role).collect();
    assert_eq!(
        roles,
        vec![Role::User, Role::Peer, Role::Ended, Role::Peer],
        "the deposit's asserted sender and epitaph decide, nothing else"
    );
}

fn key() -> DraftKey {
    DraftKey::Message("c-1".into())
}

#[test]
fn the_fold_line_is_the_settled_content_height_capped_at_half_the_pane() {
    let mut snap = SnapState::default();
    snap.observe(&key(), 2, 0.0);
    // Unmeasured: the region opens at zero and the one-frame settle fills it.
    assert!(near(snap.desired(400.0, 0.0), 0.0));
    snap.settle(120.0);
    assert!(near(snap.settled(), 120.0));
    assert!(near(snap.desired(400.0, 0.1), 120.0));
    // Past the cap the line stops and the queue scrolls instead — the render
    // reads the uncapped settled height to know the region is overfull.
    snap.settle(900.0);
    assert!(near(snap.settled(), 900.0));
    assert!(near(snap.desired(400.0, 0.2), 400.0));
}

#[test]
fn an_item_landing_raises_the_line_without_any_snap() {
    let mut snap = SnapState::default();
    snap.observe(&key(), 1, 0.0);
    snap.settle(60.0);
    snap.observe(&key(), 2, 1.0);
    assert!(
        !snap.active(1.0),
        "a rising count is an arrival, not a drain"
    );
    snap.settle(80.0);
    assert!(near(snap.desired(400.0, 1.1), 80.0));
}

#[test]
fn a_pending_drop_snaps_down_from_the_pre_drain_height_then_settles() {
    let mut snap = SnapState::default();
    snap.observe(&key(), 3, 0.0);
    snap.settle(200.0);
    // The drain: delivery commits landed, the count dropped.
    snap.observe(&key(), 0, 10.0);
    assert!(snap.active(10.0));
    // Content re-measures at the bare input row; the ease starts at the
    // pre-drain height and descends toward it.
    snap.settle(40.0);
    assert!(near(snap.desired(400.0, 10.0), 200.0));
    let mid = snap.desired(400.0, 10.0 + SNAP_SECS / 2.0);
    assert!(mid > 40.0 && mid < 200.0, "mid-ease sits between: {mid}");
    let late = snap.desired(400.0, 10.0 + SNAP_SECS * 0.9);
    assert!(late < mid, "the ease descends: {late} !< {mid}");
    assert!(near(snap.desired(400.0, 10.0 + SNAP_SECS), 40.0));
    assert!(!snap.active(10.0 + SNAP_SECS));
    // The next steady frame retires the finished ease.
    snap.observe(&key(), 0, 11.0);
    assert!(near(snap.desired(400.0, 11.0), 40.0));
}

#[test]
fn a_target_switch_resets_the_state_and_never_reads_as_a_drain() {
    let mut snap = SnapState::default();
    snap.observe(&key(), 5, 0.0);
    snap.settle(300.0);
    // A different conversation with fewer pending items: a different queue,
    // not a drain — no snap, and the measurement starts over.
    snap.observe(&DraftKey::Message("c-2".into()), 1, 0.1);
    assert!(!snap.active(0.1));
    assert!(near(snap.desired(400.0, 0.1), 0.0));
}

#[test]
fn composer_ram_defaults_empty() {
    let ram = ComposerRam::default();
    assert!(ram.folds.is_empty());
    assert!(!ram.snap.active(0.0));
}

/// The §11 tone (bl-915e): a deposit that is a file paints as a statement, and
/// §7.2's pending echo — the send yog has made and the driver has not written,
/// which is the one deposit with no file — paints faded. The whole predicate is
/// the absence of a name, so nothing real can wear the faded tone by accident.
#[test]
fn a_deposit_with_no_file_is_the_faded_one() {
    let landed = entry("user-001.md", "user", "t0", "already mail");
    let echo = InboxEntry {
        name: String::new(),
        ..entry("", "user", "t1", "just said")
    };
    assert!(!landed.in_memory(), "a listed deposit is its file");
    assert!(echo.in_memory(), "the echo has none");
    let out = rows("c-1", &[landed, echo], &Titles::default(), &HashSet::new());
    assert_eq!(out[0].tone, crate::transcript::Tone::Plain);
    assert_eq!(
        out[1].tone,
        crate::transcript::Tone::Weak,
        "faded until the derivation makes it a statement"
    );
}
