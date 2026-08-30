//! Where to cut a string that will not fit (QUALITY G1 "deliberate elision
//! shows an ellipsis and the full value is reachable", L4 "ids are tamed").
//!
//! **One rule: cut where the information is not.** An elision is a claim about
//! which end of a string a human is reading for, and the two kinds of string
//! this crate paints answer that oppositely:
//!
//! - **Prose** — a goal's headline, a ball title, a refusal's reason — is
//!   written front-first. Its head distinguishes it, so a prose cut keeps the
//!   head. Those sites are correct as they stand and are deliberately NOT
//!   routed through here; a module claiming to be the one home for every cut,
//!   while eight prose sites kept their own, would be a false claim.
//! - **A machine string** — an absolute path, a spawned `argv`, an ancestry
//!   chain — is invariant at the front and distinguishing at the back. Every
//!   activity row began
//!   `/home/u/.cache/yog-drive/…/data/yog/`, over half the row, identical on
//!   every line; what told the rows apart — which workspace, which agent — was
//!   exactly what the cut threw away, so a column of different operations
//!   scanned as one repeated string (bl-3aa1). That is [`middle`]'s case, and
//!   it is the only case this module has.
//!
//! The other half of L4 — an **id**, whose distinguishing end is a whole
//! terminal segment rather than a character count — is not a cut at all but a
//! floor, and it already has one home in `nav::convs::id_floor` (bl-63a1). Two
//! spellings of "shorten an id" is how they drift, so this module does not
//! offer one.

/// The fewest characters an elision may leave. Below this a cut has stopped
/// naming anything — `…e` distinguishes nothing from `…d` — so a cap tighter
/// than the floor is raised to it rather than honoured. It is a legibility
/// bound, not a layout one: no caller wants the answer this guards against.
pub(crate) const FLOOR: usize = 16;

/// `s` cut to `max` characters keeping **both ends**, the removed middle marked
/// with a single `…`.
///
/// The head keeps the operation's verb (`litany prompt --name growing`), the
/// tail keeps what the row is actually distinguished by (the workspace leaf,
/// the agent id), and what goes is the invariant run between them. A string
/// already within `max` is returned unchanged — the general path with nothing
/// to do, not a special case. `max` below [`FLOOR`] is raised to it.
pub(crate) fn middle(s: &str, max: usize) -> String {
    let max = max.max(FLOOR);
    let total = s.chars().count();
    if total <= max {
        return s.to_owned();
    }
    // One char is spent on the marker; the rest splits evenly, the head taking
    // the odd character so a cut never favours the end it is throwing away.
    let keep = max - 1;
    let tail_len = keep / 2;
    let head_len = keep - tail_len;
    let head: String = s.chars().take(head_len).collect();
    let tail: String = s.chars().skip(total - tail_len).collect();
    format!("{head}…{tail}")
}

#[cfg(test)]
mod tests {
    use super::{FLOOR, middle};

    /// The witness from the ball, at the activity trail's own cap: the verb
    /// survives at the head, the workspace and agent survive at the tail, and
    /// the invariant path run between them is what goes.
    #[test]
    fn a_machine_string_keeps_the_verb_and_the_thing_that_tells_the_row_apart() {
        let argv = "litany prompt --name growing \
                    /home/u/.cache/yog-drive/quality-20260807T214407Z/data/yog/workspaces/home \
                    20260807T214551Z-2a1181a3";
        let cut = middle(argv, 100);
        assert_eq!(cut.chars().count(), 100, "the cut honours its cap: {cut}");
        assert!(cut.starts_with("litany prompt --name growing"), "{cut}");
        assert!(cut.ends_with("20260807T214551Z-2a1181a3"), "{cut}");
        assert!(cut.contains('…'), "the cut is marked: {cut}");
        assert!(
            !cut.contains("/quality-20260807T214407Z/data/"),
            "the invariant run is what went: {cut}"
        );
    }

    /// Two rows sharing a long invariant prefix stay distinguishable — the
    /// property the ball is actually about ("every row scans as the same
    /// string"). Head-keeping elision fails this at any cap.
    #[test]
    fn two_rows_sharing_a_prefix_do_not_cut_to_the_same_string() {
        let prefix = "/home/u/.cache/yog-drive/quality-20260807T214407Z/data/yog/workspaces/";
        let a = middle(&format!("{prefix}home-alpha"), 40);
        let b = middle(&format!("{prefix}home-beta"), 40);
        assert_ne!(a, b, "the rows must not collapse to one string");
    }

    /// A string that fits is returned byte-identical — no marker, no cut.
    #[test]
    fn a_string_within_the_cap_passes_through_whole() {
        assert_eq!(middle("litany stop", 100), "litany stop");
        // Exactly at the cap is still whole: the boundary is inclusive.
        let exact: String = std::iter::repeat_n('x', 20).collect();
        assert_eq!(middle(&exact, 20), exact);
        assert_eq!(middle(&exact, 19).chars().count(), 19);
    }

    /// A cap below the floor is raised to it rather than honoured: an elision
    /// that leaves three characters has stopped naming anything.
    #[test]
    fn a_cap_below_the_floor_is_raised_to_the_floor() {
        let long: String = std::iter::repeat_n('x', 200).collect();
        assert_eq!(middle(&long, 0).chars().count(), FLOOR);
        assert_eq!(middle(&long, 3).chars().count(), FLOOR);
        assert_eq!(middle(&long, FLOOR).chars().count(), FLOOR);
    }

    /// Multi-byte input is cut on character boundaries, not byte offsets — the
    /// crate's `string_slice` discipline (AGENTS rule 4) as a property.
    #[test]
    fn the_cut_lands_on_character_boundaries() {
        let wide = "→ мойка ✉ 20260807T214551Z-2a1181a3 · окончание вычисления · home-δ";
        let cut = middle(wide, 30);
        assert_eq!(cut.chars().count(), 30, "{cut}");
        assert!(cut.ends_with("home-δ"), "{cut}");
    }
}
