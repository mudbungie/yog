//! The **cohort** — V2's fan, derived (VISION V2.3, §5.1 #32).
//!
//! *"A fan renders as a group: siblings-of-one-ref, each with its state badge,
//! terminal response preview, and usage figure side by side — anchored at the
//! birth notch (one card, N columns)."* V1.7 reserved that anchor; this is it.
//!
//! **Membership is a grouping, not a record.** A cohort is the children that
//! hang on one notch — `provenance_notch`, which V1's [`ChildCard`] already
//! computes from the child's own first commit. Nothing is stored, nothing is
//! registered, and no gesture declares a group: firing the fork twice from one
//! mark *is* firing a cohort, and firing it once *is* a cohort of one. That is
//! why [`cohorts`] is total and has no arm for N — the fan and the single
//! attempt come out of the same fold, differing only in `members.len()`.
//!
//! **Common ancestry is the same fact said once.** Each member already wears
//! its fork label (`from here` / `from config/<name>` / `from <Name>@<oid>`),
//! itself derived from the shared commit prefix. When every member wears the
//! same one, the cohort states it as [`Cohort::common`] and the columns stop
//! repeating it; when they differ — an operator who tried the same question
//! from two different config branches at one mark — there is no common
//! ancestry to state, so the columns say their own. Absence is a value here,
//! not a special case.

use super::{ChildCard, Rail};

/// One cohort: the children born at one notch, and the ancestry they share.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cohort {
    /// The birth notch — the anchor the group renders at (VISION V1.7).
    pub notch: usize,
    /// The fork label every member wears, when they all wear the same one:
    /// the cohort's common ancestry, stated once. `None` when the members
    /// forked off different refs and each must say its own.
    pub common: Option<String>,
    /// The candidates, in the order the rail derived them. Never empty — a
    /// cohort exists because a child does.
    pub members: Vec<ChildCard>,
}

impl Cohort {
    /// Is this the ×N case? Read only by the render, to decide whether the
    /// group wears a header — never to pick a different code path.
    pub fn fanned(&self) -> bool {
        self.members.len() > 1
    }
}

/// Group a rail's cards into cohorts, in notch order. Every card lands in
/// exactly one cohort, so this is a partition of [`Rail::cards`] and never a
/// second copy of the membership fact: the cards are the truth and this is
/// their ordering by birth.
pub fn cohorts(rail: &Rail) -> Vec<Cohort> {
    let mut out: Vec<Cohort> = Vec::new();
    for card in &rail.cards {
        match out.iter_mut().find(|c| c.notch == card.provenance_notch) {
            Some(cohort) => cohort.members.push(card.clone()),
            None => out.push(Cohort {
                notch: card.provenance_notch,
                common: None,
                members: vec![card.clone()],
            }),
        }
    }
    out.sort_by_key(|c| c.notch);
    for cohort in &mut out {
        cohort.common = common_fork(&cohort.members);
    }
    out
}

/// The fork label shared by every member, or `None` when they disagree. A
/// cohort of one shares its own label with itself, which is the general path
/// with one input rather than an arm: the render then has one column and one
/// header line, exactly V1's card.
fn common_fork(members: &[ChildCard]) -> Option<String> {
    let first = members.first()?;
    members
        .iter()
        .all(|m| m.fork == first.fork)
        .then(|| first.fork.clone())
}
