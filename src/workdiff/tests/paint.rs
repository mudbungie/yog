//! **S11-T3 work-paint**: what the Work tab puts on screen — the range it is
//! reading, each changed file's churn, the click that picks one, and the three
//! declines said in words rather than shown as an empty list.

use crate::files_view::{PREVIEW_CAP, Preview};
use crate::workdiff::{Attempt, Change, Churn, FileChurn, WorkFile, render};

fn diffed() -> Vec<Attempt> {
    vec![Attempt {
        project: "proj".to_owned(),
        ball_id: "bl-1".to_owned(),
        change: Change::Diff {
            target: "main".to_owned(),
            source: "work/bl-1".to_owned(),
            target_oid: "aaaa1111bbbb".to_owned(),
            source_oid: "cccc2222dddd".to_owned(),
            files: vec![
                FileChurn {
                    path: "src/a.rs".to_owned(),
                    churn: Churn::Text {
                        added: 12,
                        removed: 3,
                    },
                },
                FileChurn {
                    path: "logo.png".to_owned(),
                    churn: Churn::Binary,
                },
            ],
            truncated: true,
        },
    }]
}

fn painted(attempts: &[Attempt], patch: Option<&Preview>) -> String {
    let mut sel = None;
    crate::paint_probe::paint(|ui| {
        render(ui, attempts, patch, &mut sel);
    })
}

/// The tab names the ball, its project, the exact range, both commits, and
/// every changed file with its churn — binary said as binary.
#[test]
fn the_tab_names_the_range_and_every_changed_file() {
    let text = painted(&diffed(), None);
    assert!(text.contains("bl-1"), "{text}");
    assert!(text.contains("proj"), "{text}");
    assert!(text.contains("main..work/bl-1"), "{text}");
    assert!(text.contains("aaaa111"), "the target commit, short: {text}");
    assert!(text.contains("cccc222"), "the source commit, short: {text}");
    assert!(text.contains("+12 -3  src/a.rs"), "{text}");
    assert!(text.contains("binary  logo.png"), "{text}");
    assert!(text.contains("more files changed"), "{text}");
    assert!(text.contains("pick a file"), "{text}");
}

/// Clicking a file row picks it — ball and path together, because a path alone
/// would not say which attempt's diff it belongs to.
#[test]
fn clicking_a_file_picks_it_for_its_patch() {
    let attempts = diffed();
    let ctx = egui::Context::default();
    let mut sel: Option<WorkFile> = None;
    let frame = |input: egui::RawInput, sel: &mut Option<WorkFile>| {
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render(ui, &attempts, None, sel);
            });
        });
    };
    let screen = crate::paint_probe::screen;
    frame(screen(), &mut sel);
    let painted = crate::paint_probe::painted_settled(1024.0, 4096.0, |ui| {
        render(ui, &attempts, None, &mut None);
    });
    let pos = painted
        .iter()
        .find(|(text, _)| text == "+12 -3  src/a.rs")
        .map(|(_, rect)| rect.center())
        .expect("the row is on screen");
    let button = |pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::default(),
    };
    frame(
        egui::RawInput {
            events: vec![egui::Event::PointerMoved(pos), button(true), button(false)],
            ..screen()
        },
        &mut sel,
    );
    assert_eq!(
        sel,
        Some(WorkFile {
            ball: "bl-1".to_owned(),
            path: "src/a.rs".to_owned(),
        })
    );
}

/// The picked file's patch paints through the same three classes a file
/// preview does, cap and size said outright.
#[test]
fn the_picked_files_patch_paints_bounded() {
    let attempts = diffed();
    let text = painted(&attempts, Some(&Preview::Text("+added line".to_owned())));
    assert!(text.contains("+added line"), "{text}");
    let empty = painted(&attempts, Some(&Preview::Text(String::new())));
    assert!(empty.contains("came back empty"), "{empty}");
    let binary = painted(&attempts, Some(&Preview::Binary { size: 41 }));
    assert!(binary.contains("41 bytes"), "{binary}");
    let long = painted(
        &attempts,
        Some(&Preview::Truncated {
            text: "@@".to_owned(),
            size: 99_999,
        }),
    );
    assert!(
        long.contains(&format!("{} KiB of 99999 bytes", PREVIEW_CAP / 1024)),
        "{long}"
    );
}

/// The four things that are not a diff are four sentences, and none of them is
/// a blank list: no claim at all, an unreadable project, an absent ref, and a
/// branch that has changed nothing yet.
#[test]
fn every_decline_is_said_in_words() {
    assert!(painted(&[], None).contains("holds no ball"));

    let attempt = |change| {
        vec![Attempt {
            project: "proj".to_owned(),
            ball_id: "bl-1".to_owned(),
            change,
        }]
    };
    let unreadable = painted(&attempt(Change::Unreadable), None);
    assert!(unreadable.contains("cannot be read here"), "{unreadable}");

    let absent = painted(
        &attempt(Change::Absent {
            target: "main".to_owned(),
            source: "work/bl-1".to_owned(),
            missing: vec!["work/bl-1".to_owned()],
        }),
        None,
    );
    assert!(absent.contains("comparing main..work/bl-1"), "{absent}");
    assert!(absent.contains("no work/bl-1"), "{absent}");

    let quiet = painted(
        &attempt(Change::Diff {
            target: "main".to_owned(),
            source: "work/bl-1".to_owned(),
            target_oid: "aaaa1111".to_owned(),
            source_oid: "aaaa1111".to_owned(),
            files: Vec::new(),
            truncated: false,
        }),
        None,
    );
    assert!(quiet.contains("nothing changed yet"), "{quiet}");
}
