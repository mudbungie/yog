//! **What the queue region is handed**, split off [`super`] at §12's budget on
//! the seam between the fold and the paint: `super` wires widgets, and this is
//! the one derivation behind them — the target's undelivered deposits with this
//! window's own §3.4 echo folded on, and the past turns ↑ pages back through.
//!
//! It is a fold and never a read (§11: no frame-time IO). Both halves are
//! selections out of standing questions the §11 Inbox tab and the chat pane
//! already ask, so the seats cost one ask apiece rather than two (REMOTE §9.7,
//! bl-b4b5, bl-13f9).

use crate::AppModel;

/// **What the §11 queue region is handed**: the target's undelivered deposits
/// (§5.1 #11) with this window's own §3.4 echo folded on, and the operator's
/// past turns ↑ pages back through (bl-f908). One function because the second
/// is derived from the first — the recall walks the pending listing ahead of
/// the delivered transcript — and because both are the same two standing
/// questions the §11 Inbox tab and the chat pane already ask, so the seats are
/// one ask apiece rather than two (REMOTE §9.7, bl-b4b5, bl-13f9).
///
/// `branchless` is the §3.4 start window: a conversation with no branch has no
/// address, so neither question is *declared* for it — each would refuse and
/// pay the rung-3 disk fallback to do it, every ask period, for the whole
/// window (bl-56c6). What the queue paints there is the echo, which is a fold.
///
/// Every empty answer here is the honest one and not a case: no target, an
/// unanswered question and a conversation with no mail all read as nothing
/// queued and no past turns, exactly as they always did.
pub(super) fn queued(
    model: &mut AppModel,
    ws: &std::path::Path,
    target: Option<&str>,
    branchless: bool,
) -> (Vec<crate::inboxview::InboxEntry>, Vec<String>) {
    let Some(agent) = target else {
        return (Vec::new(), Vec::new());
    };
    let landed = if branchless {
        Vec::new()
    } else {
        crate::shell::inspector::inbox(model, ws, agent)
            .value
            .unwrap_or_default()
    };
    // This window's own §3.4 echo folded on (`AppModel::echoed_pending`) — the
    // third projection of one optimism, for bl-44e9's reason: a seat's optimism
    // reaches whatever that seat actually reads, and what this one reads is now
    // an answer.
    let pending = model.echoed_pending(agent, landed);
    if branchless {
        return (pending, Vec::new());
    }
    let tx = crate::shell::inspector::transcript(model, ws, agent)
        .value
        .unwrap_or_default();
    let prompts = crate::composer::prompts(&pending, &tx);
    (pending, prompts)
}
