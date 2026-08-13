//! STORIES **S0 step 4** ("any step failure is a **rendered fact**") for the one
//! class that satisfied neither it nor §7.3's "a failed action is never
//! stderr-only": a conversation whose driver died before the model said
//! anything (bl-8e07's real-substrate finding, a substrate version skew).
//!
//! On disk that is `request.json` written, `response.json` at zero bytes, no
//! `meta.json` — which the §4.4 framing reads as `Killed`, painting the same
//! quiet ash badge over a `0 attempts · 0 tok` row a mid-stream kill gets. The
//! §7.3 no-response wound is the distinct state; this drives the same public
//! view-model the shell's Steps tab and Altitude-1 banner paint.
//!
//! Pure derivation over an on-disk fixture — no subprocess, no fake substrate
//! (the stories_s0_t6 precedent).

#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tempfile::tempdir;
use yog::app::{Cadence, WoundGrace};
use yog::git_tree::{AgentState, Framing};
use yog::steps_view::{self, NO_RESPONSE, Wound, latest_wound};
use yog::ui_state::Clock;

const AGENT: &str = "20260725T050000Z-skew";

/// What the bl-55d8 falsifying run left in `steps/<agent>/002/stderr.log`,
/// verbatim — a `bz` that refused before it could speak in band.
const BZ_REFUSAL: &str = "bz: no workspace in this environment — providers, sign-ins and the \
model cache belong to a workspace, and there is nothing shared to fall back to. Run this inside \
a yog workspace, or focus one in yog.";

/// The default cadence's grace window (bl-3381) — what the shell passes when
/// the operator has not re-tuned.
fn window() -> Duration {
    Cadence::default().wound_grace()
}

/// A clock the test moves by hand (the crate's `FakeClock` is `pub(crate)`), so
/// the grace window is exercised without sleeping.
struct TestClock {
    base: Instant,
    millis: AtomicU64,
}

impl TestClock {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            base: Instant::now(),
            millis: AtomicU64::new(0),
        })
    }
    fn advance(&self, by: Duration) {
        self.millis
            .fetch_add(by.as_millis() as u64, Ordering::SeqCst);
    }
}

impl Clock for TestClock {
    fn now(&self) -> Instant {
        self.base + Duration::from_millis(self.millis.load(Ordering::SeqCst))
    }
    fn stamp(&self) -> String {
        "TS".to_string()
    }
}

/// Lay down one step exactly as lernie leaves it (ARCH §2.3 write order):
/// `request.json` first, then `response.json`, and `meta.json` only once the
/// model call returns.
fn write_step(ws: &Path, seq: u32, response: Option<&[u8]>, meta: Option<&[u8]>) {
    write_step_with_stderr(ws, seq, response, meta, b"");
}

/// The same, plus the adapter's captured stderr — `stderr.log`, which lernie
/// opens on every attempt and which is empty on an ordinary run (ARCH §2.3).
fn write_step_with_stderr(
    ws: &Path,
    seq: u32,
    response: Option<&[u8]>,
    meta: Option<&[u8]>,
    stderr: &[u8],
) {
    let dir = ws.join("steps").join(AGENT).join(format!("{seq:03}"));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("request.json"), br#"{"model":"opus"}"#).unwrap();
    std::fs::write(dir.join("stderr.log"), stderr).unwrap();
    if let Some(bytes) = response {
        std::fs::write(dir.join("response.json"), bytes).unwrap();
    }
    if let Some(bytes) = meta {
        std::fs::write(dir.join("meta.json"), bytes).unwrap();
    }
}

#[test]
fn a_driver_that_died_without_a_response_renders_a_cause() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    // Step 001 answered and settled; step 002's driver died producing nothing.
    write_step(
        ws,
        1,
        Some(b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n"),
        Some(br#"{"commit":"abc","started_at":"t0","ended_at":"t1"}"#),
    );
    write_step(ws, 2, Some(b""), None);

    let view = steps_view::build(ws, AGENT, AgentState::Stopped);
    assert_eq!(view.steps.len(), 2);
    assert!(
        !view.steps[0].wound.wounded(),
        "an answered step is not a wound"
    );

    let dead = &view.steps[1];
    assert!(dead.wound.wounded(), "{NO_RESPONSE} — the rendered fact");
    // The counts that made it read as a quiet step are unchanged; the wound is
    // the state beside them, not a different figure.
    assert_eq!(dead.attempts, 0);
    assert_eq!(dead.tokens.total_tokens(), 0);
    assert_eq!(dead.framing, Framing::Killed);

    // The §11 Altitude-1 banner reads the same derivation, so the cause is on
    // the conversation surface whichever inspector tab is open.
    assert!(latest_wound(&steps_view::build(ws, AGENT, AgentState::Stopped)).wounded());
    // …and never while a driver is still filling that step (§10).
    assert!(!latest_wound(&steps_view::build(ws, AGENT, AgentState::InFlight)).wounded());
}

/// bl-55d8, from the falsifying run's own bytes: `002/response.json` at zero
/// bytes, no `meta.json`, and the reason sitting in `002/stderr.log` — the
/// operator's whole signal was that nothing ever came back.
#[test]
fn the_wound_states_the_reason_the_step_recorded() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_step(
        ws,
        1,
        Some(b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n"),
        Some(br#"{"commit":"abc","started_at":"t0","ended_at":"t1"}"#),
    );
    write_step_with_stderr(ws, 2, Some(b""), None, BZ_REFUSAL.as_bytes());

    let wound = latest_wound(&steps_view::build(ws, AGENT, AgentState::Stopped));
    assert_eq!(wound, Wound::Spoke(BZ_REFUSAL.to_owned()));
    let sentence = wound.banner();
    assert!(sentence.contains(NO_RESPONSE), "the class: {sentence}");
    assert!(
        sentence.contains("no workspace in this environment"),
        "the reason itself, in words (§7.3 / §11 glyph doctrine): {sentence}"
    );

    // The step that answered keeps its own silence: reading a reason is gated
    // on the wound, so a healthy conversation is unchanged and pays nothing.
    let view = steps_view::build(ws, AGENT, AgentState::Stopped);
    assert_eq!(view.steps[0].wound, Wound::None);
}

/// bl-90bf: the banner is an alarm, and the predicate's liveness half is the
/// snapshot's — up to a §7.2 poll behind the disk half the frame re-reads. So a
/// freshly-sent step whose driver already holds the lock reads wounded until
/// the cache catches up. The grace gate is what keeps that off the screen,
/// without ever silencing a driver that really did die.
#[test]
fn the_banner_waits_out_the_grace_window() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    // A live send, one instant old: request written, response empty, no meta —
    // byte-for-byte the wound's shape, and the cached state has not yet seen
    // the flock the driver took (no fs event announces one).
    write_step(ws, 1, Some(b""), None);
    let wounded = latest_wound(&steps_view::build(ws, AGENT, AgentState::Stopped)).wounded();
    assert!(wounded, "the raw predicate reads the wound");

    let clock = TestClock::new();
    // The double is a whole `Clock`: `stamp` is the trait's §4.2 ops-timestamp
    // half, which the grace gate never reads and this one answers constant.
    assert_eq!(clock.stamp(), "TS");
    let mut grace = WoundGrace::new(clock.clone());
    assert!(
        !grace.paints(ws, AGENT, wounded, window()),
        "never on first sight"
    );
    clock.advance(window() / 2);
    assert!(
        !grace.paints(ws, AGENT, wounded, window()),
        "still inside the window"
    );
    // The snapshot catches up: the driver was alive the whole time. The alarm
    // that would have flashed red on a healthy send never painted.
    let healed = latest_wound(&steps_view::build(ws, AGENT, AgentState::InFlight)).wounded();
    assert!(!grace.paints(ws, AGENT, healed, window()));

    // Now the honest wound: the driver really is gone, and it stays wounded
    // across the window — so it banners, delayed but never dropped.
    let clock = TestClock::new();
    let mut grace = WoundGrace::new(clock.clone());
    assert!(!grace.paints(ws, AGENT, wounded, window()));
    clock.advance(window());
    assert!(
        grace.paints(ws, AGENT, wounded, window()),
        "{NO_RESPONSE} — banner-ed"
    );
}
