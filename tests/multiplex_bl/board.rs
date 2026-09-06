//! **The board reads the store the directory's operator reads** (DESIGN §16.2's
//! one-store-per-project invariant, ruling 1 on bl-262a) — the other half of the
//! parent binary's drive, and the half that was silently wrong.
//!
//! The sighting: a workspace was bound to a checkout the operator tracks with
//! `bl`, and the conversation aimed at it wrote a status report whose first
//! heading was *"The board is **empty**"* — eight `bl list` variants, all `[]`,
//! and a "Tracker health" section arguing the absence could be trusted. `bl -C
//! <repo> list` in the same directory printed four balls. Two stores at one
//! path: balls keys a clone on `(state home, invocation path)`, and the world
//! overrode the state home.
//!
//! So this asserts the two readers agree, over a store neither of them founded
//! for the occasion: the parent's rung has just `prime`d and `create`d in this
//! directory through the embedded `bl` — which, being the directory's own
//! store, is the operator's — and yog's own board read (`BlStore`, the §5.1 #2
//! effect the derivation worker holds) is asked for the same project and must
//! answer with the same ball.

use std::path::Path;

use balls::layout::Xdg;
use balls::reads::Catalog;
use yog::cli_outbound::Cli;
use yog::projects::runner::{BlRunner, BlStore};

/// `id` is the ball the parent's `create` sealed in this directory's store.
pub(crate) fn the_board_reads_the_directorys_own_store(proj: &Path, id: &str) {
    // The composed world, exactly as `Engine::boot` composes it from the
    // ambient environment this binary owns — `YOG_HOST_STATE` and all.
    let world = yog::world::compose(&yog::xdg::Env::from_env());
    let board = BlStore::new(world, Cli::new("bl"));
    // Listable means a founded clone at this path — the §3.5 orphaned-project
    // signal is the other answer, and it is not the one this store gives.
    let live = board.live(proj).unwrap();
    assert!(
        live.iter().any(|ball| ball.id == id),
        "the board answered {live:?}, which does not carry the ball the \
         directory's own store holds ({id})"
    );
    // And the detail read resolves through the same clone, so a seat that opens
    // the row is not answered out of a second store either.
    assert_eq!(
        board.detail(proj, id).map(|b| b.id),
        Some(id.to_owned()),
        "the detail read addresses the same store as the listing"
    );

    // **And the store both of them read is the OPERATOR's.** This is the
    // sentence the ruling is written in, so it is asserted against balls' own
    // layout built the way the operator's own shell builds it — `$HOME` and
    // `$XDG_STATE_HOME` off the ambient environment, with no world anywhere in
    // the derivation — rather than against a path this file spells.
    let operator = Xdg::with(
        Path::new(&std::env::var("HOME").unwrap()),
        None,
        Some(&std::env::var("XDG_STATE_HOME").unwrap()),
    );
    let theirs = Catalog::load(&operator.clone_dir(proj).store()).unwrap();
    assert!(
        theirs.get(id).is_some(),
        "the operator's own store does not hold {id}: yog and its owner are \
         addressing two stores at one path, which is the whole defect"
    );
}
