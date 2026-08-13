//! End-to-end drive of the §8.5 `gesture` multiplex arm through
//! `yog::multiplex::dispatch` — `yog gesture '<json>'` composes the world,
//! validates the envelope, deposits into the world's gestures inbox and waits
//! for a reply.
//!
//! **This test binary owns its process environment** (the
//! `tests/multiplex_bl.rs` precedent): the arm composes the world from
//! `$HOME`/`$XDG_*` live at the process boundary, so this file mutates its own
//! env — lawful exactly here, one `#[test]`, its own process.

#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::time::Duration;
use yog::multiplex::dispatch;

fn set(key: &str, value: &Path) {
    // SAFETY: single-threaded env mutation before any reader thread exists
    // (module doc; the answering thread below only reads the filesystem).
    unsafe { std::env::set_var(key, value) };
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn yog_gesture_deposits_waits_and_exits_on_the_reply() {
    let root = tempfile::tempdir().unwrap();
    set("HOME", &root.path().join("home"));
    set("XDG_DATA_HOME", &root.path().join("data"));
    set("XDG_STATE_HOME", &root.path().join("state"));
    set("XDG_CONFIG_HOME", &root.path().join("config"));
    let world = yog::world::compose(&yog::xdg::Env::from_env());
    let state_root = world.yog_state_root();

    // A refused envelope never composes a deposit: exit 2, empty inbox.
    assert_eq!(dispatch(&argv(&["yog", "gesture", "not json"])), Some(2));
    assert_eq!(
        dispatch(&argv(&["yog", "gesture", r#"{"op":"warp"}"#])),
        Some(2)
    );
    assert!(yog::boundary::deposit::pending(&state_root).is_empty());

    // The consumer half, played by a thread: answer whatever lands.
    let answering_root = state_root.clone();
    let answerer = std::thread::spawn(move || {
        for _ in 0..2000 {
            if let Some((id, _)) = yog::boundary::deposit::pending(&answering_root).first() {
                yog::boundary::deposit::claim(&answering_root, id).unwrap();
                yog::boundary::deposit::write_reply(
                    &answering_root,
                    id,
                    &serde_json::json!({"ok": true, "kind": "balls", "rows": []}),
                )
                .unwrap();
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("no deposit ever arrived");
    });
    assert_eq!(
        dispatch(&argv(&["yog", "gesture", r#"{"op":"balls"}"#])),
        Some(0),
        "the reply's verdict is the exit"
    );
    answerer.join().unwrap();
}
