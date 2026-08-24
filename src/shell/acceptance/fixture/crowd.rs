//! **The crowded roster's own bytes** (bl-86a5): how many conversations
//! [`Roster::Crowded`] seats, what each is called, and the one loop that builds
//! them. Split from [`super`] at §12's budget, on the seam the builder already
//! had — the shipped world is one thing, the crowd laid on top of it another.
//!
//! Until this existed no fixture in the suite held more than **one**
//! conversation, so 1900+ tests ran against a §11 column that never once had to
//! divide itself — and the defect they all missed was a list eating the whole
//! column, taking the ⚙ Config entry (the only visible door to the §3.6
//! workspace delete) off the glass with it.
//!
//! [`Roster::Crowded`]: super::build::Roster::Crowded

use crate::git_tree::tests::fixture::Fixture;

/// How many conversations the crowd seats. Enough that the list outgrows the
/// column at **every** size in [`SIZES`](crate::shell::acceptance::SIZES),
/// including the 2560x1700 maximum — a budget defect that only shows at 420x320
/// would be indistinguishable from a window too small to hold anything, and a
/// beat run at a size where the list happens to fit asserts nothing at all.
/// `acceptance::reach` checks that outright rather than trusting this number,
/// so a row that grows taller reddens there instead of quietly going vacuous.
const COUNT: usize = 120;

/// The §3.1 name the crowded wall wears — the word the §3.6 dialog asks the
/// operator to retype, so a beat can name it outright.
pub(in crate::shell::acceptance) const WALL: &str = "ops";

/// The crowd's `n`th conversation, by the id its row is named after — a needle
/// no other surface in the window paints, so a beat can count how many of them
/// reached the glass without a galley from somewhere else answering for one.
/// One hyphen-free token, so the §2.3 descent grammar reads it as a **root**
/// and the list shows it as one row. **Fixed width**, so no needle is a prefix
/// of another: a beat that counts how many of these reached the glass reads a
/// run's opening glyphs (elision is a different rule with a different guard),
/// and `crowdrow12` inside `crowdrow120` would have it count a row twice and
/// so believe a list fits when it does not.
pub(in crate::shell::acceptance) fn name(n: usize) -> String {
    format!("crowdrow{n:03}")
}

/// Every name the crowd seats, the world's own opening conversation excluded —
/// what a beat counts against to say the list really outgrew its column.
pub(in crate::shell::acceptance) fn names() -> Vec<String> {
    (2..=COUNT).map(name).collect()
}

/// Build the crowd onto `fx` — **bare** branches off the same lineage, one
/// `git branch` apiece. What these beats need is *rows*, not transcripts: the
/// world's opening conversation is the fully built one every other surface
/// reads, and a row is a ref (ARCH §2.3). Built the ordinary way this fixture
/// cost a dozen forks, two worktrees and a merge per row, forty times over, in
/// every world every beat here opens — enough to lean on a loaded box rather
/// than measure a layout on it.
pub(super) fn seat(fx: &Fixture) {
    for n in 2..=COUNT {
        fx.build_bare_agent(&name(n));
    }
}
