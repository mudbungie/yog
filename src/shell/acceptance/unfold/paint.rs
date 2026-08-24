//! **What a fold states, and what it reveals** (bl-fa82) — the §11 unfold at
//! the paint layer, split from [`super`] at §12's budget on the seam that file
//! now runs on: the three-generation fixture and the reads over it are the
//! parent's, one claim per child, and this is the claim no hand makes — what
//! the list must show shut and open, however it got there.
//!
//! Asserted on **geometry as well as text**, in `name_column`'s discipline and
//! for its reason: the claim this ball makes is *where* a child row sits, and a
//! string assertion passes on a tree that paints the child flush against its
//! parent — which is the whole defect it would exist to catch. So the title's
//! left edge is measured per depth, and the field is measured against the title
//! it is pinned to the right of.
//!
//! Two directions everywhere, because a fold has two states and only asserting
//! the open one would pass on a list that could not close: the collapsed frame
//! must paint **no** elbow and **no** child name, the open one must paint both.

use super::{CHILD, GRANDCHILD, elbows, left_of, name_of, nested_world, painted, rows};

/// Shut, the row states the ruling's two numbers — `direct` then
/// `total`, which the three-generation fixture makes different — behind the
/// collapsed arrow, and its descent is not in the list: no child name, no
/// elbow. The field is **right** of the title, which is the seat bl-b9e3's
/// name-column rule requires of every conditional mark.
#[test]
fn a_collapsed_row_states_direct_and_total_right_of_its_title_and_hides_the_descent() {
    let mut world = nested_world();
    let list = rows(&world);
    assert_eq!(
        list.len(),
        1,
        "collapsed, the three-generation conversation is one row: {list:?}"
    );
    let shut = painted(&mut world);

    let title = left_of(&shut, &list[0].1).expect("the root row paints its title");
    let field = left_of(&shut, "▶ 1/2").unwrap_or_else(|| {
        panic!(
            "the subagent field states ▶ direct/total — 1 dispatched here, 2 under it \
             altogether:\n{:?}",
            shut.iter().map(|(t, _)| t).collect::<Vec<_>>()
        )
    });
    assert!(
        field > title,
        "the field rides the trailing right-pinned group: {field} <= {title}"
    );
    assert_eq!(elbows(&shut), 0, "a shut list draws no reply elbow");
    // The needles are [`name_of`]'s, and the same two are asserted **present**
    // below on the frame that opens the fold: two directions inside the one
    // beat, which is what makes an absence claim evidence of anything.
    let hidden: Vec<String> = [CHILD, GRANDCHILD]
        .iter()
        .map(|id| name_of(&world, id))
        .collect();
    for name in &hidden {
        assert!(
            left_of(&shut, name).is_none(),
            "{name} is folded away, so nothing paints it"
        );
    }
    world.state.expanded.insert("c-1".to_owned());
    world.state.expanded.insert(CHILD.to_owned());
    let open = painted(&mut world);
    for name in &hidden {
        assert!(
            left_of(&open, name).is_some(),
            "and the very same needle lands once the fold opens: {name}"
        );
    }
}

/// Open, each generation is a row of the same anatomy indented past the one
/// above it — the per-depth title edge §11 promises — wearing the reply elbow,
/// and each row's field states **its own** subtree: the middle row says 1/1
/// where the root says 1/2, which is the whole of "a row is the subtree rooted
/// at its agent".
#[test]
fn unfolding_indents_each_generation_past_the_one_above_it() {
    let mut world = nested_world();
    world.state.expanded.insert("c-1".to_owned());
    world.state.expanded.insert(CHILD.to_owned());
    let list = rows(&world);
    assert_eq!(
        list.iter().map(|(d, _)| *d).collect::<Vec<_>>(),
        vec![0, 1, 2],
        "three generations, one row each: {list:?}"
    );
    let painted = painted(&mut world);

    // The title edges: strictly increasing, depth by depth. Equal edges is the
    // defect this beat exists for — a child painted flush under its parent
    // reads as a sibling, and every string assertion in the file would pass.
    let edges: Vec<f32> = list
        .iter()
        .map(|(_, name)| left_of(&painted, name).unwrap_or_else(|| panic!("{name} paints")))
        .collect();
    for pair in edges.windows(2) {
        assert!(
            pair[1] > pair[0] + 1.0,
            "each depth's title edge sits right of the one above it: {edges:?}"
        );
    }
    // Two children, two elbows — and the elbow is ahead of the title it belongs
    // to, not somewhere in the trailing group.
    assert_eq!(elbows(&painted), 2, "one reply elbow per revealed child");
    let elbow_left = painted
        .iter()
        .filter(|(text, _)| text.ends_with(crate::theme::ELBOW))
        .map(|(_, rect)| rect.min.x)
        .reduce(f32::min)
        .unwrap();
    assert!(
        elbow_left < edges[1],
        "the elbow leads the child's row: {elbow_left} >= {}",
        edges[1]
    );
    // Each row folds its own subtree: the arrow flips where it was clicked, and
    // the numbers are that row's, not the conversation's.
    assert!(
        left_of(&painted, "▼ 1/2").is_some() && left_of(&painted, "▼ 1/1").is_some(),
        "the root says 1/2 open, the middle row 1/1 open:\n{:?}",
        painted.iter().map(|(t, _)| t).collect::<Vec<_>>()
    );
    assert!(
        left_of(&painted, "▶ 1/2").is_none(),
        "and no row still wears the shut arrow it was opened from"
    );
}
