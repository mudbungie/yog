//! The populated fixture world the acceptance tests render (§11): one
//! workspace symlinked under the lernie root with a transcript, a settled
//! auth-failed step with tool i/o, and an inbox deposit — every inspector
//! surface has something to paint. Split from the smoke test for §12's budget.

use super::super::ShellState;
use crate::AppModel;
use crate::app::Roots;
use crate::cli_outbound::Cli;
use crate::git_tree::tests::fixture::Fixture;
use crate::projects::runner::BlStore;
use crate::ui_state::SystemClock;
use crate::xdg::Env;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

/// The mint seed every acceptance world starts from (bl-cba6). The shell rolls
/// this from [`entropy_seed`](crate::shell::clock) at construction, which made
/// the §3.3 name preview a different word every run — and the preview reaches
/// the paint layer (itself a §3.3 rule), so any needle another assertion
/// searched `Screen::text` for could collide with it by chance: `ping` inside a
/// minted `dripping` failed an unrelated drafts assertion once in ~10 runs.
/// **The entropy was the defect.** A world's seed is an input the fixture owns,
/// not one it inherits from the process — pinned, the opening preview is always
/// [`MINTED_FIRST`], so an assertion can name the word outright instead of
/// dodging it, and a collision would be a reproducible failure rather than a
/// one-in-ten flake. Arbitrary but not accidental: this value's word shares no
/// substring with any needle the suite searches for.
pub(super) const MINT_SEED: u64 = 0xc0df;

/// The word [`MINT_SEED`]'s one draw (§3.3 mints on a single draw) yields
/// against an empty occupied set — what every acceptance world's opening
/// preview paints. The pool is **lernie's** since bl-cd38 (yog deleted its own
/// list and draws through [`lernie::mint`]), so this pair is also the seam
/// check on that consumption: a corpus change in the crate moves it. Pinned in
/// `mint_seed.rs`, so such a change fails *there*, naming the cause, instead of
/// surfacing as a stray needle collision somewhere else in the suite.
pub(super) const MINTED_FIRST: &str = "metronome";

/// A `lernie` whose `new` authors the workspace the real one does (ARCH §2.2);
/// every other verb exits 0. Written executable into `dir`. Shared by every
/// acceptance test whose gesture has to *land* — one fake, so a start that must
/// succeed succeeds the same way wherever it is driven from.
pub(super) fn fake_lernie(dir: &Path) -> Cli {
    let path = dir.join("lernie");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\ncase \"$1\" in\n{}esac\nexit 0\n",
            crate::test_support::authoring_new_arm()
        ),
    )
    .unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).unwrap();
    Cli::new(path)
}

/// Seed the nested world's `LERNIE_HOME` marker (§16.6 W3) so a start's
/// `EnsureSeeded` step short-circuits instead of shelling `lernie prime` —
/// keeping every fake to the verbs the test's own gesture really runs.
pub(super) fn seed_world(world: &World) {
    let lernie_home = crate::world::layout_under(&world.yog_data).lernie;
    std::fs::create_dir_all(&lernie_home).unwrap();
    std::fs::write(lernie_home.join("models.yaml"), b"models: {}\n").unwrap();
}

/// A workspace populated across every inspector surface (transcript, steps +
/// tool i/o, inbox), symlinked under the lernie workspaces root so the model
/// enumerates it.
pub(super) struct World {
    _root: TempDir,
    fx: Fixture,
    /// Every extra sphere [`World::add_workspace`] minted, held only so their
    /// temp dirs outlive the test — the §3.1 wall boundary is unobservable with
    /// one workspace, so a wall drive needs a second.
    spheres: Vec<Fixture>,
    pub(super) model: AppModel,
    /// The §7.2 derivation, driven by hand: in the app a `Worker` thread runs
    /// it, and the frame renders only what it publishes (bl-ee0a).
    deriver: crate::app::Deriver,
    pub(super) state: ShellState,
    pub(super) ws: PathBuf,
    /// yog's own data root — where the §16 nested world and the §3.1 names root
    /// live. A raise mints under it, so a test that drives one needs to seed the
    /// world here and read the raised sphere back.
    pub(super) yog_data: PathBuf,
    /// Where a second sphere is symlinked from ([`World::add_workspace`]).
    lernie_workspaces: PathBuf,
}

impl World {
    /// One derivation pass and the frame's take of it — what the smoke test
    /// does whenever it has just changed something on disk.
    pub(super) fn converge(&mut self) {
        self.deriver.step();
        self.model.refresh();
    }

    /// Mint a **second sphere** under the same lernie root: another workspace
    /// with one conversation, symlinked where the model enumerates it. Its §3.1
    /// leaf names its own wall (§16.2 as amended), so this is what a wall drive
    /// switches to. Caller converges to fold it in.
    pub(super) fn add_workspace(&mut self, name: &str, agent: &str) -> PathBuf {
        let fx = Fixture::new();
        fx.build_agent(agent, name);
        let ws = self.lernie_workspaces.join(name);
        std::os::unix::fs::symlink(&fx.path, &ws).unwrap();
        self.spheres.push(fx);
        ws
    }

    /// Fork a nameless descent child off `parent_id` (§2.3) — the bl-63a1
    /// chained-id shape: no name blob, no goal on disk, no step record, so the
    /// §3.3 ladder bottoms out at its floor. Caller converges to fold it in.
    pub(super) fn add_child(&self, parent_id: &str, child_id: &str) {
        self.fx.build_child(parent_id, child_id);
    }

    /// A **second root** conversation (§2.3) wearing `name` as its §3.3 name
    /// fact — a second row in the list, so a test can compare two rows of the
    /// same list rather than two worlds. Caller converges to fold it in.
    pub(super) fn add_root(&self, conv_id: &str, name: &str) {
        self.fx.build_agent(conv_id, name);
        self.fx.name_agent(conv_id, name);
    }

    /// Mark `conv_id` **abandoned** — §6's will-not-retry assertion, the one
    /// gate that suppresses rule 2 (`attention::rest_evidence`). It is how a
    /// fixture gets a settled row bearing **no** attention beside one that
    /// does, without focusing anything and so without spending an ack.
    pub(super) fn quiet(&self, conv_id: &str) {
        self.fx
            .mark_ref(&format!("refs/lernie/abandoned/{conv_id}"));
    }
}

/// **Whether the fixture's world has a workspace in it** — the one axis these
/// builders differ on. Named rather than spelled as a bool because the state it
/// distinguishes was, until bl-37bf, not reachable at all.
#[derive(PartialEq, Eq)]
enum Roster {
    /// One workspace, enumerated and focused.
    One,
    /// **None** — an operator's very first launch, and the only state in which
    /// `shell::bootstrap` paints: `shell::workspace::center` hands over to it
    /// exactly when `focused_workspace()` is `None`.
    Empty,
}

pub(super) fn world_titled(title: &str) -> World {
    build_world(title, &Roster::One)
}

pub(super) fn world() -> World {
    build_world("hello", &Roster::One)
}

/// **A world with no workspace in it** — the §3.4 / STORIES S0 first launch, and
/// the only fixture from which `shell::bootstrap` is reachable (bl-37bf).
///
/// It differs from [`world`] by exactly the fact that makes it empty: the
/// conversation is built and then not symlinked where the model enumerates it,
/// so the roster is empty and everything else about the world is identical.
///
/// Its predecessor `world_unfocused` withheld only the *startup focus* argument
/// and claimed in its doc to render the bootstrap composer. It never did — the
/// roster still held the workspace, so `AppModel::startup_focus` derived a focus
/// onto it and the centre painted the conversation view. A test named for the
/// bootstrap ran against the start pane for as long as that fixture existed.
pub(super) fn world_empty() -> World {
    build_world("hello", &Roster::Empty)
}

fn build_world(title: &str, roster: &Roster) -> World {
    let root = tempdir().unwrap();
    let yog_data = root.path().join("yog");
    let roots = Roots {
        yog_data: yog_data.clone(),
        lernie_data: root.path().join("lernie"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("balls").join("clones"),
        home: root.path().join("home"),
        // A hermetic world rooted at this fixture's own temp dir, whose lernie
        // leaf IS `roots.lernie_data` — the coincidence production has (§16.2:
        // `LERNIE_HOME` nests lernie's config and data onto one dir), so the
        // §9.2 `models.yaml` a test writes is the one the worker reads.
        world: crate::test_support::world_under(root.path()),
    };
    std::fs::create_dir_all(roots.lernie_data.join("workspaces")).unwrap();
    // The §9.2 global `models.yaml` a founded world carries, declaring the
    // §5.1 #35 context window of `m` — the model the fixture's step records
    // name in their own `request.json` (`write_step_record`).
    std::fs::write(
        roots.lernie_data.join("models.yaml"),
        b"models:\n  m:\n    provider: anthropic\n    model_id: m\n    context_window: 200000\n",
    )
    .unwrap();
    std::fs::create_dir_all(&roots.yog_state).unwrap();
    let fx = Fixture::new();
    fx.build_agent("c-1", title);
    let messages = fx.path.join("agents/c-1/messages");
    std::fs::create_dir_all(&messages).unwrap();
    std::fs::write(messages.join("001-user.md"), "please ping").unwrap();
    std::fs::write(
        messages.join("002-opus.json"),
        br#"{"content":[{"type":"text","text":"pong reply"}]}"#,
    )
    .unwrap();
    let step = fx.path.join("steps/c-1/001");
    std::fs::create_dir_all(step.join("tools/toolu_1")).unwrap();
    // A settled auth-shaped failure (the live stench-pug kind:auth shape) with a
    // usage event: framing Failed → Stopped, tokens still fold, and the §11
    // inline Login banner must reach the paint layer.
    std::fs::write(
        step.join("response.json"),
        b"{\"type\":\"usage\",\"input_tokens\":10,\"output_tokens\":5,\"cache_read_tokens\":50000}\n{\"type\":\"error\",\"kind\":\"auth\",\"message\":\"no credential for this provider\",\"provider_detail\":null}\n{\"type\":\"end\"}\n",
    )
    .unwrap();
    std::fs::write(step.join("tools/toolu_1/input.json"), br#"{"name":"Read"}"#).unwrap();
    std::fs::write(
        step.join("tools/toolu_1/output.json"),
        br#"{"exit_code":0}"#,
    )
    .unwrap();
    fx.deposit_message(
        "c-1",
        "user-001.md",
        "---\nfrom: user\ndeposited_at: t0\n---\nfollow-up message",
    );
    let lernie_workspaces = roots.lernie_data.join("workspaces");
    let ws = lernie_workspaces.join("ws");
    // The one line that makes a world empty: unlinked, the conversation is on
    // disk and invisible to enumeration, so the roster has nothing in it and
    // `startup_focus` has nothing to derive.
    if *roster != Roster::Empty {
        std::os::unix::fs::symlink(&fx.path, &ws).unwrap();
    }
    let env = Env::from_pairs([("HOME", root.path().display().to_string())]);
    let balls = Box::new(BlStore::new(env.balls_layout(), Cli::new("bl")));
    let (model, deriver) = AppModel::boot(
        roots,
        (*roster == Roster::One).then(|| ws.clone()),
        Arc::new(SystemClock),
        balls,
        Some("me".into()),
    );
    let mut state = ShellState::new(&env, Arc::new(SystemClock)).unwrap();
    // The one entropy read the acceptance suite refuses to inherit (bl-cba6):
    // pinned so the §3.3 preview paints a known word every run.
    state.start.mint_seed = MINT_SEED;
    World {
        _root: root,
        fx,
        spheres: Vec::new(),
        model,
        deriver,
        state,
        ws,
        yog_data,
        lernie_workspaces,
    }
}
