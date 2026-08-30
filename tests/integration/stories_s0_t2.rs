//! STORIES **S0-T2** seeded-skip (the seed half owned by Z7): a world whose
//! `models.yaml` marker is present spawns no `litany prime` — the seeded world
//! is the general path with the seed present, not a bootstrap branch (STORIES
//! S0.2, DESIGN §16.6 W3, §3.4, §15 M6 Z7). Driven straight through the current
//! `world::seed::ensure_seeded` dispatch API; no fake binary is even reachable.

#![allow(clippy::unwrap_used)]

use tempfile::tempdir;
use yog::cli_outbound::Cli;
use yog::opslog;
use yog::world::{layout_under, seed};

/// STORIES **S0-T2** seed-skip half.
#[test]
fn s0_t2_seeded_world_skips_prime() {
    let state = tempdir().unwrap();
    let yog_data = tempdir().unwrap();
    let layout = layout_under(yog_data.path());
    // The founded marker: `<LITANY_HOME>/models.yaml` (§16.6 W3 / litany §4.2).
    std::fs::create_dir_all(&layout.litany).unwrap();
    std::fs::write(layout.litany.join("models.yaml"), b"models: {}\n").unwrap();

    // A binary that could never run — proving the skip never spawns (an actual
    // spawn would surface as an `Io` error here).
    let litany = Cli::new("/definitely/not/a/real/litany");
    let primed = seed::ensure_seeded(
        &litany,
        state.path(),
        "T0",
        &layout,
        yog::opslog::Origin::Conversation,
    )
    .unwrap();

    assert!(!primed, "a seeded world reports nothing primed");
    assert!(
        opslog::tail(state.path(), 16).is_empty(),
        "the skip spawns nothing and logs nothing (§16.6 W3)"
    );
}
