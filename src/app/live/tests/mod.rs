//! The live tail's own contract (§7.2, bl-54f7): **freshness** and the dead
//! end. The follower's reading discipline — partial lines, the resets, the
//! thread — is [`reading`], split off at §12's per-file budget on the seam
//! between *what the tail promises the operator* and *how the bytes are
//! gathered*. The fixtures below serve both.
//!
//! The assertion that matters is freshness, not content. A beat that asserts
//! the tail is merely non-empty passed with the defect shipped — the tail was
//! never empty, it was *late* (bl-70b8's catalogue of tests that prove nothing;
//! bl-f16e on vacuous assertions). So the beats here append bytes to an open
//! step file and assert they are on the composed frame **with no derivation
//! having run**: `Rig::tick` is never called, no root is ever marked dirty, and
//! the frame's `derived` pointer is checked to be the very one it was before.

mod reading;

use super::Follower;
use crate::app::tests::{Harness, Rig};
use crate::git_tree::{Delta, Stream};
use crate::watch::Repaint;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(super) const AGENT: &str = "c-1";

/// One `content_delta` line of answer text, as brazen's `v=1` writes it.
pub(super) fn text_delta(fragment: &str) -> String {
    format!(
        "{{\"type\":\"content_delta\",\"index\":0,\"delta\":{{\"text_delta\":\"{fragment}\"}}}}\n"
    )
}

/// One `content_delta` line of reasoning.
pub(super) fn thinking_delta(fragment: &str) -> String {
    format!(
        "{{\"type\":\"content_delta\",\"index\":0,\"delta\":{{\"thinking_delta\":\"{fragment}\"}}}}\n"
    )
}

/// The open response file of `agent`'s step `seq` — the literal file lernie's
/// harness appends stream events to.
pub(super) fn response(h: &Harness, agent: &str, seq: u32) -> std::path::PathBuf {
    let step =
        h.fx.path
            .join("steps")
            .join(agent)
            .join(format!("{seq:03}"));
    std::fs::create_dir_all(&step).expect("step dir");
    step.join("response.json")
}

/// Append to an open response file, exactly as the harness does mid-call.
pub(super) fn append(path: &Path, bytes: &str) {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open response");
    file.write_all(bytes.as_bytes()).expect("append");
}

/// The stream the frame would paint for `agent` — off the **composed**
/// snapshot, which is what every render seat reads.
pub(super) fn painted(rig: &Rig, ws: &Path, agent: &str) -> Stream {
    rig.tree(ws)
        .and_then(|t| t.agents.iter().find(|a| a.agent_id == agent).cloned())
        .map(|a| a.stream)
        .unwrap_or_default()
}

/// The same stream off the **derivation** — what every gesture and every
/// machine-facing reply reads (`boundary_deps`).
fn derived(rig: &Rig, ws: &Path, agent: &str) -> Stream {
    rig.model
        .derivation()
        .trees
        .get(ws)
        .and_then(|t| t.agents.iter().find(|a| a.agent_id == agent).cloned())
        .map(|a| a.stream)
        .unwrap_or_default()
}

/// A model focused on `AGENT` plus its follower, both driven by hand. The
/// `refresh` is what publishes the ask — a frame is how the follower learns
/// what to follow.
pub(super) fn rigged(h: &Harness) -> (Rig, Follower) {
    let (_clock, mut rig) = h.model();
    let follower = rig.follower();
    rig.focus_agent(&h.ws, AGENT);
    rig.refresh();
    (rig, follower)
}

/// A [`Repaint`] double counting requests — the face the follower would wake.
pub(super) struct CountingRepaint(pub(super) Arc<AtomicUsize>);

impl Repaint for CountingRepaint {
    fn request(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn bytes_appended_to_the_open_step_file_are_on_the_next_frame_with_no_derivation() {
    let h = Harness::new();
    let (mut rig, mut follower) = rigged(&h);
    let file = response(&h, AGENT, 1);

    // The model says its first words.
    append(&file, &text_delta("the first "));
    assert!(follower.pass(), "the first bytes are news");
    rig.refresh();
    assert_eq!(
        painted(&rig, &h.ws, AGENT).text.as_deref(),
        Some("the first "),
        "the follower's fold is what the frame paints"
    );

    // The derivation the frame is holding, pinned. Nothing below may move it:
    // no `tick`, no dirty mark, no clock advance.
    let pinned = Arc::clone(rig.model.derivation());

    // Another fragment lands. One follower pass, one repaint.
    append(&file, &text_delta("half."));
    assert!(follower.pass(), "the append is seen");
    rig.refresh();

    assert_eq!(
        painted(&rig, &h.ws, AGENT).text.as_deref(),
        Some("the first half."),
        "every character that landed is on the frame"
    );
    assert!(
        Arc::ptr_eq(&pinned, rig.model.derivation()),
        "and no derivation ran to put it there — this is the whole claim"
    );
}

#[test]
fn the_derivation_the_gestures_read_is_untouched_by_the_tail() {
    let h = Harness::new();
    let (mut rig, mut follower) = rigged(&h);
    append(&response(&h, AGENT, 1), &text_delta("only in RAM"));
    follower.pass();
    rig.refresh();

    // The dead end (the in-memory carve-out): paint sees the tail,
    // the derivation does not. Every gesture, every §8.5 dispatch and every
    // machine-facing reply takes the derivation, so none of them can be decided
    // by a fact only the painter holds.
    assert_eq!(
        derived(&rig, &h.ws, AGENT),
        Stream::default(),
        "the derivation has not read the file, and the tail did not tell it"
    );
    assert_eq!(
        painted(&rig, &h.ws, AGENT).text.as_deref(),
        Some("only in RAM"),
        "while the painted snapshot carries it"
    );
}

#[test]
fn reasoning_streams_too_and_carries_the_doing_split_with_it() {
    let h = Harness::new();
    let (mut rig, mut follower) = rigged(&h);
    let file = response(&h, AGENT, 1);

    // Thinking, with no answer text at all — the phase the operator described
    // as "nothing is happening on screen".
    append(&file, &thinking_delta("first I "));
    follower.pass();
    rig.refresh();
    let stream = painted(&rig, &h.ws, AGENT);
    assert_eq!(stream.thinking.as_deref(), Some("first I "));
    assert_eq!(stream.text, None, "no answer yet, and none invented");
    assert_eq!(stream.last_delta, Some(Delta::Thinking));

    // It keeps thinking and the row grows, which is the thing a badge cannot do.
    append(&file, &thinking_delta("check the refs"));
    assert!(follower.pass());
    rig.refresh();
    assert_eq!(
        painted(&rig, &h.ws, AGENT).thinking.as_deref(),
        Some("first I check the refs")
    );

    // Then it answers. The same one fold moves `last_delta`, so the §11 live
    // mark flips from Thinking to Inference off the tail, not off a sweep.
    append(&file, &text_delta("here goes"));
    assert!(follower.pass());
    rig.refresh();
    let stream = painted(&rig, &h.ws, AGENT);
    assert_eq!(stream.text.as_deref(), Some("here goes"));
    assert_eq!(stream.last_delta, Some(Delta::Text));
}
