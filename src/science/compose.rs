//! The fan group's four affordances, **composed as dispatches** (VISION V3
//! items 1–2, bl-77bc): each click on the §11 group card becomes text in the
//! one composer, and the operator's Enter is what fires it. Nothing here posts
//! anything — the affordance composes, the boundary's existing doors spend.
//!
//! That is V3.1's ruling made mechanical: Judge and Synthesize are **V2's fire
//! path** — an ordinary dispatch whose *goal* carries the candidates' exact
//! terminal refs, which the science rows already state — so their affordances
//! seed the ordinary new-conversation composer and add no fan-in primitive.
//! Deliver and Retire are the §3.8 boundary gestures the line already spells,
//! so their affordances seed the line (`/deliver <handle> …` awaits the
//! operator's summary, because a delivery subject is the operator's statement
//! of what landed and yog does not invent one).

use super::Attempt;
use crate::workdiff::Change;

/// One affordance click on the group card — what the operator asked for,
/// before it is words. The card returns it; [`draft`] turns it into the text
/// the seat puts in the composer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// Dispatch a judge over the cohort (V3.1) — a read-only advisory child.
    Judge,
    /// Dispatch a synthesizer over the cohort (V3.1) — itself an ordinary
    /// attempt on the same target when it writes project bytes (§4.10 item 1).
    Synthesize,
    /// **Deliver candidate** (V3.2) — the acceptance gesture, by handle.
    Deliver { handle: String },
    /// Retire one candidate — release its worktree, retention keeping the ref.
    Retire { handle: String },
}

/// The composed text and which composer it belongs in: a goal fires a **new
/// conversation** (the ordinary bare start), a line runs in whichever composer
/// is open (`/`-led, so the seat's mode cannot bend it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    pub text: String,
    /// `true` seeds the new-conversation composer (clearing the selection is
    /// the caller's §11 gesture); `false` is a line, valid in any composer.
    pub new_conversation: bool,
}

/// Compose `intent` over the rows the card rendered. `None` when there is
/// nothing to compose about — a judge over no candidates is not a dispatch.
pub fn draft(intent: &Intent, rows: &[Attempt]) -> Option<Draft> {
    match intent {
        Intent::Judge => goal(rows, JUDGE),
        Intent::Synthesize => goal(rows, SYNTHESIZE),
        Intent::Deliver { handle } => Some(Draft {
            text: format!("/deliver {handle} "),
            new_conversation: false,
        }),
        Intent::Retire { handle } => Some(Draft {
            text: format!("/retire {handle}"),
            new_conversation: false,
        }),
    }
}

/// What a judge is asked to do, ahead of the refs.
const JUDGE: &str = "Judge the candidate attempts below. Each is one isolated attempt branch in \
     the project repository, all forked from the same base toward the same \
     target. Read each candidate at its exact commit and reply with a verdict \
     per candidate: approve or reject, and why. Do not merge or deliver \
     anything.";
/// What a synthesizer is asked to do, ahead of the same refs.
const SYNTHESIZE: &str = "Synthesize one result from the candidate attempts below. Each is one \
     isolated attempt branch in the project repository, all forked from the \
     same base toward the same target. Take the best of each; your own work is \
     an ordinary attempt on that same target.";

/// The goal both dispatches share: the ask, the obligation, then one line per
/// candidate carrying its exact refs (V3.1 — handle, attempt branch tip, base,
/// target). `None` when no candidate has a resolved branch to cite.
fn goal(rows: &[Attempt], ask: &str) -> Option<Draft> {
    let candidates: Vec<&Attempt> = rows
        .iter()
        .filter(|row| row.diff.handle.is_some())
        .collect();
    let cited: Vec<String> = candidates.iter().filter_map(|row| cite(row)).collect();
    // The obligation line anchors on the first candidate whose branch
    // resolved — the same set the citations come from, so a cohort that cites
    // anything can always state its target.
    let (project, ball, target, target_oid) =
        candidates.iter().find_map(|row| match &row.diff.change {
            Change::Diff {
                target, target_oid, ..
            } => Some((&row.diff.project, &row.diff.ball_id, target, target_oid)),
            _ => None,
        })?;
    (!cited.is_empty()).then(|| Draft {
        text: format!(
            "{ask}\n\nProject {project}, ball {ball}, target {target} at {target_oid}.\n{}",
            cited.join("\n")
        ),
        new_conversation: true,
    })
}

/// One candidate's citation line, exact refs only: a candidate whose branch
/// did not resolve has no commit to cite and earns no line.
fn cite(row: &Attempt) -> Option<String> {
    let handle = row.diff.handle.as_ref()?;
    let Change::Diff {
        source, source_oid, ..
    } = &row.diff.change
    else {
        return None;
    };
    let base = row.base.as_deref().unwrap_or("unknown");
    Some(format!(
        "- candidate {handle}: branch {source} at {source_oid}, forked from base {base}"
    ))
}
