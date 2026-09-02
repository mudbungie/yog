//! **What a `bl create`/`bl update` carries** (§8.2) — the `bl` family's
//! payload vocabulary, beside the argv fold that spends it.
//!
//! Split out of [`super`](crate::actions::verbs) at §12's budget on the seam
//! that file's doc already drew: the verbs act on *a ball in a project*, this is
//! *what a ball is made of*, and only the second of those grows every time
//! balls learns a field.
//!
//! It is the **boundary's** payload as well as the executor's:
//! [`Verb::Create`](super::Verb::Create) and [`Verb::Update`](super::Verb::Update)
//! — the family's carrier, which the §8.5 roster holds one row of — carry these
//! types whole.
//! Before bl-dbde the same fact was written twice — once as the roster's own
//! variant fields and once as a struct here, bridged by an `Update::of` that
//! re-cloned them — which is the two-representations drift the house rule names.
//! One vocabulary means a fact balls learns is added in ONE place.

use super::{AS, BODY, NOTE, TITLE};

// The scheduling flags, pinned to `bl create --skill` / `bl update --skill`.
const PRIORITY: &str = "-p";
const NO_PRIORITY: &str = "--no-priority";
const TAG: &str = "-t";
const NO_TAG: &str = "--no-tag";
const PARENT: &str = "--parent";
const NO_PARENT: &str = "--no-parent";
const NEEDS: &str = "--needs";
const NO_NEEDS: &str = "--no-needs";

/// One **scheduling fact** applied to a ball, in balls' own vocabulary
/// (bl-dbde). These are the four the §11 board already READS and the §4.3 fleet
/// already selects on — priority orders the ready rows, tags carry policy, the
/// parent draws the tree and a blocker edge decides what is ready at all — so
/// before this a remote seat could arm a fleet it had no way to schedule.
///
/// **Two shapes, not four.** A fact that is set or cleared is an `Option`
/// ([`Priority`](Self::Priority), [`Parent`](Self::Parent)); one that is added
/// or dropped carries its value and a direction ([`Tag`](Self::Tag),
/// [`Needs`](Self::Needs)). A *list* rather than four typed options because
/// tags repeat, which no field-per-fact shape expresses.
///
/// **`--subtask-of` and `--blocks` earn no spelling**, being derivable as a
/// second gesture: `--subtask-of E` is `--parent E` on this ball plus an update
/// of E carrying `Needs { edge: "<this>:close" }`. The surface stays four.
///
/// **yog validates none of it.** balls owns this grammar and refuses a cycle
/// naming it (§8.2: the substrate's stderr is the product), so a second opinion
/// here would be a second representation of balls' own rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Field {
    /// `-p N` / `--no-priority` — higher sorts first on the board.
    Priority(Option<i64>),
    /// `-t TAG` / `--no-tag TAG`, repeatable.
    Tag { tag: String, on: bool },
    /// `--parent ID` / `--no-parent` — containment; it gates nothing itself.
    Parent(Option<String>),
    /// `--needs ID[:OP]` / `--no-needs ID` — this ball's own blocker edge.
    Needs { edge: String, on: bool },
}

/// `bl create <title> --as <name> [--body B] [fields…]`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Create {
    pub title: String,
    pub body: Option<String>,
    pub fields: Vec<Field>,
}

/// The field edits `bl update` carries from the ball editor (§11 ball detail):
/// a retitle, a body rewrite (the living document), a journal note, and the
/// scheduling facts above. All optional — an all-empty update still restamps
/// `updated` (bl's note commit).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Update {
    pub title: Option<String>,
    pub body: Option<String>,
    pub note: Option<String>,
    pub fields: Vec<Field>,
}

impl Field {
    /// The argv this application spells. **A clearing form is empty at create**
    /// — a new ball's fields start empty, so clearing one is the general path
    /// at zero input rather than a create/update split in the grammar, and
    /// `bl create` has no `--no-…` flag to spend on it.
    pub(crate) fn argv(&self, creating: bool) -> Vec<String> {
        match self {
            Self::Priority(Some(n)) => set(PRIORITY, &n.to_string()),
            Self::Priority(None) => clear(NO_PRIORITY, creating),
            Self::Parent(Some(id)) => set(PARENT, id),
            Self::Parent(None) => clear(NO_PARENT, creating),
            Self::Tag { tag, on } => pair(TAG, NO_TAG, tag, *on, creating),
            Self::Needs { edge, on } => pair(NEEDS, NO_NEEDS, edge, *on, creating),
        }
    }
}

impl Create {
    /// The `bl create` argv this payload spells, after the verb.
    pub(crate) fn argv(&self, name: &str) -> Vec<String> {
        let mut args = vec![self.title.clone(), AS.to_owned(), name.to_owned()];
        push_opt(&mut args, BODY, self.body.as_ref());
        push_fields(&mut args, &self.fields, true);
        args
    }
}

impl Update {
    /// The `bl update` argv this payload spells, after the id.
    pub(crate) fn argv(&self, name: &str) -> Vec<String> {
        let mut args = vec![AS.to_owned(), name.to_owned()];
        for (flag, value) in [
            (TITLE, self.title.as_ref()),
            (BODY, self.body.as_ref()),
            (NOTE, self.note.as_ref()),
        ] {
            push_opt(&mut args, flag, value);
        }
        push_fields(&mut args, &self.fields, false);
        args
    }
}

/// One optional `--flag value` pair, appended when the operator set it.
fn push_opt(args: &mut Vec<String>, flag: &str, value: Option<&String>) {
    if let Some(text) = value {
        args.push(flag.to_owned());
        args.push(text.clone());
    }
}

/// The scheduling facts, **in the order they were said** — two writes of one
/// fact do not commute, so the list is applied as typed.
fn push_fields(args: &mut Vec<String>, fields: &[Field], creating: bool) {
    args.extend(fields.iter().flat_map(|field| field.argv(creating)));
}

fn set(flag: &str, value: &str) -> Vec<String> {
    vec![flag.to_owned(), value.to_owned()]
}

fn clear(flag: &str, creating: bool) -> Vec<String> {
    if creating {
        Vec::new()
    } else {
        vec![flag.to_owned()]
    }
}

fn pair(on_flag: &str, off_flag: &str, value: &str, on: bool, creating: bool) -> Vec<String> {
    match (on, creating) {
        (true, _) => set(on_flag, value),
        (false, true) => Vec::new(),
        (false, false) => set(off_flag, value),
    }
}
