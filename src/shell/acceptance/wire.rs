//! **A wire the acceptance world answers itself** (REMOTE §1.2 and its
//! read-path residual; bl-adcb).
//!
//! A migrated surface paints a decoded `Reply`, so a test that paints one paints
//! nothing at all unless something answers. The real answerer is the engine's
//! listener reached over loopback mTLS, which an acceptance world has no
//! business standing up: it mints no certificate, binds no port and holds no
//! [`Engine`](crate::engine::Engine) — it is a model, a deriver and a shell
//! state over a temp dir.
//!
//! So this stands in for the *transport* and nothing else. The questions are the
//! frame's own, taken off the [`LinkEnd`](crate::wire::link::LinkEnd) exactly as
//! the [`Asker`](crate::wire::asker::Asker) takes them; each is decoded by the
//! one [`codec`] and answered by [`AppModel::answer`], which is the same
//! chokepoint `ConsumerCtx::answer_as` reaches over the socket. **No second
//! dispatch implementation is added** — REMOTE §11's rejection is about a face,
//! and this is a wire with the socket taken out for a test that cannot have one.
//!
//! Two things it deliberately does not reproduce, both stated rather than
//! implied: the answers are **unscoped** (there is no registration to narrow
//! them against, and the fixture registers no client), and they are **prompt**
//! rather than one [`ASK_PERIOD`](crate::wire::asker::ASK_PERIOD) late — the
//! delay is time, not shape, and [`wired`] pays the *shape* in full by painting
//! the settle-then-render dance out.

use super::super::ShellState;
use super::fixture::World;
use crate::AppModel;
use crate::boundary::{Gesture, codec};
use crate::cli_outbound::Cli;
use crate::wire::link::LinkEnd;

/// Paint `f` with the wire answered — the whole settle-then-render dance, which
/// is three passes and cannot be fewer.
///
/// A frame *declares* its standing questions while it paints and hands them over
/// at [`AppModel::refresh`], so the first pass is what asks; the answers land in
/// a channel the *next* settle drains; and a settle whose frame declared nothing
/// would drop them as no longer standing. Hence: paint (ask), settle, answer,
/// paint (re-declare), settle (land), paint — and the last painting is the one
/// returned. Exactly what the window does over half a second, minus the waiting.
pub(super) fn wired(
    world: &mut World,
    paint: &mut dyn FnMut(&mut AppModel, &mut ShellState) -> String,
) -> String {
    let (link, mut end) = crate::wire::link::pair();
    world.model.adopt_wire(link);
    let _ = paint(&mut world.model, &mut world.state);
    world.model.refresh();
    pump(&mut world.model, &mut end);
    let _ = paint(&mut world.model, &mut world.state);
    world.model.refresh();
    paint(&mut world.model, &mut world.state)
}

/// Answer every question standing on `end`, through the boundary chokepoint the
/// wire's own answerer runs. A gesture that is not an ask cannot reach here —
/// [`Link::ask`](crate::wire::link::Link::ask) is the only writer of the
/// standing set — so one is answered with the refusal that says so rather than
/// silently dropped.
fn pump(model: &mut AppModel, end: &mut LinkEnd) {
    let deps = model.boundary_deps(&Cli::new("lernie"), &Cli::new("bl"));
    let now = super::super::clock::now_unix();
    for question in end.standing() {
        let landed = match codec::decode(&question) {
            Ok(Gesture::Ask(query)) => model.answer(&deps, &query, now),
            Ok(Gesture::Act(_)) => Err("the read path carries no acts".to_owned()),
            Err(said) => Err(said),
        };
        end.publish(&question, landed);
    }
}

/// Arm a §4.3 loop on the fixture's own state root and fold it in — the one
/// board fact reachable without a `bl` store, so a board witness needs no
/// project, no ball and no fork of `bl`. The label the fold paints is
/// [`crate::fleet::Facts::label`]'s.
fn arm(world: &mut World) {
    let root = world.model.state_root().to_path_buf();
    std::fs::write(
        root.join(crate::app::cadence::CADENCE_YAML),
        "fleet:\n  /ws/a:\n    project: /dev/yog\n    cap: 3\n",
    )
    .expect("the fixture world takes a cadence file");
    world
        .model
        .dirty_handle()
        .mark_all([(root, crate::watch::Mark::Watch)]);
    world.converge();
}

/// Paint the §11 balls fold, wired or not — the two directions of the same
/// witness, so the assertion below is about the *wire* and not about a fixture
/// that happens to hold no balls.
fn painted_board(world: &mut World, answered: bool) -> String {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let mut paint = |model: &mut AppModel, state: &mut ShellState| {
        crate::paint_probe::paint(|ui| {
            super::super::board::board(ui, model, state, &lernie, &bl);
        })
    };
    if answered {
        wired(world, &mut paint)
    } else {
        paint(&mut world.model, &mut world.state)
    }
}

/// **The board is painted from a `Reply`, in both directions** (REMOTE §1.2,
/// bl-adcb). An armed loop is a board fact and nothing else's, so its label on
/// screen can only have come through `Query::Board` — and with nobody answering,
/// the same world with the same armed loop paints no board at all.
///
/// The negative half is the half that matters: without it, a fold that had
/// quietly kept deriving in process would pass the positive assertion unchanged.
#[test]
fn the_balls_fold_paints_the_board_the_wire_answers_and_nothing_without_one() {
    let mut world = super::fixture::world();
    arm(&mut world);
    let unanswered = painted_board(&mut world, false);
    assert!(
        !unanswered.contains("fleet"),
        "with nothing answering, the fold has no board to paint:\n{unanswered}"
    );
    let answered = painted_board(&mut world, true);
    assert!(
        answered.contains("fleet"),
        "and the armed loop reaches the glass once a reply lands:\n{answered}"
    );
    assert!(
        answered.contains("3 drones"),
        "the row's own figures ride the same reply:\n{answered}"
    );
}
