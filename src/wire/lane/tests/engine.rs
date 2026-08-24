//! **The two halves together** (bl-73e7): a held read that delays nothing, and
//! one connection carrying every growth of a real conversation's tail.
//!
//! Split from [`super`] at §12's per-file budget on the seam its own doc draws
//! — the beats there stand the engine in, and these two cannot, because what
//! they claim is about the engine and the lane at once.

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use tempfile::TempDir;

use super::super::*;
use super::{awaited, frame, subject, watching};
use crate::registry::presence::Presence;
use crate::test_support::wire::{EPHEMERAL, material, mint};
use crate::watch::NoRepaint;
use crate::wire::material::Role;
use crate::wire::server::{Answerer, Listener};

/// An engine that holds a follow read until released, and answers anything else
/// at once. The terminator is gated too, so a beat can observe a frame *before*
/// the stream ends rather than racing it.
struct Holds {
    release: Arc<AtomicUsize>,
    served: Arc<AtomicUsize>,
}

impl Answerer for Holds {
    fn answer(
        &self,
        _client: &crate::registry::Client,
        request: Value,
    ) -> Box<dyn Iterator<Item = Value>> {
        if request["op"] == "follow" {
            let release = Arc::clone(&self.release);
            return Box::new((0..=1).filter_map(move |i| {
                while release.load(Ordering::Relaxed) <= i {
                    std::thread::yield_now();
                }
                (i == 0).then(|| frame("held, then said"))
            }));
        }
        self.served.fetch_add(1, Ordering::Relaxed);
        Box::new(std::iter::once(
            json!({"ok": true, "kind": "balls", "rows": []}),
        ))
    }
}

/// **A held read never delays the standing set** (the lane's whole reason). An
/// engine that answers a follow read by holding — and only then writing —
/// answers every other query of the same connection-per-ask kind meanwhile, so
/// the asker's serial pass runs to completion while the lane is still parked.
#[test]
fn a_held_read_does_not_delay_the_standing_set() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());

    let release = Arc::new(AtomicUsize::new(0));
    let served = Arc::new(AtomicUsize::new(0));
    let listener = Listener::bind(
        &material(tmp.path(), Role::Server, EPHEMERAL),
        Arc::new(Holds {
            release: Arc::clone(&release),
            served: Arc::clone(&served),
        }),
        Presence::default(),
    )
    .expect("bind");
    let at = crate::wire::loopback(&listener.address());
    let lane_seat =
        crate::wire::client::Seat::open(&material(tmp.path(), Role::Window, &at)).expect("seat");
    let asker_seat =
        crate::wire::client::Seat::open(&material(tmp.path(), Role::Window, &at)).expect("seat");

    let (mut tail, end) = pair();
    watching(&mut tail);
    let mut lane = Lane::new(
        crate::wire::dial::Dial::of(lane_seat),
        end,
        Arc::new(NoRepaint),
    );
    let held = std::thread::spawn(move || lane.turn());

    // The lane is parked on a read that has not answered. The standing set's
    // own asks cross and are answered anyway — which is the whole claim.
    for _ in 0..3 {
        assert!(
            asker_seat.answered(&json!({"op": "balls"})).is_ok(),
            "the serial pass is not behind the held read"
        );
    }
    assert_eq!(served.load(Ordering::Relaxed), 3);

    release.store(1, Ordering::Relaxed);
    assert!(
        awaited(&mut tail, "held, then said"),
        "and the held frame lands when it is written"
    );
    release.store(2, Ordering::Relaxed);
    assert!(held.join().expect("the lane's turn ends"));
}

/// **The two halves together**: the real intake, a real workspace, and one
/// connection held across several growths of the tail — then terminated when
/// the call settles. Every other query's semantics are untouched, which the
/// same connection proves by answering one before the follow ever starts.
#[test]
fn the_engine_holds_one_connection_across_every_growth_of_the_tail() {
    use crate::boundary::tests::{agent, snapshot};
    use crate::git_tree::AgentState;

    let tmp = TempDir::new().expect("tmp");
    // Where the §3.1 enumeration actually looks: the intake re-asks the
    // workspace set of disk per request (bl-6c9e), so a hand-built snapshot
    // alone would name a workspace no address could resolve.
    let data = tmp.path().join("data");
    let ws = crate::binding::workspace_path(&data, "alba");
    std::fs::create_dir_all(ws.join("repo.git")).expect("the workspace marker");
    let step = ws.join("steps").join("c-1").join("001");
    std::fs::create_dir_all(&step).expect("step dir");
    let file = step.join("response.json");

    let cell = crate::state::new_snapshot_cell(Arc::new(snapshot(
        &ws,
        "alba",
        vec![agent("c-1", AgentState::InFlight, 100)],
        vec![],
    )));
    let state_root = tmp.path().join("state");
    std::fs::create_dir_all(&state_root).expect("state root");
    // Authorization is registration (REMOTE §4): unregistered, the window sees
    // no workspace at all and the address resolves to nothing.
    crate::registry::register(&state_root, &crate::registry::window(), "alba")
        .expect("seat the window");
    let intake =
        crate::wire::intake::Intake::new(Arc::new(crate::boundary::consumer::ConsumerCtx {
            yog_binary: std::path::PathBuf::from("/no/such/yog"),
            world: crate::test_support::no_world(),
            lernie: crate::cli_outbound::Cli::new("/no/such/lernie"),
            bl: crate::cli_outbound::Cli::new("/no/such/bl"),
            state_root: state_root.clone(),
            home: std::path::PathBuf::from("/home/x"),
            yog_data_root: data,
            balls_state_root: tmp.path().join("balls"),
            ui_path: state_root.join("ui.json"),
            cell: Arc::clone(&cell),
            presence: Presence::default(),
            mailbox: crate::registry::mailbox::Mailbox::default(),
            clock: Arc::new(crate::ui_state::SystemClock),
        }));

    mint(tmp.path());
    let listener = Listener::bind(
        &material(tmp.path(), Role::Server, EPHEMERAL),
        Arc::new(intake) as Arc<dyn Answerer>,
        Presence::default(),
    )
    .expect("bind");
    let seat = crate::wire::client::Seat::open(&material(
        tmp.path(),
        Role::Window,
        &crate::wire::loopback(&listener.address()),
    ))
    .expect("seat");

    // An ordinary connection-per-ask read over the same listener, unchanged:
    // one frame, then a terminator.
    assert!(
        seat.ask(&json!({"op": "balls"})).expect("answered").len() == 1,
        "every other query is still one frame and a terminator"
    );

    // The tail grows three times while ONE connection stays open, and settles.
    let writer = std::thread::spawn(move || {
        for word in ["one ", "two ", "three"] {
            let mut at = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file)
                .expect("append");
            writeln!(
                at,
                "{{\"type\":\"content_delta\",\"index\":0,\
                  \"delta\":{{\"text_delta\":\"{word}\"}}}}"
            )
            .expect("write");
            drop(at);
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        crate::state::publish_snapshot(
            &cell,
            Arc::new(snapshot(
                &ws,
                "alba",
                vec![agent("c-1", AgentState::Quiescent, 100)],
                vec![],
            )),
        );
    });

    let mut said: Vec<String> = Vec::new();
    seat.followed(&subject(), &mut |landed| {
        if let Ok(crate::boundary::reply::Reply::Follow(stream)) = landed {
            said.push(stream.text.unwrap_or_default());
        }
        true
    })
    .expect("the held read ends cleanly");
    writer.join().expect("the writer finishes");

    assert_eq!(
        said.last().map(String::as_str),
        Some("one two three"),
        "the last frame carries the whole tail: {said:?}"
    );
    assert!(
        said.len() > 1,
        "and the connection was held across the growths rather than answering once: {said:?}"
    );
}
