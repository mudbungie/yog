//! **How a fixture world is assembled** — the roster axis the builders differ
//! on, the four named builders, and the one procedure that lays a world on
//! disk and boots a window over it. Split from [`super::fixture`] at §12's
//! budget on the seam that file already had: the pinned facts and fake
//! binaries a test *names* are one subject, the assembly that produces a world
//! is another.

use super::super::super::ShellState;
use super::{MINT_SEED, World, crowd};
use crate::AppModel;
use crate::app::Roots;
use crate::cli_outbound::Cli;
use crate::git_tree::tests::fixture::Fixture;
use crate::model_pick::PROVIDERS;
use crate::projects::runner::BlStore;
use crate::ui_state::SystemClock;
use crate::xdg::Env;
use std::sync::Arc;
use tempfile::tempdir;

/// **Whether the fixture's world has a workspace in it** — the one axis these
/// builders differ on. Named rather than spelled as a bool because the state it
/// distinguishes was, until bl-37bf, not reachable at all.
#[derive(PartialEq, Eq)]
pub(in crate::shell::acceptance) enum Roster {
    /// One workspace, enumerated and focused.
    One,
    /// **None** — an operator's very first launch, and the only state in which
    /// `shell::bootstrap` paints: `shell::workspace::center` hands over to it
    /// exactly when `focused_workspace()` is `None`.
    Empty,
    /// **A list taller than any window the audit renders** — the state every
    /// §11 rule-5/6 claim about the navigator column is a claim *about*, and
    /// the state no fixture in this suite could reach before bl-86a5. Its
    /// wall is one of yog's own (§3.1) rather than foreign, because the §3.6
    /// workspace delete exists nowhere else and its census is the other half
    /// of what this roster is for. [`crowd`] holds its bytes.
    Crowded,
}

pub(in crate::shell::acceptance) fn world_titled(title: &str) -> World {
    build_world(title, &Roster::One)
}

pub(in crate::shell::acceptance) fn world() -> World {
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
pub(in crate::shell::acceptance) fn world_empty() -> World {
    build_world("hello", &Roster::Empty)
}

/// **A world whose conversation list outgrows the column** ([`Roster::Crowded`])
/// — the fixture every §11 navigator-budget beat and both §3.6 census beats are
/// driven over (bl-86a5). Identical to [`world`] in every other respect, so a
/// beat that reddens under it and passes under [`world`] has found a defect of
/// *length*, which is the only axis these two fixtures differ on.
pub(in crate::shell::acceptance) fn world_crowded() -> World {
    build_world("hello", &Roster::Crowded)
}

pub(in crate::shell::acceptance) fn build_world(title: &str, roster: &Roster) -> World {
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
    // name in their own `request.json` (`write_step_record`). Written in the
    // block's legacy shape, which the read still takes whole (bl-3ffa).
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
    if *roster == Roster::Crowded {
        crowd::seat(&fx);
    }
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
    // **The crowded world's wall is one of yog's own** (§3.1's named root),
    // where every other fixture's is foreign. Not decoration: §3.6 offers the
    // workspace delete on a named wall and nowhere else, so a foreign wall's
    // confirmation dialog closes on the frame it opens
    // (`acceptance::focus::a_dismissed_modal_hands_the_keyboard_back` reads
    // that as its own door) — and half of what this roster exists for is the
    // dialog's own census.
    let names_root = crate::binding::names_root(&yog_data);
    let ws = if *roster == Roster::Crowded {
        std::fs::create_dir_all(&names_root).unwrap();
        names_root.join(crowd::WALL)
    } else {
        lernie_workspaces.join("ws")
    };
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
        (*roster != Roster::Empty).then(|| ws.clone()),
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
    // The follow lane's ends, on the same terms (bl-73e7): a world that took
    // none would be a window with no lane, which is a true reading and no
    // witness at all for the one this ball exists to prove.
    let (tail, tail_end) = crate::wire::lane::pair();
    model.adopt_wire(crate::wire::channels::Channels::of(link));
    model.adopt_post(post);
    model.adopt_tail(tail);
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
        tail: tail_end,
        followed: None,
        // Deliberately absent substrate, named by **absolute** path (bl-f558):
        // a test that never spawns one wants a binary that cannot be found,
        // and a bare name is not that — the world's `PATH` is fronted by the
        // tools dir (§16.2), so a relative name resolves against it. The whole
        // suite spells these the same way, and `world::tools::ensure_shim`
        // refuses to persist a non-absolute target for that reason.
        lernie: Cli::new("/yog-absent-lernie"),
        bl: Cli::new("/yog-absent-bl"),
    }
}
