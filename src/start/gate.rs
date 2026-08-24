//! **The start flow's first rung** (DESIGN §8.1, §8.3; bl-1fd0): can the wall
//! this start aims at reach a model at all, and if not, what the pane says
//! instead of inviting a goal.
//!
//! The operator ruling this exists for: *"On a wall holding no usable provider
//! credential, typing a goal and hitting Enter will work zero percent of the
//! time — the conversation is born, immediately dies on no-models, and the
//! operator learns it from a dead row (or from nothing at all). The pane is
//! inviting the one act that cannot succeed while hiding the one act that must
//! come first."* It was hit live, twice in one evening, and both first goals
//! were wasted.
//!
//! **It is a pure read of a table the frame already holds** — brazen's
//! effective provider rows (§5.1 #20/#21), folded by the same §8.3 `ask` that
//! populates the Login roster. No network, no spawn, no per-frame cost.
//!
//! **A keyless row is not a credential, and it is not an escape either.** The
//! ruling reads *"any row `stored` or `not required`"*. Taken literally that is
//! vacuous: brazen merges its built-in table under every config, so `ollama`
//! and `claude-code` read `not required` on **every** wall there can be, and a
//! predicate they satisfy is a predicate nothing ever fails — the rung would
//! not appear on the wall it was ruled for. Nor are they what a doomed start
//! was routed to: both claim no model prefixes and are reached only by an
//! explicit `--provider`, so a start whose role names an uncredentialled row
//! dies exactly as the ruling describes with both of them sitting in the table.
//! So `not required` does not make a wall ready; it only changes what the rung
//! says, because an operator looking at two keyless rows deserves to be told
//! why they do not count.
//!
//! **Every other credential spelling IS ready**, including the two the ruling's
//! enumeration omitted (`ambient`, `inline`) and any this build has never heard
//! of. Refusing a wall whose rows carry a credential a run would actually spend
//! would block a working setup, and no surface here refuses on the strength of
//! a question that went unanswered — the rule `providers::capability` already
//! keeps for the `protocol` column.
//!
//! **What this still cannot see** (the residual, recorded rather than hidden):
//! *which* row a start routes to. That is `roles.<r>.provider` on the config
//! branch — several git reads, and §9.4's subject, not the pane's — so the gate
//! judges the wall rather than the route. It is therefore exactly right when
//! nothing at all is signed in and conservative for an operator whose roles all
//! name a keyless row: they are told to sign in when their setup needs no
//! sign-in. The remedy is the same one sentence, so the cost is a sentence.

use crate::config_edit::brazen::{MISSING, NOT_REQUIRED, ProviderRow};

/// The rung's refusal, and the whole of what Send says instead of firing. One
/// sentence, as the ruling asks — and short, because it is painted above the
/// goal box in a pane the operator sized for typing, where every line it takes
/// is a line the roster beneath it does not get.
const NO_CREDENTIAL: &str = "nothing on this wall is signed in, so a goal started here reaches no model. Sign a \
     provider in below — your draft is kept.";

/// Appended when the wall's only credential-free rows are the keyless ones, so
/// the operator reading `no credential needed` beside two of them is told why
/// they are not the answer.
const BUT_KEYLESS: &str = " Its keyless rows are reached only by an explicit provider.";

/// What one wall's provider table says about reaching a model (§8.1) — two
/// booleans over the `credential` column, and the whole of what the gate reads.
///
/// Derived where the §8.3 roster is derived, off the same rows, so the rung and
/// the roster beneath it can never disagree about the same wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WallCredit {
    /// Any row carrying a credential a run would actually spend — `stored`,
    /// `ambient`, `inline`, and any spelling this build cannot read.
    pub credentialed: bool,
    /// Any row that needs no credential at all (`not required`). Never a reason
    /// to call the wall ready — see the module note — only a reason to say more.
    pub keyless: bool,
}

impl WallCredit {
    /// Fold the effective table. An **empty** table reads as neither, which is
    /// the honest answer for a wall brazen could not be asked about and is the
    /// same refusal — a wall with no rows has nothing to route to either.
    pub fn read(rows: &[ProviderRow]) -> Self {
        Self {
            credentialed: rows
                .iter()
                .any(|r| r.credential != MISSING && r.credential != NOT_REQUIRED),
            keyless: rows.iter().any(|r| r.credential == NOT_REQUIRED),
        }
    }
}

/// The start pane's first rung (§8.1) — three states, total over the two facts
/// the pane can hold about its target wall.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartGate {
    /// A row carries a credential. **Today's flow, byte for byte** — no rung,
    /// no sentence, nothing added to the pane.
    Ready,
    /// No row carries one. The rung paints the reason and the §8.3 roster
    /// beneath it, and Send says the reason instead of firing. The goal box
    /// stays draftable throughout: the draft is what the ruling is protecting.
    SignIn(WallCredit),
    /// The workspace is hosted by a §8.2 entry. This box reads its OWN wall's
    /// brazen, which is not the wall the agents will run on, so the rung says
    /// so rather than answering with the wrong table. **The seam bl-61bf
    /// fills**: when a remote wall's provider read and login act have a
    /// designed home, this variant becomes a [`SignIn`](Self::SignIn) aimed at
    /// the host. Never a refusal — an unread wall is not a wall known to be
    /// empty.
    Unknown(String),
}

impl StartGate {
    /// Read the gate for a start aimed at a workspace `channel` hosts (`None`
    /// for this window's own engine) whose wall folded to `credit`.
    pub fn read(channel: Option<String>, credit: WallCredit) -> Self {
        match channel {
            Some(leaf) => Self::Unknown(leaf),
            None if credit.credentialed => Self::Ready,
            None => Self::SignIn(credit),
        }
    }

    /// **What Send says instead of firing**, or `None` where nothing here
    /// refuses. Read by the button's disabled state **and** by the §11 Enter
    /// binding, which is the other hand on the same trigger — a pointer and a
    /// keypress cannot disagree about whether the wall can run.
    pub fn refusal(&self) -> Option<String> {
        matches!(self, Self::SignIn(_)).then(|| NO_CREDENTIAL.to_owned())
    }

    /// The rung's own sentence — the one line saying why the goal box is not
    /// yet the point. `None` on a wall that plainly runs.
    pub fn note(&self) -> Option<String> {
        match self {
            Self::Ready => None,
            Self::SignIn(credit) if credit.keyless => Some(format!("{NO_CREDENTIAL}{BUT_KEYLESS}")),
            Self::SignIn(_) => Some(NO_CREDENTIAL.to_owned()),
            Self::Unknown(leaf) => Some(format!(
                "this workspace is hosted by the entry {leaf:?}, and its providers cannot be \
                 read from here yet — nothing on this pane says whether a goal started here \
                 will reach a model."
            )),
        }
    }

    /// Whether the §8.3 sign-in roster is offered beneath the sentence. Not for
    /// [`Unknown`](Self::Unknown): the rows this box holds belong to the wrong
    /// wall, and offering them would be the lie the honest sentence replaces.
    pub fn roster(&self) -> bool {
        matches!(self, Self::SignIn(_))
    }
}
