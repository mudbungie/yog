//! **The §3.8 fan's door and aftermath** (bl-77bc) — split from
//! [`start`](super::start) on the family's own seam: a fan is the start spent
//! N times, so this file posts one `Fan(Spread)` and, on its receipt, walks the
//! rebound starts back through `Prompt`'s ordinary door. Coverage-excluded
//! shell glue like its siblings: the spread, the rebinding and the ceiling all
//! live in tested modules; this only wires the composer's pick to them.

use super::{Owes, Seat};
use crate::AppModel;
use crate::boundary::Action;
use crate::shell::ShellState;
use crate::start::Prepared;
use std::path::Path;

/// Post the §3.8 fan (bl-77bc): spend the pending start N times over the
/// focused ball's delivery obligation. The obligation is the seat's own —
/// the same `/fan` context read the line makes — and a seat with none gets the
/// refusal in words rather than a silent single start: the picker offered a
/// fan, so quietly firing something else would be a different gesture.
pub(in crate::shell) fn fan(
    model: &mut AppModel,
    state: &mut ShellState,
    ws: Option<&Path>,
    prepared: &Prepared,
    n: usize,
) {
    let ctx = model.line_context();
    let (Some(project), Some(crate::start::BallSpec::Existing { id, .. })) =
        (ctx.project, ctx.ball)
    else {
        state.slash = Some(
            "nothing to fan over — a fan spends the focused ball's delivery obligation, \
             and this workspace holds none"
                .to_owned(),
        );
        return;
    };
    let action = Action::Fan(crate::fan::Verb::Spread {
        prepared: prepared.clone(),
        obligation: crate::fan::Obligation {
            project,
            ball: Some(id),
        },
        n,
    });
    super::hold(
        model,
        state,
        ws,
        &action,
        Seat::Quiet,
        Owes::Fanned {
            goal: prepared.goal.clone(),
        },
    );
}

/// A landed `Fan(Spread)`: fire each rebound start through `Prompt`'s own door
/// (§3.8 — the spread fires nothing itself, so the ceiling gates every birth
/// exactly as it gates a single start). The **last** prompt is held, so the
/// pane, the focus hand-back and the seed ride its receipt exactly as a single
/// start's do; the ones before it fire quietly with no seed — the composer's
/// prediction was one name, and N candidates cannot all wear it. Returns
/// whether the gesture handed off.
pub(super) fn fanned(
    model: &mut AppModel,
    state: &mut ShellState,
    ws: Option<&Path>,
    candidates: &[Prepared],
    goal: &str,
) -> bool {
    state.start.fan_n = 1;
    let Some((last, rest)) = candidates.split_last() else {
        return false;
    };
    for prepared in rest {
        model.post_act(&Action::Prompt {
            prepared: prepared.clone(),
            goal: goal.to_owned(),
            seed: None,
        });
    }
    super::start::prompt(model, state, ws, last, goal);
    true
}
