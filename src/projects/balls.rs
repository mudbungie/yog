//! Balls-per-project projection and the derived status ladder (DESIGN §5.1
//! #2–#4, §15 Y14). The §3.5 join-state table is [`super::join`]; the `bl`
//! effect that feeds both is [`super::runner`].
//!
//! Two pure layers:
//! - **Project** — a [`Ball`] is built from balls' own typed store record,
//!   [`balls::reads::Entry`] (id + `task::Task`), in-process (§16.7 W8). The
//!   forgiving `serde_json` reader ([`parse_list`]) survives for the ONE read
//!   still served by a subprocess: the history-reconstructed **closed** listing,
//!   whose dead-ball walk is not on balls' promoted read surface (see
//!   [`super::runner::BlStore::closed`]).
//! - **Ladder** ([`ladder`]) — claimant ⇒ claimed; else a *live* claim-blocker ⇒
//!   blocked; else ready (§5.1 #3). "Closed" is absence from the live set. The
//!   rungs are balls' own [`Status`], re-exported: one vocabulary, one home.

use balls::reads::Entry;
use serde_json::Value;
use std::collections::HashSet;

/// The derived status of a *live* ball — balls' own §3 ladder rungs
/// ([`balls::task::Status`]), re-exported so yog names the rungs balls names
/// them. Closed is absence and has no variant: it enters the join as
/// `JoinStatus::Closed`, never via the ladder.
pub use balls::task::Status;

/// A claim/close gate edge from a ball's bedrock `blockers`; `on` is the gated
/// verb (`"claim"` for the ladder's claim-blockers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocker {
    pub id: String,
    pub on: String,
}

/// A ball as projected from `bl … --json` (§5.1 #2). Only the fields yog derives
/// status and detail from are named; every other bedrock key round-trips unread.
/// `claimant`/`parent`/`root_commit` normalize an empty string to `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ball {
    pub id: String,
    pub title: String,
    pub body: String,
    pub claimant: Option<String>,
    pub blockers: Vec<Blocker>,
    pub parent: Option<String>,
    pub priority: i64,
    pub tags: Vec<String>,
    pub created: Option<i64>,
    pub updated: Option<i64>,
    pub root_commit: Option<String>,
}

/// A present, non-empty string field, else `None`.
fn opt_str(obj: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    obj.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

/// The `blockers` array as `{id, on}` pairs; malformed entries skipped.
fn blockers_of(obj: &serde_json::Map<String, Value>) -> Vec<Blocker> {
    let Some(arr) = obj.get("blockers").and_then(Value::as_array) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|v| {
            let b = v.as_object()?;
            Some(Blocker {
                id: b.get("id")?.as_str()?.to_owned(),
                on: b.get("on")?.as_str()?.to_owned(),
            })
        })
        .collect()
}

/// The `tags` array as strings (non-string entries skipped).
fn tags_of(obj: &serde_json::Map<String, Value>) -> Vec<String> {
    let Some(arr) = obj.get("tags").and_then(Value::as_array) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

/// A string field with an empty-string default (`title`/`body`).
fn str_or_empty(obj: &serde_json::Map<String, Value>, key: &str) -> String {
    obj.get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}

/// Project one bedrock object to a [`Ball`], or `None` when it carries no `id`
/// (a ball with no identity is not a ball — the one rejection; all else defaults).
fn ball_of(v: &Value) -> Option<Ball> {
    let obj = v.as_object()?;
    Some(Ball {
        id: opt_str(obj, "id")?,
        title: str_or_empty(obj, "title"),
        body: str_or_empty(obj, "body"),
        claimant: opt_str(obj, "claimant"),
        blockers: blockers_of(obj),
        parent: opt_str(obj, "parent"),
        priority: obj.get("priority").and_then(Value::as_i64).unwrap_or(0),
        tags: tags_of(obj),
        created: obj.get("created").and_then(Value::as_i64),
        updated: obj.get("updated").and_then(Value::as_i64),
        root_commit: opt_str(obj, "root_commit"),
    })
}

/// Forgiving parse of `bl list -s closed --json` (a bedrock array): well-formed
/// balls only; malformed entries and a non-array document alike drop to empty.
/// The last JSON reader in yog — the closed listing's dead-ball walk is the one
/// read balls does not expose to a linked consumer (§16.7 W8).
pub fn parse_list(json: &str) -> Vec<Ball> {
    match serde_json::from_str::<Value>(json) {
        Ok(Value::Array(items)) => items.iter().filter_map(ball_of).collect(),
        _ => Vec::new(),
    }
}

/// Project balls' own typed store record — one [`Entry`] off a
/// [`balls::reads::Catalog`] — to a [`Ball`], in-process (§16.7 W8). The id is
/// the entry's (the filename identity, balls §3); every other field is stored
/// frontmatter read straight off `task::Task`, with the same empty-string ⇒
/// `None` normalization the JSON reader applies. Nothing derived crosses here:
/// status is [`ladder`]'s, as it is balls' `Task::status`'s.
impl From<&Entry> for Ball {
    fn from(entry: &Entry) -> Self {
        let task = &entry.task;
        Ball {
            id: entry.id.clone(),
            title: task.title.clone(),
            body: task.body.clone(),
            claimant: non_empty(task.claimant.clone()),
            blockers: task
                .blockers
                .iter()
                .map(|b| Blocker {
                    id: b.id.clone(),
                    on: b.on.token().to_owned(),
                })
                .collect(),
            parent: non_empty(task.parent.clone()),
            priority: task.priority.unwrap_or(0),
            tags: task.tags.clone(),
            created: Some(task.created),
            updated: Some(task.updated),
            root_commit: non_empty(task.root_commit.clone()),
        }
    }
}

/// A stored optional string with the empty string normalized away — the same
/// rule [`opt_str`] applies to the JSON shape, so both projections agree.
fn non_empty(v: Option<String>) -> Option<String> {
    v.filter(|s| !s.is_empty())
}

/// The §5.1 #3 ladder over the live set: claimant ⇒ claimed; else a claim-blocker
/// whose target is still present in `live` ⇒ blocked; else ready. A claim-blocker
/// onto a closed (absent) target is resolved and does not block. This is balls'
/// own `Task::status` ladder with its §10 resolver spelled `!live.contains(id)`
/// — the live set IS the resolver (balls: a resolved ball's file is gone).
pub fn ladder(ball: &Ball, live: &HashSet<&str>) -> Status {
    if ball.claimant.is_some() {
        Status::Claimed
    } else if ball
        .blockers
        .iter()
        .any(|b| b.on == "claim" && live.contains(b.id.as_str()))
    {
        Status::Blocked
    } else {
        Status::Ready
    }
}

#[cfg(test)]
mod tests;
