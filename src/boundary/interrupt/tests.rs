//! Send-and-interrupt against a real trail: the order of the two acts, the two
//! §4.2 rows they leave, and what a stop that never spawned does to the deposit
//! behind it.

use std::path::PathBuf;
use std::sync::Arc;

use tempfile::{TempDir, tempdir};

use super::interrupt;
use crate::boundary::dispatch::Deps;
use crate::boundary::reply::Reply;
use crate::boundary::tests::snapshot;
use crate::cli_outbound::Cli;
use crate::opslog::tail;

const TS: &str = "2026-08-15T00:00:00Z";
const AGENT: &str = "alba-1";

/// A state root the rows land in and a workspace the bound spawn addresses.
/// `true` stands in for `litany`: it exists on every platform the suite runs on
/// and exits 0, which is all this executor reads off either half.
struct World {
    dir: TempDir,
}

impl World {
    fn new() -> World {
        let world = World {
            dir: tempdir().expect("tempdir"),
        };
        // A short verb runs *in* its workspace (bl-bf79's bound spawn), so the
        // directory has to be there or the spawn is refused before either half.
        std::fs::create_dir_all(world.workspace()).expect("workspace dir");
        world
    }

    fn workspace(&self) -> PathBuf {
        self.dir.path().join("names").join("alba")
    }

    fn state(&self) -> PathBuf {
        self.dir.path().join("state")
    }

    fn deps(&self) -> Deps {
        Deps {
            litany: Cli::new("/usr/bin/true"),
            bl: Cli::new("/no/such/bl"),
            state_root: self.state(),
            home: self.dir.path().join("home"),
            yog_data_root: self.dir.path().join("data"),
            balls_state_root: self.dir.path().join("balls"),
            yog_binary: PathBuf::from("/no/such/yog"),
            world: crate::xdg::Env::from_env(),
            snapshot: Arc::new(snapshot(&self.workspace(), "alba", Vec::new(), Vec::new())),
            caller: crate::boundary::dispatch::Caller::default(),
        }
    }

    /// The trail's verbs, oldest first — the argv word after the binary.
    fn verbs(&self) -> Vec<String> {
        tail(&self.state(), usize::MAX)
            .into_iter()
            .filter_map(|e| e.argv.get(1).cloned())
            .collect()
    }
}

/// The gesture is two acts in one order, and the trail says so: a `stop`, then
/// the `message` whose own driver-start is the trigger. Two rows, never one
/// composite — the ruling this gesture was filed under (§4.2).
#[test]
fn it_stops_then_deposits_and_leaves_one_row_for_each() {
    let world = World::new();
    let reply = interrupt(
        &world.deps(),
        TS,
        &world.workspace(),
        AGENT,
        "do this instead",
    );
    assert!(matches!(reply, Ok(Reply::Outcome(_))), "{reply:?}");
    assert_eq!(world.verbs(), vec!["stop".to_owned(), "message".to_owned()]);
}

/// The reply is the **deposit's** outcome: it is the act that resumes the
/// conversation, so its capture is the one the operator is waiting on.
#[test]
fn the_reply_is_the_deposits_own_outcome() {
    let world = World::new();
    let Ok(Reply::Outcome(outcome)) =
        interrupt(&world.deps(), TS, &world.workspace(), AGENT, "text")
    else {
        panic!("a clean pair answers with an outcome");
    };
    assert_eq!(outcome.exit, 0);
    let rows = tail(&world.state(), usize::MAX);
    assert_eq!(rows.len(), 2, "and both halves are on the trail");
}

/// A stop that never spawned aborts the gesture rather than depositing behind
/// it: the handle is broken, so the deposit would fail the same way. The
/// synthetic failure row stands alone, which is what says the deposit did not
/// run (INV-2 — nothing is dropped, the row is the record).
#[test]
fn a_stop_that_cannot_spawn_refuses_before_the_deposit() {
    let world = World::new();
    let deps = Deps {
        litany: Cli::new("/no/such/litany"),
        ..world.deps()
    };
    let refused = interrupt(&deps, TS, &world.workspace(), AGENT, "text");
    assert!(refused.is_err(), "{refused:?}");
    assert_eq!(world.verbs(), vec!["stop".to_owned()]);
}
