//! **The `bl` family's carrier type** (bl-92d3) — what the §8.5 action roster
//! holds one row of, split off [`super`] at §12's pre-split band on the seam
//! the family already has: the parent spends an argv, this is the intent the
//! boundary carries to it.

use super::edit;

/// **The `bl` family as ONE verb** (bl-92d3) — the five acts on a ball in a
/// project, carried by the family's own type instead of by five rows of the
/// §8.5 action roster.
///
/// The fold the monitor's, the fleet's, the routing leg's and the §3.8 fan's
/// families each took, on the seam every layer beneath already draws: this
/// module holds all five executors, `boundary::codec::balls` all five
/// spellings, `boundary::line::balls` their readers, `boundary::answer::balls`
/// and `boundary::reply::balls` their answer. The carrier now says what those
/// four files were already saying, and
/// [`action`](crate::boundary::action) is four rows narrower — which is the
/// whole point: that roster rested one line under §12's wall, where the
/// cheapest way past it for whoever touched it next was exactly the shave the
/// rule forbids.
///
/// **The fold is in the carrier, never in the surface.** Each of the five still
/// spells as its own slash verb, its own envelope `op` and its own help page,
/// and still carries its whole parameter set. Nothing about what an operator
/// types or what crosses the wire moves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verb {
    /// `bl close <id> --as <name>` (§8.2) — `name` is the ball's bound
    /// workspace name (§3.2), never the operator `$USER`.
    Close {
        project: String,
        id: String,
        name: String,
    },
    /// `bl claim <id> --as <name>` — assign a ready ball (§8.2/§3.2).
    Assign {
        project: String,
        id: String,
        name: String,
    },
    /// `bl unclaim <id> --as <name>` — release (§8.2/§3.2).
    Release {
        project: String,
        id: String,
        name: String,
    },
    /// `bl create <title> --as <name> [fields…]` (§8.2): the whole payload is
    /// [`edit::Create`], which owns the argv fold that spends it (bl-dbde).
    Create {
        project: String,
        name: String,
        fields: edit::Create,
    },
    /// `bl update <id> --as <name> [fields…]` (§8.2), payload [`edit::Update`].
    /// One vocabulary, so a fact balls learns is added in one place instead of
    /// in the roster, the codec's field list and a second struct beside them.
    Update {
        project: String,
        id: String,
        name: String,
        fields: edit::Update,
    },
}

impl Verb {
    /// The project this act mutates — the §8.2 after-verb ball refresh's
    /// subject ([`Action::project`](crate::boundary::Action::project)).
    ///
    /// Every member names one, which is what makes the family a family; the
    /// address table therefore reads one arm here instead of five of its own,
    /// and a sixth member cannot be added without answering this question.
    pub fn project(&self) -> String {
        match self {
            Self::Close { project, .. }
            | Self::Assign { project, .. }
            | Self::Release { project, .. }
            | Self::Create { project, .. }
            | Self::Update { project, .. } => project.clone(),
        }
    }
}
