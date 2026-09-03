//! The Counterfactualist's headless tests — STORIES §S11's fire half.
//!
//! Split by what each half asserts: [`argv`] the attempt's own argv and the
//! world pool it pins from; [`choices`] the fire-time policy read off a real
//! workspace repo. The composer's own halves left with it (bl-7cc8).

mod argv;
mod choices;

use crate::fork::Attempt;

/// One attempt, spelled out.
pub(super) fn attempt(from: &str, role: &str, skills: &[&str]) -> Attempt {
    Attempt {
        from: from.to_owned(),
        role: role.to_owned(),
        skills: skills.iter().map(|s| (*s).to_owned()).collect(),
    }
}
