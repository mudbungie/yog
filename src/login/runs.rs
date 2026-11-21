//! **Where a sign-in lives while it runs** (REMOTE §8.3, DESIGN §8.3 as
//! amended by bl-61bf; bl-c285): the server's map of live sign-ins, **one per
//! workspace × provider**.
//!
//! The flow is a boundary act, not a pane's own spawn, and that moves two
//! facts here. The child runs where the *wall* is, so the credential lands in
//! the sphere whose agents read it (§16.2) whichever box the seat is on; and
//! nothing waits for it — the intake is one thread for the whole world (REMOTE
//! §3), so [`Runs::start`] answers the run's standing at once and every later
//! look is a re-read of that same standing.
//!
//! **Nothing here chooses a flow** (§8.3 rule 1 as amended by bl-0127). Every
//! row is fired the same bare command line and its own brazen serves whichever
//! sign-in it has — this workspace's for a bare row, the named machine's for
//! `<row>@<client>` (REMOTE §5.7), reached by the `bz` arm
//! ([`crate::bz_host::far`]) with no second path. So the act reads no catalog
//! before it spawns, and a machine that does not answer is a line in the lane
//! rather than a refusal at the door.
//!
//! **A `Login` naming no code starts the pair's sign-in over; one naming a
//! code finishes the run that is waiting for it.** Those are the act's only
//! two meanings and each is its own invariant:
//!
//! - *Start* terminates and replaces whatever stood. The operator's own
//!   restart is the cancel, so there is no cancel verb, and of any number of
//!   starts only the newest can finish — the row's brazen parks one verifier
//!   per row and each start overwrites it (brazen's pasted-code sign-in). Two seats
//!   starting the same sign-in therefore share one run and one URL: the
//!   earlier lane ends, its re-ask replays the newer run, and a code obtained
//!   at the older URL is refused by the brazen that issued it.
//! - *Finish* is taken only by a run standing with an `authorize` URL. A code
//!   handed to a pair with no such run — nothing started, a device flow, a run
//!   already settled, a server that restarted and forgot — is refused here,
//!   naming the restart, and touches nothing. Which seat sends it is not
//!   asked: every seat on the workspace is the operator, and the code is
//!   worthless to anything but the verifier it was issued against.
//!
//! **A reader thread per command line, and lanes that only read.** Nothing
//! else polls the child: two seats may hold lanes on one run and a run with no
//! lane at all still has to settle, because settling is what writes the
//! `ops.jsonl` outcome row (§4.2, `LoginRun::finalize`'s one-source rule). So
//! the thread owns the *cadence* and the map owns the *run*, and a lane reads
//! the buffer. The thread retires the moment its command settles, stands
//! waiting, or stops being its run — the serial is that test, re-stamped by a
//! replacement and by a redeemed code alike, and it needs no second flag for
//! anyone to forget to set.
//!
//! **A run stays until it is swept**, settled or waiting, so re-asking
//! replays: a dropped lane, a re-attached seat and a finished sign-in are one
//! case. The bound is the §5.3 mailbox's own — an hour from the START — swept
//! at the one moment the map can grow, exactly as `registry::mailbox` sweeps
//! its slots. An abandoned waiting run costs nothing meanwhile: no child, no
//! port, and brazen's parked verifier lapses on its own in ten minutes.
//!
//! The lock itself is the crate's chokepoint's (AGENTS rule 7,
//! [`state::LoginCell`](crate::state::LoginCell)); what lives here is the map
//! it guards and every rule about what may be in it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{LoginRun, LoginView};
use crate::cli_outbound::{Binary, Cli};
use crate::state::{LoginCell, lock_logins as lock_runs};

/// How often a reader thread drains its child. The §7.2 follower's own period,
/// which is what "live" means in a number — and the lane's tick, so a line
/// reaches a seat within one look of arriving rather than two.
const READ_TICK: Duration = Duration::from_millis(16);

/// How long a run survives after it began: an hour, the §5.3 mailbox bound. It
/// is the *whole* run's age and not its idle time, because a sign-in the
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

/// The server's sign-in runs, shared by handle exactly as
/// [`Mailbox`](crate::registry::mailbox::Mailbox) and
/// [`Presence`](crate::registry::presence::Presence) are.
///
/// A default one runs whatever `bz` the ambient environment resolves and holds
/// nothing — the posture of a face that builds its own `Deps` (the §4.3 pilot),
/// which fires no sign-in. The server's is [`of`](Self::of), over the world-
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
    /// The holder that spawns `bz` — the server's world-nested one at boot, a
    /// fake script's under test.
    pub fn of(bz: Cli) -> Self {
        Self {
            cell: LoginCell::default(),
            bz,
            tick: READ_TICK,
        }
    }

    /// **Start this pair's sign-in — or, given a `code`, finish the one that
    /// is waiting for it** — and answer its standing at once (module doc).
    ///
    /// `world` is the composed world and `workspace` the gesture's own resolved
    /// path (bl-fcd5, never a focus): the wall is folded ONCE, by the very lens
    /// `boundary::config` reads providers through
    /// ([`wall::env`](crate::world::wall::env)), and laid on the child by that
    /// lens's own inverse ([`wall::pairs_of`](crate::world::wall::pairs_of)) —
    /// so the sphere the child writes its credential into is the one the
    /// gesture named. A spawn failure refuses in bz's own words and has
    /// already left its synthetic `ops.jsonl` row (`login::start`).
    pub fn start(
        &self,
        world: &crate::xdg::Env,
        workspace: &Path,
        provider: &str,
        code: Option<&str>,
        state_root: &Path,
        ts: &str,
    ) -> Result<LoginView, String> {
        let now: i64 = ts.parse().unwrap_or(0);
        let key = (workspace.to_path_buf(), provider.to_owned());
        let wall = crate::world::wall::env(world, workspace);
        let bz = self.bz.and_env(crate::world::wall::pairs_of(&wall));
        let mut board = lock_runs(&self.cell);
        // The sweep, at the one moment the map can grow.
        board.live.retain(|_, slot| now - slot.at <= TTL_SECONDS);
        board.seq += 1;
        let serial = board.seq;
        let view = if let Some(code) = code {
            let waiting = |slot: &&mut Slot| slot.run.view().authorize.is_some();
            let Some(slot) = board.live.get_mut(&key).filter(waiting) else {
                return Err(nothing_waiting(provider));
            };
            slot.run.redeem(&bz, code, ts).map_err(|e| e.to_string())?;
            slot.serial = serial;
            slot.run.view()
        } else {
            // The replacement drops the old run BEFORE the spawn: its child is
            // terminated first, never left racing its successor.
            board.live.remove(&key);
            let run = super::start(&bz, provider, state_root, ts, Some(workspace))
                .map_err(|e| e.to_string())?;
            let view = run.view();
            let at = now;
            board.live.insert(key.clone(), Slot { run, serial, at });
            view
        };
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
    /// `false` once the command settles or stands waiting, **or** once the
    /// slot's serial is another's — which is how a replacement and a redeemed
    /// code each retire the thread before them, with no flag to set.
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
            ..view
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

/// The refusal a code earns where no run is waiting for one (module doc):
/// the fact, and the one act that changes it.
fn nothing_waiting(provider: &str) -> String {
    format!(
        "no sign-in for {provider:?} is waiting for a code here — start one with /{} {provider}",
        crate::boundary::codec::LOGIN
    )
}

#[cfg(test)]
mod tests;
