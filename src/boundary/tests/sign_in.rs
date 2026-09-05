//! bl-2291 — **sign in first** (DESIGN §8.1's bl-1fd0 ruling), asserted at the
//! `Prompt` door over real walls: one with nothing signed in, one with a stored
//! credential, one whose lineage names a keyless row by hand, and one whose
//! brazen cannot answer at all.
//!
//! Each refusing beat also asserts that nothing was paid for it — the trail
//! file is never created — so the gate stands ahead of the confinement read
//! and of the ceiling's own `["yog-step","ceiling"]` row.

use super::snapshot;
use crate::boundary::dispatch::{Deps, prompt};
use crate::bz_host::store::WallCredStore;
use crate::cli_outbound::Cli;
use crate::config_edit::brazen::BrazenPaths;
use crate::git_tree::tests::fixture::Fixture;
use crate::opslog::Origin;
use crate::start::Prepared;
use crate::test_support::world_under;
use crate::ui_state::UiState;
use brazen::{Cred, CredStore as _, Secret};
use std::path::PathBuf;
use std::sync::Arc;

/// The word every refusal opens with — stated once so a reworded refusal
/// fails loudly.
const REFUSAL: &str = "sign in first";

/// A brazen config declaring one keyed row and one keyless row, so the wall's
/// table holds both spellings this gate judges beside brazen's built-ins.
const TABLE: &str = "[[provider]]\nname = \"keyed\"\nprotocol = \"openai_chat\"\n\
                     base_url = \"https://keyed.test\"\nauth = \"api_key\"\n\
                     api_header = { name = \"x-api-key\", scheme = \"raw\" }\n\n\
                     [[provider]]\nname = \"local\"\nprotocol = \"openai_chat\"\n\
                     base_url = \"http://localhost:1\"\nauth = \"none\"\n";

/// A workspace with a bare repo carrying `config/default` (the lineage the
/// fire forks off), and a world whose wall for it holds [`TABLE`].
struct Wall {
    fixture: Fixture,
    world: crate::xdg::Env,
    state: PathBuf,
}

impl Wall {
    fn new() -> Self {
        let fixture = Fixture::new();
        let world = world_under(&fixture.path.join("world"));
        let paths = BrazenPaths::in_wall(&crate::world::wall::root_of(&world, &fixture.path));
        std::fs::create_dir_all(paths.config.parent().expect("brazen dir")).unwrap();
        std::fs::write(&paths.config, TABLE).unwrap();
        Self {
            state: fixture.path.join("state"),
            fixture,
            world,
        }
    }

    fn workspace(&self) -> PathBuf {
        self.fixture.path.clone()
    }

    /// Sign the keyed row in, the way `bz --login` would leave the wall.
    fn sign_in(&self) {
        let paths =
            BrazenPaths::in_wall(&crate::world::wall::root_of(&self.world, &self.workspace()));
        WallCredStore::new(paths.credentials_dir, self.world.clone())
            .put(
                "keyed",
                &Cred::ApiKey {
                    key: Secret::new("k-notreal"),
                },
            )
            .unwrap();
    }

    /// Substrate binaries that do not exist: a beat here that reached the
    /// spawn fails loudly on the fork rather than passing on a mock.
    fn deps(&self) -> Deps {
        let ws = self.workspace();
        Deps {
            litany: Cli::new("/no/such/litany"),
            bl: Cli::new("/no/such/bl"),
            state_root: self.state.clone(),
            yog_binary: PathBuf::from("/no/such/yog"),
            world: self.world.clone(),
            home: self.fixture.path.join("home"),
            yog_data_root: self.fixture.path.join("data"),
            balls_state_root: self.fixture.path.join("balls"),
            snapshot: Arc::new(snapshot(&ws, "alba", vec![], vec![])),
            caller: crate::boundary::dispatch::Caller::default(),
        }
    }

    fn fire(&self) -> Result<String, String> {
        let ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
        prompt(
            &self.deps(),
            &ui,
            "T1",
            &self.workspace(),
            &prepared(),
            "do the thing",
            None,
        )
    }

    fn trail(&self) -> bool {
        self.state.join("ops.jsonl").exists()
    }
}

/// A `Prepared` for the bare rung off the default lineage.
fn prepared() -> Prepared {
    Prepared {
        workspace: "alba".to_owned(),
        binding: None,
        lineage: None,
        goal: String::new(),
        origin: Origin::Conversation,
    }
}

/// The ruling itself: a wall with nothing signed in refuses the fire whole, in
/// the words that name the act, and costs no trail row.
#[test]
fn an_unsigned_wall_refuses_the_fire_and_pays_nothing() {
    let wall = Wall::new();
    let refusal = wall.fire().unwrap_err();
    assert!(refusal.starts_with(REFUSAL), "{refusal}");
    assert!(refusal.contains("/login"), "{refusal}");
    assert!(refusal.contains("keyed"), "the rows are named: {refusal}");
    assert!(!wall.trail(), "a refused fire costs no trail row");
}

/// A stored credential on any row readies the wall — the fire passes this
/// gate and fails only where it always would with no substrate to spawn,
/// which is a different sentence and a trail row of its own.
#[test]
fn a_signed_wall_passes_the_gate() {
    let wall = Wall::new();
    wall.sign_in();
    let past = wall.fire().unwrap_err();
    assert!(!past.contains(REFUSAL), "{past}");
    assert!(wall.trail(), "the fire reached the spawn and left its row");
}

/// A keyless row readies nothing by itself (the built-in table merges under
/// every config), and readies the wall exactly when the lineage names it in a
/// role — the operator's own hand.
#[test]
fn a_role_naming_a_keyless_row_readies_the_wall() {
    let wall = Wall::new();
    assert!(wall.fire().unwrap_err().starts_with(REFUSAL));
    wall.fixture.commit_other(
        "providers.yaml",
        "roles:\n  worker:\n    provider: local\n    model: tiny\n",
    );
    let past = wall.fire().unwrap_err();
    assert!(!past.contains(REFUSAL), "{past}");
}

/// A role naming a row that is `missing` does not ready the wall: `missing` is
/// the one spelling that is a refusal, whoever named the row.
#[test]
fn a_role_naming_an_unsigned_keyed_row_is_still_refused() {
    let wall = Wall::new();
    wall.fixture.commit_other(
        "providers.yaml",
        "roles:\n  worker:\n    provider: keyed\n    model: big\n",
    );
    assert!(wall.fire().unwrap_err().starts_with(REFUSAL));
    assert!(!wall.trail());
}

/// A brazen that cannot answer is an empty table, and no surface refuses on
/// the strength of a question that went unanswered: a wall whose config brazen
/// cannot read passes this gate to the ones behind it. (A wall with NO config
/// is not that case — brazen's built-in table merges under every config, so a
/// missing file still answers eight rows, and the first beat above is what it
/// answers.)
#[test]
fn an_unanswerable_table_refuses_nothing() {
    let wall = Wall::new();
    let paths = BrazenPaths::in_wall(&crate::world::wall::root_of(&wall.world, &wall.workspace()));
    std::fs::write(&paths.config, "[[provider]\nthis is not toml\n").unwrap();
    let past = wall.fire().unwrap_err();
    assert!(!past.contains(REFUSAL), "{past}");
}
