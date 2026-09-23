//! The §3.5 spend family at its executors (bl-53d1): the read's one
//! derivation, each act's write-through and receipt, and the release
//! selecting only the marks the ceiling wrote — driven through the same
//! chokepoint every seat enters.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::boundary::dispatch::{Deps, dispatch};
use crate::boundary::reply::Reply;
use crate::boundary::tests::snapshot;
use crate::boundary::{Action, Query, answer::answer};
use crate::cli_outbound::Cli;
use crate::opslog::tail;
use crate::spend::{CEILING_HEAD, Price};
use crate::ui_state::UiState;
use tempfile::{TempDir, tempdir};

const AGENT: &str = "20260101T000000Z-a1";
const OTHER: &str = "20260101T000000Z-a2";

/// One yog-owned workspace under a names root the roster enumerates, with a
/// bare repo for the marks and a `steps/` tree that has spent $3 on `opus`.
struct World {
    dir: TempDir,
}

impl World {
    fn new() -> World {
        let dir = tempdir().expect("tempdir");
        let world = World { dir };
        std::fs::create_dir_all(world.workspace().join("repo.git")).unwrap();
        world.git(&["init", "--bare", "-q"]);
        world
    }

    fn workspace(&self) -> PathBuf {
        self.dir.path().join("data").join("workspaces").join("alba")
    }

    fn state(&self) -> PathBuf {
        self.dir.path().join("state")
    }

    fn ui(&self) -> UiState {
        UiState::open(self.state().join("ui.json"))
    }

    fn deps(&self) -> Deps {
        Deps {
            // `true` exits 0: enough to prove the detached launch happened.
            litany: Cli::new("/usr/bin/true"),
            bl: Cli::new("/no/such/bl"),
            state_root: self.state(),
            home: self.dir.path().join("home"),
            yog_data_root: self.dir.path().join("data"),
            yog_binary: PathBuf::from("/no/such/yog"),
            world: crate::test_support::no_world(),
            snapshot: Arc::new(snapshot(&self.workspace(), "alba", Vec::new(), Vec::new())),
            caller: crate::boundary::dispatch::Caller::default(),
        }
    }

    fn git(&self, args: &[&str]) -> std::process::Output {
        crate::git_env::output(
            crate::git_env::git()
                .arg("--git-dir")
                .arg(self.workspace().join("repo.git"))
                .args(args),
        )
        .expect("git runs")
    }

    /// Three million input tokens on `opus` — $3 at the table below.
    fn spent(&self) {
        let step = self.workspace().join("steps").join(AGENT).join("001");
        std::fs::create_dir_all(&step).unwrap();
        std::fs::write(
            step.join("response.json"),
            r#"{"type":"usage","input_tokens":3000000}"#,
        )
        .unwrap();
        std::fs::write(step.join("request.json"), r#"{"model":"opus"}"#).unwrap();
    }

    /// Park `agent` under `reason`, exactly as litany's seam does.
    fn park(&self, agent: &str, reason: &str) {
        let staged = self.dir.path().join("mark.json");
        let blob = serde_json::json!({ "tool_use_id": "t", "tool": "bash", "reason": reason });
        std::fs::write(&staged, blob.to_string()).unwrap();
        let hashed = self.git(&["hash-object", "-w", "--", &staged.to_string_lossy()]);
        let oid = String::from_utf8_lossy(&hashed.stdout).trim().to_owned();
        self.git(&["update-ref", &format!("refs/litany/held/{agent}"), &oid]);
    }
}

fn prices(reply: Result<Reply, String>) -> crate::boundary::reply::PricesView {
    match reply {
        Ok(Reply::Prices(view)) => view,
        other => panic!("a prices reply: {other:?}"),
    }
}

fn price(
    ui: &mut UiState,
    deps: &Deps,
    model: &str,
    rates: Option<Price>,
) -> crate::boundary::reply::PricesView {
    prices(dispatch(
        deps,
        ui,
        "1000",
        &Action::Price {
            provider: "anthropic".to_owned(),
            model: model.to_owned(),
            rates,
        },
    ))
}

fn ceiling(
    ui: &mut UiState,
    deps: &Deps,
    micro_usd: Option<u64>,
) -> crate::boundary::reply::PricesView {
    prices(dispatch(deps, ui, "1000", &Action::Ceiling { micro_usd }))
}

/// A row written is in the receipt and in the next read, priced spend appears
/// the moment the world is priced, and the last delete empties both.
#[test]
fn a_price_write_receipts_the_re_read_table_and_the_world_s_ledger() {
    let world = World::new();
    world.spent();
    let (deps, mut ui) = (world.deps(), world.ui());
    let read = prices(answer(&Query::Prices, &deps, &ui, 0));
    assert!(read.rows.is_empty() && read.spent.is_none() && read.released.is_none());
    let one = Price {
        input: 1_000_000,
        ..Price::default()
    };
    let receipt = price(&mut ui, &deps, "opus", Some(one));
    assert_eq!(receipt.rows.len(), 1);
    assert_eq!(receipt.rows[0].rates, one);
    assert_eq!(receipt.spent.map(|c| c.usd()), Some("$3.00".to_owned()));
    assert_eq!(receipt.released, None, "a row write releases nothing");
    assert_eq!(prices(answer(&Query::Prices, &deps, &ui, 0)), receipt);
    assert_eq!(world.ui().prices(), ui.prices(), "written through");
    let gone = price(&mut ui, &deps, "opus", None);
    assert!(gone.rows.is_empty() && gone.spent.is_none());
}

/// The ceiling's write-through and its release: parked under the ceiling's
/// own sentence, a conversation is driven when the number lifts the world
/// back under it, and only that conversation — a policy hold stays parked.
#[test]
fn a_ceiling_lifted_over_the_spend_releases_only_the_marks_the_ceiling_wrote() {
    let world = World::new();
    world.spent();
    world.park(AGENT, &format!("{CEILING_HEAD} this world has spent $3.00"));
    world.park(OTHER, "bash {\"command\":\"curl x\"} classified open-world");
    let (deps, mut ui) = (world.deps(), world.ui());
    price(
        &mut ui,
        &deps,
        "opus",
        Some(Price {
            input: 1_000_000,
            ..Price::default()
        }),
    );
    // Set under the spend: still over, nothing released.
    let over = ceiling(&mut ui, &deps, Some(2_000_000));
    assert_eq!(over.ceiling.micro_usd(), Some(2_000_000));
    assert_eq!(over.released, Some(0));
    assert!(
        tail(&world.state(), usize::MAX).is_empty(),
        "nothing launched"
    );
    // Raised over it: the ceiling's mark is driven, the other is not.
    let lifted = ceiling(&mut ui, &deps, Some(5_000_000));
    assert_eq!(lifted.released, Some(1));
    let rows = tail(&world.state(), usize::MAX);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].argv.get(1).map(String::as_str), Some("advance"));
    assert_eq!(rows[0].argv.get(3).map(String::as_str), Some(AGENT));
    // Deleted: released again (the mark is litany's to lift), and read back
    // as no ceiling from the file.
    let off = ceiling(&mut ui, &deps, None);
    assert_eq!(off.released, Some(1));
    assert_eq!(off.ceiling, crate::spend::Ceiling::default());
    assert_eq!(world.ui().ceiling(), crate::spend::Ceiling::default());
}

/// An unpriced world bounds nothing, so a ceiling set over it releases at
/// once — the severability read a third time — and the roster reads nothing
/// where no names root exists.
#[test]
fn an_unpriced_world_releases_on_any_ceiling_and_an_empty_roster_holds_nothing() {
    let world = World::new();
    world.park(AGENT, &format!("{CEILING_HEAD} stale"));
    let (deps, mut ui) = (world.deps(), world.ui());
    assert_eq!(ceiling(&mut ui, &deps, Some(0)).released, Some(1));
    let none = Deps {
        yog_data_root: PathBuf::from("/nonexistent/data"),
        ..deps
    };
    assert_eq!(ceiling(&mut ui, &none, Some(0)).released, Some(0));
    assert!(!Path::new("/nonexistent/data").exists());
}
