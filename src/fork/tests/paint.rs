//! **S12-T3 cohort-one-path** (the seat half): the fork composer on screen —
//! what it offers, and what each click does.

use super::choices;
use crate::fork::composer::Composer;
use crate::fork::render::render;
use crate::fork::{Attempt, Choices};

fn painted(composer: &mut Composer, choices: &Choices) -> String {
    let mut copy = composer.clone();
    crate::paint_probe::paint(|ui| {
        copy = composer.clone();
        render(ui, &mut copy, choices);
    })
}

/// The seat names itself, offers every fork point, names the **model** each
/// role binds — not merely the role — and offers the world's skills.
#[test]
fn the_composer_paints_every_fire_time_control() {
    let mut c = Composer::seeded(&choices());
    let text = painted(&mut c, &choices());
    assert!(text.contains("Fork from here"), "{text}");
    assert!(text.contains("here") && text.contains("strict"), "{text}");
    assert!(
        text.contains("worker — anthropic/claude-sonnet-5"),
        "the model is visible at the point of choice:\n{text}"
    );
    assert!(text.contains("scribe — anthropic/opus"), "{text}");
    assert!(
        text.contains("bash") && text.contains("read_file"),
        "{text}"
    );
    assert!(text.contains("×1"), "one candidate, said outright:\n{text}");
    assert!(text.contains("Fire"), "{text}");
}

/// A fan paints one numbered block per candidate — the ×N control's whole
/// visible consequence.
#[test]
fn a_fan_paints_one_block_per_candidate() {
    let mut c = Composer::seeded(&choices());
    c.resize(3);
    let text = painted(&mut c, &choices());
    assert!(text.contains("×3"), "{text}");
    for n in ["#1", "#2", "#3"] {
        assert!(text.contains(n), "{n} is missing:\n{text}");
    }
}

/// The ×N buttons step the cohort's width, and the floor holds at the seat as
/// it does in the composer: pressing `−` on one candidate leaves one.
#[test]
fn the_times_buttons_step_the_cohort_and_floor_at_one() {
    let mut c = Composer::seeded(&choices());
    click(&mut c, "+");
    assert_eq!(c.attempts.len(), 2);
    click(&mut c, "−");
    assert_eq!(c.attempts.len(), 1);
    click(&mut c, "−");
    assert_eq!(c.attempts.len(), 1);
}

/// Picking a fork point moves the candidate's policy with it: the ref changes
/// **and** the role lands on one that ref declares, because a role the new
/// config does not carry would be refused at the fork.
#[test]
fn picking_a_fork_point_carries_the_policy_with_it() {
    let mut c = Composer::seeded(&choices());
    c.attempts[0].role = "scribe".to_owned();
    click(&mut c, "strict");
    assert_eq!(c.attempts[0].from, "config/strict");
    assert_eq!(c.attempts[0].role, "worker", "scribe is not on that config");
}

/// Picking a role is picking the model, and picking a skill pins it — both per
/// candidate, both a toggle away.
#[test]
fn picking_a_role_and_a_skill_edits_that_candidate() {
    let mut c = Composer::seeded(&choices());
    click(&mut c, "scribe — anthropic/opus");
    assert_eq!(c.attempts[0].role, "scribe");
    click(&mut c, "bash");
    assert_eq!(c.attempts[0].skills, vec!["bash".to_owned()]);
    click(&mut c, "bash");
    assert!(c.attempts[0].skills.is_empty());
}

/// Fire is unreachable until the composer is ready, and hands the caller the
/// one `true` it acts on when it is.
#[test]
fn fire_is_unreachable_until_the_composer_is_ready() {
    let mut c = Composer::seeded(&choices());
    assert!(!click(&mut c, "Fire"), "an empty goal cannot fire");
    c.goal = "try it the other way".to_owned();
    assert!(click(&mut c, "Fire"));
}

/// A candidate whose ref the seat no longer offers paints no role row — the
/// policy moved out from under the draft, and the composer says so by having
/// nothing to pick rather than by inventing a role.
#[test]
fn a_candidate_on_an_unoffered_ref_offers_no_role() {
    let mut c = Composer::seeded(&choices());
    c.attempts[0] = Attempt {
        from: "config/gone".to_owned(),
        role: "worker".to_owned(),
        skills: Vec::new(),
    };
    let text = painted(&mut c, &choices());
    assert!(!text.contains(" — anthropic/"), "no model to name:\n{text}");
}

/// Moving to a fork point that declares no role leaves the candidate role-less
/// — the composer names no model it cannot honour, and Fire goes out of reach
/// rather than firing a role the config would refuse.
#[test]
fn moving_to_a_ref_that_declares_no_role_leaves_the_candidate_unfireable() {
    let mut offered = choices();
    offered.points.push(crate::fork::ForkPoint {
        label: "bare".to_owned(),
        refspec: "config/bare".to_owned(),
        roles: Vec::new(),
    });
    let mut c = Composer::seeded(&offered);
    c.goal = "try it the other way".to_owned();
    assert!(c.ready());
    click_at(&mut c, "bare", &offered);
    assert_eq!(c.attempts[0].from, "config/bare");
    assert_eq!(c.attempts[0].role, "", "no role to land on");
    assert!(!c.ready());
}

/// Two-frame pointer click on the galley whose text is `label` — the shared
/// click idiom. Returns whether that frame fired.
fn click(composer: &mut Composer, label: &str) -> bool {
    click_at(composer, label, &choices())
}

/// The same click against a stated seat — what a workspace offering a
/// role-less ref is read with.
fn click_at(composer: &mut Composer, label: &str, choices: &Choices) -> bool {
    let choices = choices.clone();
    let ctx = egui::Context::default();
    let run = |input: egui::RawInput, composer: &mut Composer| {
        let mut fired = false;
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                fired = render(ui, composer, &choices);
            });
        });
        fired
    };
    let screen = crate::paint_probe::screen;
    let _ = run(screen(), composer);
    let _ = run(screen(), composer);
    let mut probe = composer.clone();
    let painted = crate::paint_probe::painted_settled(1024.0, 4096.0, |ui| {
        probe = composer.clone();
        render(ui, &mut probe, &choices);
    });
    let pos = painted
        .iter()
        .find(|(text, _)| text == label)
        .map(|(_, rect)| rect.center())
        .expect("the label is on screen");
    let button = |pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::default(),
    };
    run(
        egui::RawInput {
            events: vec![egui::Event::PointerMoved(pos), button(true), button(false)],
            ..screen()
        },
        composer,
    )
}
