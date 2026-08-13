//! The pure half of the work-diff (DESIGN §5.1 #32): which attempts a
//! workspace holds, what each one's two ends are called, and how `git`'s
//! numstat rows read. No IO — every input is already on the snapshot.
//!
//! **The target is derived, never chosen** (VISION §4.10 item 1). balls'
//! own rule, verbatim from its `target` module: *"If the ball close-gates its
//! LIVE parent — it has `parent = X` AND `X` carries the blocker `{this, on:
//! close}` — the target is `X`. Otherwise the target is absent, meaning the
//! integration branch."* Both coordinates are already on [`Ball`] (§5.1 #2),
//! so yog re-derives the same graph arithmetic rather than storing an answer
//! or asking for a verb that would print one.

use crate::projects::balls::Ball;
use std::collections::HashMap;
use std::path::PathBuf;

use super::{Churn, FileChurn};

/// `bl close`'s gated verb token, as the ball's bedrock blocker spells it.
const ON_CLOSE: &str = "close";
/// `git diff --numstat` field count: `<added>\t<removed>\t<path>`.
const NUMSTAT_FIELDS: usize = 3;
/// What numstat writes for a count it cannot state — a binary file's.
const NO_COUNT: &str = "-";

/// One attempt's ends before git has resolved them: the project it lands in,
/// the ball whose claim materialized it, and the ball its work targets (`None`
/// ⇒ the integration branch, which only the repo can name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Plan {
    pub(super) project: PathBuf,
    pub(super) ball_id: String,
    pub(super) target_ball: Option<String>,
}

/// Every attempt the workspace named `name` holds, project-path ordered so two
/// instances answer identically (I9). The binding is the §3.2 claimant
/// equality — the same join the roster reads, asked from the workspace's side.
/// A workspace holding two balls has two attempts, and the surface says both:
/// there is no rule that picks one, so nothing here invents one.
pub(super) fn plans(by_project: &HashMap<PathBuf, Vec<Ball>>, name: &str) -> Vec<Plan> {
    let mut projects: Vec<(&PathBuf, &Vec<Ball>)> = by_project.iter().collect();
    projects.sort_by_key(|(path, _)| *path);
    let mut out = Vec::new();
    for (project, live) in projects {
        for ball in live.iter().filter(|b| b.claimant.as_deref() == Some(name)) {
            out.push(Plan {
                project: project.clone(),
                ball_id: ball.id.clone(),
                target_ball: target_ball(ball, live),
            });
        }
    }
    out
}

/// balls' delivery-target rule (its `target::derive`): the parent ball when
/// this ball close-gates it and that parent is still live; else `None` — the
/// integration branch. A parent whose file is gone gates nothing, which falls
/// out of "still live" rather than needing an arm.
fn target_ball(ball: &Ball, live: &[Ball]) -> Option<String> {
    let parent = ball.parent.as_deref()?;
    let gated = live
        .iter()
        .find(|b| b.id == parent)?
        .blockers
        .iter()
        .any(|b| b.id == ball.id && b.on == ON_CLOSE);
    gated.then(|| parent.to_owned())
}

/// Parse `git diff --numstat` output into per-file churn, in git's own order.
/// A row whose counts are `-` is a binary file — stated as binary, never as
/// zero lines, because a zero would be a lie about a file that changed.
pub(super) fn parse_numstat(stdout: &[u8]) -> Vec<FileChurn> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter_map(parse_row)
        .collect()
}

/// One `<added>\t<removed>\t<path>` row. A rename's composite path
/// (`{old => new}`) rides through verbatim: it is what changed, said the way
/// git says it, and inventing two rows out of one would be yog's guess.
pub(super) fn parse_row(line: &str) -> Option<FileChurn> {
    let mut fields = line.splitn(NUMSTAT_FIELDS, '\t');
    let (added, removed, path) = (fields.next()?, fields.next()?, fields.next()?);
    if path.is_empty() {
        return None;
    }
    let churn = match (added, removed) {
        (NO_COUNT, NO_COUNT) => Churn::Binary,
        _ => Churn::Text {
            added: added.parse().ok()?,
            removed: removed.parse().ok()?,
        },
    };
    Some(FileChurn {
        path: path.to_owned(),
        churn,
    })
}
