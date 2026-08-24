//! §11 / QUALITY L4's *other* elision question, driven through the real trail:
//! not **whether** a row elides but **where** it cuts (bl-3aa1).
//!
//! [`super::elision`] proves rule 1b — that a control is never the thing that
//! goes. This is the same evidence answering the complementary claim, and it is
//! its own subject: an activity row is one long argv, so what the operator can
//! still read after the cut is the end that tells two ops apart, not the head
//! they share.
//!
//! `OpRow::summary`'s own tables (`opslog/rows/tests.rs`) pin the cut as a
//! string; this pins the **painted glyphs**, because an egui galley truncated
//! to `…` still reports the whole string from `Galley::text()` and a dump read
//! that way is blind to exactly this defect (bl-bc06).

/// **The activity row's cut reaches the glass keeping the end that tells rows
/// apart** (bl-3aa1, QUALITY L4 "ids are tamed … floor to the terminal segment
/// or middle-elide, with the full value one gesture away").
///
/// `OpRow::summary`'s own tables (`opslog/rows/tests.rs`) pin the string; this
/// pins the **painted glyphs**, which is a different claim: the row is laid in
/// the real trail, and what an operator can read is the galley's laid-out
/// glyphs, never `Galley::text()` (bl-bc06 — the input string survives every
/// truncation, so a dump read that way is blind to exactly this defect).
///
/// Two ops that share the audit's invariant prefix and differ only at the end
/// are painted together, and asserted in both directions: each row's own tail
/// is on screen, the invariant run between the ends is not, and the two rows
/// are not the same line. A head-keeping cut passes none of the three.
#[test]
fn the_activity_trail_paints_the_tail_that_tells_two_ops_apart() {
    const PREFIX: &str = "lernie prompt --name growing \
         /home/u/.cache/yog-drive/quality-20260807T214407Z/data/yog/workspaces/";
    const HOME: &str = "home 20260807T214551Z-2a1181a3";
    const SCRATCH: &str = "scratch 20260807T220107Z-c0ffeeba";
    let mut world = crate::shell::acceptance::fixture::world();
    for leaf in [HOME, SCRATCH] {
        crate::opslog::append(
            world.model.state_root(),
            &crate::opslog::OpEntry {
                ts: "1785630266".into(),
                argv: vec![format!("{PREFIX}{leaf}")],
                cwd: "/proj".into(),
                exit: 0,
                stdout: String::new(),
                stderr: String::new(),
                origin: crate::opslog::Origin::default(),
            },
        )
        .expect("the fixture world takes an ops line");
    }
    world.model.after_lernie_verb();
    world.converge();
    world.converge();
    let mut open = true;
    // The trail is a `Reply::Ops` since bl-adcb, so the rows exist only once
    // something has answered — [`super::wire::wired`] is the settle-then-render
    // dance the window does over half a second, paid out here in three passes.
    let painted = super::wire::wired(&mut world, &mut |model, _| {
        crate::paint_probe::paint(|ui| {
            crate::shell::activity::accessory(ui, model, &mut open);
        })
    });
    let rows: Vec<&str> = painted
        .lines()
        .filter(|line| line.contains("lernie prompt"))
        .collect();
    assert_eq!(rows.len(), 2, "both ops paint one row each:\n{painted}");
    assert_ne!(rows[0], rows[1], "the two rows must not read alike");
    for (row, leaf) in rows.iter().zip([HOME, SCRATCH]) {
        assert!(
            row.ends_with(leaf),
            "the row ends with what names it: {row}"
        );
        assert!(row.contains('…'), "the cut is marked on screen: {row}");
        assert!(
            !row.contains("quality-20260807T214407Z"),
            "and the invariant run between the ends is what went: {row}"
        );
    }
}
