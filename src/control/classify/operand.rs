//! The two intrinsic rows judged **against the writable root at consult time**
//! (VISION §4.11 item 1) — `cd` and `apply_patch`.
//!
//! Their own file beside [`super::intrinsic`], on the seam that module's doc
//! already draws: every other row is a fixed class the name alone decides,
//! and these two are the ones that have to ask where the invocation points.
//! The path algebra they ask it through is [`Root`](super::Root)'s, which is
//! derived from facts yog owns and never from one the agent controls
//! (`super::super::root`).

use super::{Classified, Effect, Root};

/// A `cd`: [`Read`](Effect::Read) inside the writable root (moving is not
/// writing), [`OpenWorld`](Effect::OpenWorld) out of it — a move out of the root
/// is how every later relative operand leaves it.
pub(super) fn move_to(path: &str, root: &Root) -> Classified {
    let dest = root.resolve(path);
    if root.holds(&dest) {
        Classified::new(
            Effect::Read,
            format!("moves to {} inside the writable root", dest.display()),
        )
    } else {
        Classified::new(
            Effect::OpenWorld,
            format!("moves to {}, outside the writable root", dest.display()),
        )
    }
}

/// An `apply_patch`: a target write when every file the envelope names resolves
/// inside the writable root, open-world otherwise. An envelope naming no file
/// at all patches nothing and reads as a write of nothing.
pub(super) fn patch(envelope: &str, root: &Root) -> Classified {
    let paths = patch_paths(envelope);
    if root.holds_all(&paths) {
        Classified::new(
            Effect::TargetWrite,
            "patches files inside the writable root",
        )
    } else {
        Classified::new(
            Effect::OpenWorld,
            format!(
                "patches {}, outside the writable root",
                outside(&paths, root)
            ),
        )
    }
}

/// The first operand of `paths` that falls outside the root, for the reason
/// line. Total: the caller only asks when one exists, and an empty answer would
/// still read as a sentence.
pub(super) fn outside(paths: &[String], root: &Root) -> String {
    paths
        .iter()
        .find(|p| !root.holds(&root.resolve(p)))
        .cloned()
        .unwrap_or_default()
}

/// Every path an `apply_patch` envelope names — its `Add File` / `Delete File` /
/// `Update File` sections and any `Move to` destination.
fn patch_paths(envelope: &str) -> Vec<String> {
    const MARKERS: [&str; 4] = [
        "*** Add File: ",
        "*** Delete File: ",
        "*** Update File: ",
        "*** Move to: ",
    ];
    envelope
        .lines()
        .filter_map(|line| {
            MARKERS
                .iter()
                .find_map(|m| line.trim_end().strip_prefix(m))
                .map(|p| p.trim().to_owned())
        })
        .filter(|p| !p.is_empty())
        .collect()
}
