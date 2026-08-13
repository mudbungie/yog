//! The composer's line seat (§8.5), driven through the real keyboard on the
//! real window: a draft that starts with `/` is a command, Enter runs it, and
//! what the boundary answered lands under the box.
//!
//! Asserted at the paint layer because that is where the seat's whole claim
//! lives — the reading and the dispatch are covered where they are written; what
//! a widget test could not show is that the one box changes what Enter means,
//! and says so.

use super::fixture::world;
use super::screen::{Screen, press};

fn typed(text: &str) -> Vec<egui::Event> {
    vec![egui::Event::Text(text.to_owned())]
}

fn enter() -> Vec<egui::Event> {
    vec![press(egui::Key::Enter, egui::Modifiers::NONE)]
}

/// A query typed at the window is answered at the window — the same reply the
/// deposit's file would carry, rendered under the box that asked.
#[test]
fn a_slash_query_is_answered_under_the_box() {
    let mut world = world();
    let screen = Screen::new();
    assert!(screen.idle(&mut world), "the cursor starts in the composer");
    screen.frame(&mut world, typed("/balls"));
    let drafted = screen.text(&mut world);
    assert!(
        drafted.contains("Run"),
        "a command re-labels the one button:\n{drafted}"
    );
    screen.frame(&mut world, enter());
    let answered = screen.text(&mut world);
    assert!(
        answered.contains("\"ok\""),
        "the reply's own JSON is the answer:\n{answered}"
    );
    assert!(
        !answered.contains("/balls"),
        "a landed line clears its draft:\n{answered}"
    );
}

/// A line that is not a gesture refuses **and keeps the draft**: the operator
/// fixes what they typed rather than retyping it, and the reason names the verb.
#[test]
fn a_refused_line_says_why_and_keeps_the_draft() {
    let mut world = world();
    let screen = Screen::new();
    assert!(screen.idle(&mut world));
    screen.frame(&mut world, typed("/enhance"));
    screen.frame(&mut world, enter());
    let refused = screen.text(&mut world);
    assert!(
        refused.contains("unknown command /enhance"),
        "the refusal is on screen:\n{refused}"
    );
    assert!(
        refused.contains("/enhance"),
        "the draft survives a refusal:\n{refused}"
    );

    // Typing again is a different line, so the answer to the last one goes.
    screen.frame(&mut world, typed("d"));
    let edited = screen.text(&mut world);
    assert!(
        !edited.contains("unknown command"),
        "an edit retires the note:\n{edited}"
    );
}

/// Help renders as help, not as the reply JSON every other query prints — and
/// one verb's page is its detail, at the seat that asked.
#[test]
fn asking_about_one_command_prints_its_page() {
    let mut world = world();
    let screen = Screen::new();
    assert!(screen.idle(&mut world));
    screen.frame(&mut world, typed("/close --help"));
    screen.frame(&mut world, enter());
    let page = screen.text(&mut world);
    assert!(page.contains("/close [id]"), "the usage line:\n{page}");
    assert!(
        page.contains("pre-commit gate"),
        "the page, not the one-liner:\n{page}"
    );
    assert!(!page.contains("\"kind\""), "help is not raw JSON:\n{page}");
}

/// A bare `/` is the whole roster — the same gesture `/help` is (§8.5).
#[test]
fn a_bare_slash_lists_every_command() {
    let mut world = world();
    let screen = Screen::new();
    assert!(screen.idle(&mut world));
    screen.frame(&mut world, typed("/"));
    screen.frame(&mut world, enter());
    let roster = screen.text(&mut world);
    for usage in crate::boundary::help::table().iter().map(|row| row.usage) {
        assert!(roster.contains(usage), "{usage} is unlisted:\n{roster}");
    }
}
