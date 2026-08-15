//! The populated fixture world the acceptance tests render (§11): one
//! workspace symlinked under the lernie root with a transcript, a settled
//! auth-failed step with tool i/o, and an inbox deposit — every inspector
//! surface has something to paint. Split from the smoke test for §12's budget.

use super::super::ShellState;
use crate::AppModel;
use crate::app::Roots;
use crate::cli_outbound::Cli;
use crate::git_tree::tests::fixture::Fixture;
use crate::model_pick::PROVIDERS;
use crate::projects::runner::BlStore;
use crate::ui_state::SystemClock;
use crate::xdg::Env;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

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
/// substring with any needle the suite searches for. Pinning it pins the
/// **whole session** (bl-dd3d), not just its opening: a landed fire takes its
/// next seed off this one's stream (`StartState::spend_mint`), never a second
/// entropy read, so [`MINTED`] is a fact of this value, not a run of dice.
pub(super) const MINT_SEED: u64 = 0xc0df;

/// The words a [`MINT_SEED`] world mints, in fire order (§3.3 draws once
/// apiece, each off the seed the fire before it spent) — three, the run the
/// bl-28ba drive walks. Naming them is what retired that drive's probabilistic
/// `assert_ne!` (bl-dd3d): "a fresh name each time" is read off the pinned
/// sequence, so a mint regressing to repeats fails every run, not one in 541.
/// The pool is **lernie's** since bl-cd38 (yog deleted its own list and draws
/// through [`lernie::mint`]), so this is also the seam check on that
/// consumption: a corpus change moves it, and fails in `mint_seed.rs` naming
/// the cause instead of as a needle collision elsewhere in the suite.
pub(super) const MINTED: [&str; 3] = ["metronome", "granola", "balmy"];

/// [`MINTED`]'s opening — named alone because tests that merely *contain* a
/// minted name, rather than walking the sequence, only ever see this one.
pub(super) const MINTED_FIRST: &str = MINTED[0];

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

/// The world a test drives, and the wire behind it — its own file per §12's
/// budget.
mod world;
pub(in crate::shell::acceptance) use world::World;

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
    // The §9.4 picker's config lineage (bl-a842). Without a `providers.yaml` on
    // `config/default` the pane takes its first early return — "cannot read
    // `roles:`" — and everything below it (the role strip, the two dropdowns,
    // the roster query and its remedy) is unreachable from any acceptance test.
    // The text is `TEMPLATE_PROVIDERS`, the same bytes lernie's own
    // `template/providers.yaml` commits and the `model_pick` unit tests pin, so
    // the fixture and those tests read ONE shape.
    //
    // It is committed **before** the agent branch forks, which is the whole
    // reason this seam moves no existing beat: `config/default`'s tip is then
    // the conversation's own governing commit, so the §9.4 row claims no drift
    // and grows no drift clause. Seeding it afterwards would advance the config
    // past every conversation and paint one on every settings surface.
    fx.commit_other(PROVIDERS, crate::test_support::TEMPLATE_PROVIDERS);
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
    let (mut model, deriver) = AppModel::boot(
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
    // **The wire the window is a client of** (REMOTE §1.2 as ruled 2026-08-14,
    // §9.8): a fixture mints no certificate and binds no port, so the transport
    // is stood in for — but the ENDS are the real ones, taken here at boot
    // exactly as `Engine::window_wire` hands them over. Without them every act
    // this world fires answers "this window has no wire behind it", which is a
    // window whose every gesture is refused.
    let (link, link_end) = crate::wire::link::pair();
    let (post, outbox) = crate::wire::post::pair();
    model.adopt_wire(link);
    model.adopt_post(post);
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
        link: link_end,
        outbox,
        lernie: Cli::new("yog-absent-lernie"),
        bl: Cli::new("yog-absent-bl"),
    }
}
