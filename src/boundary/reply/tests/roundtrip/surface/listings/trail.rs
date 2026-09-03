//! The §4.2 trail's corpus fixture (bl-4d81): one trail whose rows spell **all
//! five** [`Standing`](crate::opslog::Standing) words, plus both `failed`
//! answers and three readings of `exit_label`.
//!
//! Its own file rather than six inline literals in [`super`], which is already
//! at §12's pre-split band — and its own seam: every other listing here is a
//! list of independent rows, while these rows only mean anything **together**.
//! `standings` is the producer, exactly as it is in the answer; a hand-built
//! standing beside a hand-built row would be a second authority for the fold
//! the fixture exists to pin.

use crate::opslog::{DETACHED_EXIT, DRIFT_EXIT, OpRow, OpView, Origin, standings};

/// One durable line. `cwd` is shared, because §6's retirement key is
/// `(cwd, verb)` and every case here turns on the verb. `stderr` is passed
/// rather than inferred from the exit: on a `-2` line it is the whole question
/// — a folded sink means the driver died, and an empty one means it handed off
/// (§4.2, bl-b95e).
fn line(ts: &str, argv: &str, exit: i32, stderr: &str, origin: Origin) -> OpRow {
    OpRow {
        ts: ts.to_owned(),
        argv: argv.to_owned(),
        cwd: "/p".to_owned(),
        exit,
        stdout: String::new(),
        stderr: stderr.to_owned(),
        origin,
    }
}

/// The trail, oldest first: a failure a later clean run retired, that clean
/// run, a failure the ack line below covers, the ack itself, a live wound after
/// it, a handoff nobody has observed, and a §7.2 drift — which is an alarm
/// about the watcher and therefore never a failure.
pub(super) fn ops() -> Vec<OpView> {
    standings(&[
        line("1700", "bl close x", 1, "gate", Origin::Balls),
        line("1701", "bl close x", 0, "", Origin::Balls),
        line("1702", "bz login", 2, "no credential", Origin::World),
        line("1703", "yog-step ack-failures", 0, "", Origin::World),
        line(
            "1704",
            "litany prime",
            3,
            "models.yaml missing",
            Origin::Conversation,
        ),
        line(
            "1705",
            "litany prompt c-1",
            DETACHED_EXIT,
            "",
            Origin::Conversation,
        ),
        line(
            "1706",
            "yog-drift unannounced",
            DRIFT_EXIT,
            "/root",
            Origin::World,
        ),
    ])
}
