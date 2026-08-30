//! Arming, as a `cadence.yaml` entry (VISION §4.3, rung V4 item 2).
//!
//! The loop's whole configuration is one two-space entry per armed workspace
//! under a `fleet:` block, in the file yog's clock already owns (§7.2) — a
//! sibling of `cadence:` and of the monitor's own block, a row rather than a
//! rebuild:
//!
//! ```text
//! fleet:
//!   /home/u/.local/share/yog/world/litany/workspaces/otter:
//!     project: /home/u/dev/yog
//!     cap: 3
//!     lease_min: 30
//! ```
//!
//! The entry key is the workspace path — the same key `ui.json` uses for its
//! §4.1 watermarks and the monitor uses for its own entries, so yog's durables
//! name a workspace one way. **Presence is armed**; absence is the default, and
//! under it no board fact renders, no row is written and nothing is spawned.
//! Severability is deleting the entry.
//!
//! **Two fields are the operator's to choose and yog will not guess either.**
//! An entry with no `project:` or no readable `cap:` is **not armed**: the
//! project is where the loop takes work from and the cap is how much money it
//! may spend at once, and a default for either would be yog's opinion charged
//! to the operator. `lease_min` is different — it is **absent by default and
//! absence means never reap**, because a reap releases a claim and yog must not
//! do that on an opinion. The arm gesture writes the first two; the lease is
//! added by editing this file, like every other tuning knob (§9.5).

use crate::model_pick::grammar::{entry_field, entry_names, remove_entry, set_entry};
use std::path::PathBuf;
use std::time::Duration;

/// The column-0 block key. A sibling of [`cadence`](crate::app::cadence::BLOCK)
/// and of [`monitor`](crate::monitor::arming::BLOCK) in the same file: the
/// clock's settings file carries the clock's policies, so the world gains no
/// further durable.
pub const BLOCK: &str = "fleet";
/// The entry's project field — where the loop takes ready work from.
pub const PROJECT: &str = "project";
/// The entry's cap field — how many balls this workspace may hold at once.
pub const CAP: &str = "cap";
/// The entry's optional lease field, in whole minutes. Absent is *never reap*.
pub const LEASE_MIN: &str = "lease_min";

/// One workspace's fleet entry — the whole armed configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// The project whose ready balls this workspace's loop takes.
    pub project: PathBuf,
    /// The most balls this workspace may hold claimed at once.
    pub cap: usize,
    /// How long a ball's drones may be quiet before the loop releases the
    /// claim. `None` — the default — reaps nothing, ever.
    pub lease: Option<Duration>,
}

/// Every armed workspace key the file declares, in file order. A file with no
/// `fleet:` block arms nothing — absence is a value, not an error.
pub fn armed(text: &str) -> Vec<String> {
    entry_names(text, BLOCK)
}

/// One workspace's entry, or `None` when it is not armed — which a declared
/// entry missing either required field also is (see the module note).
pub fn policy(text: &str, key: &str) -> Option<Policy> {
    let field = |name| entry_field(text, BLOCK, key, name).filter(|v| !v.is_empty());
    Some(Policy {
        project: PathBuf::from(field(PROJECT)?),
        cap: field(CAP)?.parse().ok()?,
        lease: field(LEASE_MIN)
            .and_then(|m| m.parse::<u64>().ok())
            .map(|m| Duration::from_secs(m.saturating_mul(60))),
    })
}

/// Arm `key` on `project` at `cap`, replacing any entry it already had. `None`
/// is the one refusal — an inline `fleet:` key, which cannot be rewritten
/// without this becoming a YAML parser.
///
/// A re-arm rewrites exactly the two fields it states, which is also how the
/// operator's hand-added `lease_min` is *not* preserved: the gesture states the
/// whole entry it means, and an instruction is never half an instruction.
pub fn arm(text: &str, key: &str, project: &str, cap: usize) -> Option<String> {
    set_entry(
        text,
        BLOCK,
        key,
        &[(PROJECT, project.to_owned()), (CAP, cap.to_string())],
    )
}

/// Disarm `key`: the entry and every line it owns, gone. A key that was never
/// armed yields the same file — deleting what is already deleted is the same
/// world, not an error.
pub fn disarm(text: &str, key: &str) -> String {
    remove_entry(text, BLOCK, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARMED: &str = "cadence:\n  watcher:\n    debounce_ms: 100\nfleet:\n  /ws/a:\n    project: /dev/yog\n    cap: 3\n";

    #[test]
    fn an_absent_block_arms_nothing() {
        assert!(armed("cadence:\n  watcher:\n    debounce_ms: 100\n").is_empty());
        assert_eq!(policy("", "/ws/a"), None);
    }

    #[test]
    fn a_declared_entry_is_the_policy() {
        assert_eq!(armed(ARMED), vec!["/ws/a".to_owned()]);
        assert_eq!(
            policy(ARMED, "/ws/a"),
            Some(Policy {
                project: PathBuf::from("/dev/yog"),
                cap: 3,
                lease: None,
            })
        );
        assert_eq!(policy(ARMED, "/ws/b"), None, "a sibling key is not armed");
    }

    #[test]
    fn a_missing_or_unreadable_required_field_is_not_armed() {
        assert_eq!(
            policy("fleet:\n  /ws/a:\n    cap: 3\n", "/ws/a"),
            None,
            "no project, no fleet"
        );
        assert_eq!(
            policy("fleet:\n  /ws/a:\n    project: /dev/yog\n", "/ws/a"),
            None,
            "no cap, no fleet"
        );
        assert_eq!(
            policy(
                "fleet:\n  /ws/a:\n    project: /dev/yog\n    cap: lots\n",
                "/ws/a"
            ),
            None,
            "a cap yog cannot read is not a cap it may guess"
        );
    }

    #[test]
    fn the_lease_is_optional_and_reads_as_minutes() {
        let text = "fleet:\n  /ws/a:\n    project: /dev/yog\n    cap: 1\n    lease_min: 30\n";
        assert_eq!(
            policy(text, "/ws/a").expect("armed").lease,
            Some(Duration::from_mins(30))
        );
        let broken = "fleet:\n  /ws/a:\n    project: /dev/yog\n    cap: 1\n    lease_min: soon\n";
        assert_eq!(
            policy(broken, "/ws/a").expect("armed").lease,
            None,
            "an unreadable lease reaps nothing rather than reaping on a guess"
        );
    }

    #[test]
    fn arming_creates_the_block_then_replaces_the_entry() {
        let base = "cadence:\n  watcher:\n    debounce_ms: 100\n";
        let once = arm(base, "/ws/a", "/dev/yog", 2).expect("block absent is creatable");
        assert_eq!(policy(&once, "/ws/a").expect("armed").cap, 2);
        assert!(
            once.contains("debounce_ms: 100"),
            "the clock's own entry survives byte-for-byte"
        );
        let again = arm(&once, "/ws/a", "/dev/yog", 5).expect("armable");
        assert_eq!(policy(&again, "/ws/a").expect("armed").cap, 5);
        assert_eq!(armed(&again).len(), 1, "re-arming replaces, never appends");
    }

    #[test]
    fn arming_a_second_workspace_leaves_the_first() {
        let two = arm(ARMED, "/ws/b", "/dev/litany", 1).expect("armable");
        assert_eq!(policy(&two, "/ws/a").expect("armed").cap, 3);
        assert_eq!(
            policy(&two, "/ws/b").expect("armed").project,
            PathBuf::from("/dev/litany")
        );
    }

    #[test]
    fn an_inline_block_key_refuses() {
        assert_eq!(arm("fleet: {}\n", "/ws/a", "/dev/yog", 1), None);
    }

    #[test]
    fn disarming_removes_the_entry_and_is_idempotent() {
        let off = disarm(ARMED, "/ws/a");
        assert_eq!(policy(&off, "/ws/a"), None);
        assert!(off.contains("debounce_ms: 100"), "the clock is untouched");
        assert_eq!(disarm(&off, "/ws/a"), off, "disarming twice is one world");
    }
}
