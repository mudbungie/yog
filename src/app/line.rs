//! [`AppModel`]'s half of the §8.5 **line**: the seat's own context, and the
//! query door beside the action one.
//!
//! A slash command elides its target and takes it from the seat
//! ([`crate::boundary::line::Context`]). At the window that seat *is* this
//! model's focus, so the context is a **derivation over the published
//! snapshot** — the same facts the click-glue resolves before it constructs a
//! variant, read once, in one place, so `/close` and the Close button can never
//! aim at two different balls. The one fact the model does not hold is the
//! pending [`Prepared`](crate::start::Prepared): that is start-flow RAM (§5.3),
//! and the shell folds it in as it types. The query half needs nothing new:
//! [`AppModel::answer`] is already the frame-side chokepoint a `/balls` lands
//! on.

use super::AppModel;
use crate::boundary::line::Context;
use crate::projects::join::owner_name;

impl AppModel {
    /// The line context this instance's focus supplies (§8.5).
    ///
    /// `name` is the §3.2 stamp a `bl` verb carries: **the focused ball's
    /// claimant** when a ball is focused — the ownership line, exactly as
    /// [`ball_bar`](crate::shell) stamps it — and otherwise the focused
    /// workspace's own name, which is what a ball being *acquired* (`/create`)
    /// is stamped with. One field, because it is one fact: the workspace name
    /// this seat's next `bl` verb acts as.
    pub fn line_context(&self) -> Context {
        let join = self.focused_join().cloned();
        Context {
            // The seam (REMOTE §8, bl-f5f6): a seat's *selection* is a path,
            // a gesture's *address* is a name — this is where the one mapping
            // is read forwards, and the chokepoint reads it back.
            workspace: self.focused_workspace().map(|p| self.snap.ws_name(p)),
            agent: self.focused_agent().map(|a| a.agent_id.clone()),
            project: join
                .as_ref()
                .map(|row| self.snap.project_name(&row.project)),
            name: join
                .as_ref()
                .map(owner_name)
                .or_else(|| self.focused_workspace().map(crate::naming::leaf)),
            ball: join.as_ref().and_then(|row| self.ball_spec(row)),
            // Start-flow RAM, not a derivation (§5.3): the shell adds it.
            prepared: None,
        }
    }
}
