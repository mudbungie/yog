//! The populated fixture world the acceptance tests render (§11): one
//! workspace symlinked under the lernie root with a transcript, a settled
//! auth-failed step with tool i/o, and an inbox deposit — every inspector
//! surface has something to paint. Split from the smoke test for §12's budget.

use crate::cli_outbound::Cli;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

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
/// sequence, so a mint regressing to repeats fails every run, not once in the
/// pool's size.
/// The pool is **lernie's** since bl-cd38 (yog deleted its own list and draws
/// through [`lernie::mint`]), so this is also the seam check on that
/// consumption: a corpus change moves it, and fails in `mint_seed.rs` naming
/// the cause instead of as a needle collision elsewhere in the suite. It has
/// moved once, and that is the check working: lernie's bl-79a2 widened the draw
/// from one lowercase word to an ordered PascalCase **pair** (yog bl-0219's
/// consume), so these three re-pinned words are two-word names now.
pub(super) const MINTED: [&str; 3] = ["CourtyardRooftop", "AxolotlHeadland", "XylophoneAzure"];

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

/// The crowd [`Roster::Crowded`] lays on top of the shipped world — its own
/// file per §12's budget.
pub(in crate::shell::acceptance) mod crowd;

/// The rows whose **width** outgrows the column, laid on the shipped world —
/// [`crowd`]'s twin on the other axis, its own file per §12's budget.
pub(in crate::shell::acceptance) mod wide;

/// The follow lane's stand-in (bl-73e7) — its own file at §12's budget; the
/// seam is its own doc's.
mod follow;
/// **The wire standing behind the world** — the questions and the acts the
/// frame's own channel ends carry, answered through the chokepoints the real
/// listener reaches; its own file at §12's budget, on the seam between the
/// fixture a test mutates and the engine that answers what it said.
mod wire;
/// The world a test drives — its own file per §12's budget.
mod world;
pub(in crate::shell::acceptance) use world::World;

/// **How a world is assembled** — the roster axis, the four named builders and
/// the procedure behind them; its own file per §12's budget, on the seam this
/// file already had between the facts a test names and the assembly that
/// produces one.
mod build;
pub(super) use build::{world, world_crowded, world_empty, world_titled};
