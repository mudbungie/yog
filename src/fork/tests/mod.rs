//! The Counterfactualist's headless tests — STORIES §S11's fire half.
//!
//! Split by what each half asserts: [`argv`] the attempt's own argv and the
//! world pool it pins from; [`choices`] the fire-time policy read off a real
//! workspace repo; [`composer`] the ×N and the readiness rule; [`paint`] the
//! composer on screen.

mod argv;
mod choices;
mod composer;
mod paint;

use crate::fork::{Attempt, Choices, ForkPoint};
use crate::model_pick::grammar::RoleModel;

/// One attempt, spelled out.
pub(super) fn attempt(from: &str, role: &str, skills: &[&str]) -> Attempt {
    Attempt {
        from: from.to_owned(),
        role: role.to_owned(),
        skills: skills.iter().map(|s| (*s).to_owned()).collect(),
    }
}

/// A role as a config declares it: the name, and the model it names.
pub(super) fn role(name: &str, model: &str) -> RoleModel {
    RoleModel {
        role: name.to_owned(),
        provider: "anthropic".to_owned(),
        model: model.to_owned(),
    }
}

/// Choices with two fork points — `here` carrying two roles, one config branch
/// carrying one — and a two-skill pool.
pub(super) fn choices() -> Choices {
    Choices {
        points: vec![
            ForkPoint {
                label: "here".to_owned(),
                refspec: "aaaa1111".to_owned(),
                roles: vec![role("worker", "claude-sonnet-5"), role("scribe", "opus")],
            },
            ForkPoint {
                label: "strict".to_owned(),
                refspec: "config/strict".to_owned(),
                roles: vec![role("worker", "claude-haiku-4-5")],
            },
        ],
        skills: vec!["bash".to_owned(), "read_file".to_owned()],
    }
}
