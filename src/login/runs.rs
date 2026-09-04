//! **Where a sign-in lives while it runs** (REMOTE §8.3, DESIGN §8.3 as
//! amended by bl-61bf; bl-c285): the engine's map of live `bz --login`
//! children, **one per workspace × provider**.
//!
//! The flow is a boundary act now, not a pane's own spawn, and that moves two
//! facts here. The child runs where the *wall* is, so the credential lands in
//! the sphere whose agents read it (§16.2) whichever box the seat is on; and
//! nothing waits for it — the intake is one thread for the whole world (REMOTE
//! §3), so [`Runs::start`] answers the run's standing at once and every later
//! look is a re-read of that same standing.
//!
//! **The flow is read here, once** (§8.3 rule 1 as amended by bl-61bf; bl-7c9f).
//! Which flow a row is fired in is the row's own `device` column, and this is
//! the one place that asks: the wall lens the spawn already folds through is
//! also what `bz --list-providers` answers under, so the read is the same lens
//! and the branch is one fact at one seam ([`flow_of`]). No surface carries a
//! selector and nothing downstream re-derives it.
//!
//! **A second `Login` on a live pair terminates and replaces it.** The
//! operator's own restart is the cancel, so there is no cancel verb; the
//! termination is not best-effort housekeeping but a precondition — an
//! abandoned loopback flow still holds the row's redirect port, and the
//! replacement would lose the bind. So the old run is dropped *under the lock,
//! before the new child is spawned* ([`Stream`](crate::cli_outbound::Stream)'s
//! own drop is the SIGTERM-then-SIGKILL, confined to `cli_outbound::sys` —
//! AGENTS rule 3), and the intake pays that grace rather than the next
//! sign-in paying a lost port.
//!
//! **A reader thread per run, and lanes that only read.** Nothing else polls
//! the child: two seats may hold lanes on one run and a run with no lane at all
//! still has to settle, because settling is what writes the one `ops.jsonl`
//! outcome row (§4.2, `LoginRun::finalize`'s one-source rule). So the thread
//! owns the *cadence* and the map owns the *run*, and a lane reads the buffer.
//! The thread retires the moment its run settles or stops being its run, which
//! is what a replacement does to it — the serial is that test, and it needs no
//! second flag for anyone to forget to set.
//!
//! **A settled run stays until it is swept**, so re-asking replays: a dropped
//! lane, a re-attached seat and a finished sign-in are one case. The bound is
//! the §5.3 mailbox's own — an hour — swept at the one moment the map can
//! grow, exactly as `registry::mailbox` sweeps its slots.
//!
//! The lock itself is the crate's chokepoint's (AGENTS rule 7,
//! [`state::LoginCell`](crate::state::LoginCell)); what lives here is the map
//! it guards and every rule about what may be in it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{Flow, LoginRun, LoginView};
use crate::cli_outbound::{Binary, Cli};
use crate::config_edit::brazen::{BzRunner, ProviderRow, RealBzRunner};
use crate::state::{LoginCell, lock_logins as lock_runs};

/// How often a reader thread drains its child. The §7.2 follower's own period,
/// which is what "live" means in a number — and the lane's tick, so a line
/// reaches a seat within one look of arriving rather than two.
const READ_TICK: Duration = Duration::from_millis(16);

/// How long a run survives after it began: an hour, the §5.3 mailbox bound. It
/// is the *whole* run's age and not its idle time, because a browser flow the
/// operator walked away from is exactly the thing being bounded.
const TTL_SECONDS: i64 = 3600;

/// One workspace × provider pair — the identity of a sign-in. The workspace is
/// the **resolved path** both chokepoints already hold (REMOTE §8's one
/// resolution), never the name a seat spelled.
type Key = (PathBuf, String);

/// One run: the streamed child and its accumulated view (`LoginRun` holds
/// both), the serial its reader thread was started under, and when it began.
struct Slot {
    run: LoginRun,
    serial: u64,
    at: i64,
}

/// The map behind the handle, plus the serial the next run is stamped from.
/// `pub(crate)` because the lock that guards it is the crate's chokepoint's
/// ([`state::LoginCell`](crate::state::LoginCell), AGENTS rule 7).
#[derive(Default)]
pub(crate) struct Board {
    live: BTreeMap<Key, Slot>,
    seq: u64,
}

/// The engine's sign-in runs, shared by handle exactly as
/// [`Mailbox`](crate::registry::mailbox::Mailbox) and
/// [`Presence`](crate::registry::presence::Presence) are.
///
/// A default one runs whatever `bz` the ambient environment resolves and holds
/// nothing — the posture of a face that builds its own `Deps` (the §4.3 pilot),
/// which fires no sign-in. The engine's is [`of`](Self::of), over the world-
/// nested `bz` every other substrate spawn goes through.
#[derive(Clone)]
pub struct Runs {
    cell: LoginCell,
    bz: Cli,
    tick: Duration,
}

impl Default for Runs {
    fn default() -> Self {
        Self::of(Cli::resolve(Binary::Bz))
    }
}

impl Runs {
    /// The holder that spawns `bz` — the engine's world-nested one at boot, a
    /// fake script's under test.
    pub fn of(bz: Cli) -> Self {
        Self {
            cell: LoginCell::default(),
            bz,
            tick: READ_TICK,
        }
    }

    /// **Start the sign-in for this pair** and answer its standing at once.
    ///
    /// `world` is the composed world and `workspace` the gesture's own resolved
    /// path (bl-fcd5, never a focus): the wall is folded ONCE, by the very lens
    /// `boundary::config` reads providers through
    /// ([`wall::env`](crate::world::wall::env)), and laid on the child by that
    /// lens's own inverse ([`wall::pairs_of`](crate::world::wall::pairs_of)) —
    /// so the sphere the flow is read in and the sphere the child writes its
    /// credential into are one value, not two derivations. A spawn failure
    /// refuses in bz's own words and has already left its synthetic
    /// `ops.jsonl` row (`login::start`).
    pub fn start(
        &self,
        world: &crate::xdg::Env,
        workspace: &Path,
        provider: &str,
        state_root: &Path,
        ts: &str,
    ) -> Result<LoginView, String> {
        let now: i64 = ts.parse().unwrap_or(0);
        let key = (workspace.to_path_buf(), provider.to_owned());
        // One wall lens, two uses: the flow read and the child's own env. Both
        // must be this workspace's sphere, and deriving them from one value is
        // what makes that structural rather than a convention.
        let wall = crate::world::wall::env(world, workspace);
        let flow = flow_of(&wall, provider);
        let bz = self.bz.and_env(crate::world::wall::pairs_of(&wall));
        let mut board = lock_runs(&self.cell);
        // The sweep, at the one moment the map can grow — and the replacement,
        // *before* the spawn, so the port a live flow holds is released rather
        // than contended (module doc).
        board.live.retain(|_, slot| now - slot.at <= TTL_SECONDS);
        board.live.remove(&key);
        let run = super::start(&bz, provider, state_root, ts, Some(workspace), flow)
            .map_err(|e| e.to_string())?;
        board.seq += 1;
        let serial = board.seq;
        let view = run.view();
        board.live.insert(
            key.clone(),
            Slot {
                run,
                serial,
                at: now,
            },
        );
        drop(board);
        let (runs, tick) = (self.clone(), self.tick);
        // The look comes AFTER the wait, which is not a nicety: a child that
        // has already exited by the thread's first look would otherwise leave
        // the wait unexecuted, and a line whose coverage depends on how fast a
        // process died is a flake, not a measurement.
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(tick);
                if !runs.drain(&key, serial) {
                    return;
                }
            }
        });
        Ok(view)
    }

    /// One look by the reader thread that owns `serial`: drain what the child
    /// produced into the buffer, and say whether this run is still its to read.
    /// `false` once it settles **or** once the slot is another run's — which is
    /// how a replacement retires the thread it replaced, with no flag to set.
    fn drain(&self, key: &Key, serial: u64) -> bool {
        let mut board = lock_runs(&self.cell);
        match board.live.get_mut(key) {
            Some(slot) if slot.serial == serial => slot.run.poll(),
            _ => false,
        }
    }

    /// **Everything this pair has said** — the act's receipt and the one-frame
    /// answer an intake that cannot hold a connection gives. A pair with no run
    /// answers the empty view: nobody has signed in here, which is a reading
    /// and not a refusal.
    pub fn standing(&self, workspace: &Path, provider: &str) -> LoginView {
        self.frame(workspace, provider, 0).unwrap_or_default()
    }

    /// What this pair has said **since `sent` lines** — one lane frame, the
    /// `Query::Follow` append discipline (REMOTE §5.5) at a different subject.
    /// `None` when no run stands, which ends the lane rather than answering for
    /// a successor the seat never asked about.
    pub(crate) fn frame(&self, workspace: &Path, provider: &str, sent: usize) -> Option<LoginView> {
        let key = (workspace.to_path_buf(), provider.to_owned());
        let board = lock_runs(&self.cell);
        let view = board.live.get(&key)?.run.view();
        Some(LoginView {
            lines: view.lines.get(sent..).unwrap_or_default().to_vec(),
            outcome: view.outcome,
            fallback: view.fallback,
        })
    }

    /// Seat an already-wired run at `at` — the seam the lane and the sweep are
    /// driven through without a process. Answers the serial its reader thread
    /// would have been started under.
    #[cfg(test)]
    pub(crate) fn seat(&self, workspace: &Path, provider: &str, run: LoginRun, at: i64) -> u64 {
        let mut board = lock_runs(&self.cell);
        board.seq += 1;
        let serial = board.seq;
        board.live.insert(
            (workspace.to_path_buf(), provider.to_owned()),
            Slot { run, serial, at },
        );
        serial
    }

    /// Drive one reader look by hand — [`drain`](Self::drain) under test.
    #[cfg(test)]
    pub(crate) fn read_once(&self, workspace: &Path, provider: &str, serial: u64) -> bool {
        self.drain(&(workspace.to_path_buf(), provider.to_owned()), serial)
    }
}

/// **Which flow `provider` is fired in**, off brazen's effective table read in
/// `wall` — the §8.3 rule 1 branch, and the whole of it. A row declaring a
/// device endpoint is fired headless; every other row, and a name the table does
/// not carry at all, gets `--browser`. The unknown name is not a special case:
/// `--browser` is the floor every oauth row can serve, so a row nothing declared
/// anything about is fired the way an undeclared row is, and bz refuses it in
/// its own words if it cannot serve that either.
fn flow_of(wall: &crate::xdg::Env, provider: &str) -> Flow {
    let headless = RealBzRunner::resolve(wall)
        .providers()
        .iter()
        .any(|row| row.name == provider && ProviderRow::headless_login(row));
    if headless {
        Flow::Device
    } else {
        Flow::Browser
    }
}

#[cfg(test)]
mod tests;
