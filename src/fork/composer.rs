//! The fork composer's state (VISION V2.1/V2.2): one goal, and the N attempts
//! it will be tried as.
//!
//! *"A goal composer seeded empty … **×N** fires the same fork N times with
//! per-variant overrides."* The ×N control is [`Composer::resize`] and it is
//! the whole of it: a cohort is a `Vec<Attempt>` whose length is N, so one
//! attempt is that vector with one element and the fan is that vector with
//! more. There is no branch on N anywhere in this module or below it — which
//! is the *"one attempt and a parallel cohort use one path"* requirement made
//! mechanical rather than promised.
//!
//! Growing clones the last attempt, because a candidate is nearly always the
//! previous one with one control moved; the operator then edits the difference
//! instead of restating the whole. Shrinking truncates. The floor is one: a
//! composer with no attempt is a composer that cannot fire, and "none" is
//! spelled by not firing.
//!
//! **RAM, never durable** (§5.3): the composer is viewport ephemera held
//! beside the notch selection it belongs to, and it dies with the pin.

use super::{Attempt, Choices};

/// The fork composer for one pinned notch.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Composer {
    /// The goal, seeded empty (VISION V2.1) and reaching the model verbatim.
    pub goal: String,
    /// The candidates, in the order they will fire. Never empty once
    /// [`seeded`](Composer::seeded); [`resize`](Composer::resize) floors at 1.
    pub attempts: Vec<Attempt>,
}

impl Composer {
    /// A composer seeded from what the seat offers: the goal empty, one
    /// attempt forking from the first point that declares a role, wearing that
    /// point's first role. The seed is a *reading* of the workspace, never a
    /// yog default — which is why a workspace whose config yog cannot reach
    /// seeds nothing and [`Choices::fireable`] has already declined to paint.
    pub fn seeded(choices: &Choices) -> Self {
        let attempt = choices
            .points
            .iter()
            .find(|p| !p.roles.is_empty())
            .and_then(|p| {
                p.roles.first().map(|r| Attempt {
                    from: p.refspec.clone(),
                    role: r.role.clone(),
                    skills: Vec::new(),
                })
            })
            .unwrap_or_default();
        Self {
            goal: String::new(),
            attempts: vec![attempt],
        }
    }

    /// The ×N control: make the cohort `n` candidates wide. Growing clones the
    /// last attempt; shrinking drops from the end; `0` floors at one, because
    /// the composer's own existence is the operator asking for at least one.
    pub fn resize(&mut self, n: usize) {
        let want = n.max(1);
        while self.attempts.len() > want {
            self.attempts.pop();
        }
        while self.attempts.len() < want {
            let next = self.attempts.last().cloned().unwrap_or_default();
            self.attempts.push(next);
        }
    }

    /// Toggle one skill on one attempt — the per-attempt skills control. An
    /// index the composer does not have is a no-op, so a stale click from a
    /// frame that has since shrunk cannot panic or edit a neighbour.
    pub fn toggle_skill(&mut self, index: usize, skill: &str) {
        let Some(attempt) = self.attempts.get_mut(index) else {
            return;
        };
        match attempt.skills.iter().position(|s| s == skill) {
            Some(at) => {
                attempt.skills.remove(at);
            }
            None => attempt.skills.push(skill.to_owned()),
        }
    }

    /// Can this fire? A goal with text, and every attempt naming both a fork
    /// point and a role. Checked here rather than at the button, so the
    /// headless seat and the widget refuse on the same rule.
    pub fn ready(&self) -> bool {
        !self.goal.trim().is_empty()
            && !self.attempts.is_empty()
            && self
                .attempts
                .iter()
                .all(|a| !a.from.is_empty() && !a.role.is_empty())
    }
}
