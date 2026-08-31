//! The "grouped by ball" organizing view (DESIGN §3.5, §11, §15 Z9).
//!
//! The conversation list has two orderings: **flat by recency** (the default —
//! [`super::build`]'s sorted rows straight) and **grouped by ball**, this module.
//! Grouping is a pure, stable partition of the already-sorted rows: each start-flow
//! ball (§3.3 goal stamp) heads a group with its conversations beneath it, and
//! conversations with no ball fall to a trailing group. The toggle itself is
//! viewport ephemera (§13.1, RAM) — the shell picks the ordering; all logic is here.

use super::{ConvBall, ConvRow};

/// One group in the grouped-by-ball view: a ball and the conversations stamped
/// with it. `ball` is `None` for the single trailing group of conversations with
/// no start-flow ball (§3.2: they carry no per-conversation attribution).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvGroup {
    pub ball: Option<ConvBall>,
    pub convs: Vec<ConvRow>,
}

/// Partition sorted conversation rows into per-ball groups (§11 grouped view).
/// Group order is first-appearance in the input, so the sort the rows already
/// carry (§11 recency, bl-cad5) decides which ball leads; within a group the
/// rows keep that order. Nothing here ranks — inheriting [`super::build`]'s
/// order is the whole mechanism, so a change to that order needs no change here. Unassociated conversations collect into one
/// trailing `None` group (emitted only when non-empty), so "unassociated last"
/// is the general tail, not a special case. Stable and total over any input.
pub fn group_by_ball(rows: Vec<ConvRow>) -> Vec<ConvGroup> {
    let mut groups: Vec<ConvGroup> = Vec::new();
    let mut unassociated: Vec<ConvRow> = Vec::new();
    for row in rows {
        let Some(ball) = row.ball.clone() else {
            unassociated.push(row);
            continue;
        };
        match groups
            .iter_mut()
            .find(|g| group_id(g) == Some(ball.id.as_str()))
        {
            Some(g) => g.convs.push(row),
            None => groups.push(ConvGroup {
                ball: Some(ball),
                convs: vec![row],
            }),
        }
    }
    if !unassociated.is_empty() {
        groups.push(ConvGroup {
            ball: None,
            convs: unassociated,
        });
    }
    groups
}

/// A group's ball id, if it heads one (never the trailing `None` group).
fn group_id(group: &ConvGroup) -> Option<&str> {
    group.ball.as_ref().map(|b| b.id.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git_tree::AgentState;

    fn ball(id: &str) -> ConvBall {
        ConvBall {
            id: id.to_owned(),
            state: None,
            title: None,
            badge: None,
        }
    }

    fn row(root: &str, ball: Option<&str>) -> ConvRow {
        ConvRow {
            root_id: root.to_owned(),
            state: AgentState::Quiescent,
            uncertain: false,
            preview: String::new(),
            age_secs: 0,
            flight: None,
            attention: 0,
            members: 1,
            depth: 0,
            direct: 0,
            stoppable: false,
            stop_children: false,
            ball: ball.map(self::ball),
            name: None,
            name_display_only: false,
            verdict: None,
            tone: crate::nav::convs::Tone::Plain,
        }
    }

    #[test]
    fn balls_head_groups_in_first_appearance_order_conversations_beneath() {
        // Two conversations on b1, one on b2; b1 appears first in the sorted input,
        // so its group leads, and both its conversations sit under it in order.
        let rows = vec![
            row("r1", Some("b1")),
            row("r2", Some("b2")),
            row("r3", Some("b1")),
        ];
        let groups = group_by_ball(rows);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].ball.as_ref().map(|b| b.id.as_str()), Some("b1"));
        let ids: Vec<&str> = groups[0].convs.iter().map(|c| c.root_id.as_str()).collect();
        assert_eq!(ids, ["r1", "r3"], "same-ball conversations, input order");
        assert_eq!(groups[1].ball.as_ref().map(|b| b.id.as_str()), Some("b2"));
    }

    #[test]
    fn unassociated_conversations_collect_into_one_trailing_group() {
        let rows = vec![
            row("r1", Some("b1")),
            row("bare1", None),
            row("bare2", None),
        ];
        let groups = group_by_ball(rows);
        assert_eq!(groups.len(), 2);
        assert!(groups[0].ball.is_some());
        // The trailing group carries no ball and gathers every bare conversation.
        assert!(groups[1].ball.is_none(), "unassociated group is last");
        let ids: Vec<&str> = groups[1].convs.iter().map(|c| c.root_id.as_str()).collect();
        assert_eq!(ids, ["bare1", "bare2"]);
    }

    #[test]
    fn all_unassociated_is_the_single_none_group_and_empty_input_is_empty() {
        let groups = group_by_ball(vec![row("a", None), row("b", None)]);
        assert_eq!(groups.len(), 1);
        assert!(groups[0].ball.is_none());
        assert!(group_by_ball(vec![]).is_empty(), "no rows, no groups");
    }
}
