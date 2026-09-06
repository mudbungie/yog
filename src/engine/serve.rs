//! **The server, whole** (DESIGN §8.5, VISION §4.8, REMOTE §8, §12):
//! [`Engine::serve`], which is what a bare `yog` *is*.
//!
//! It is here and not in `main.rs` for that file's own standing reason
//! (bl-f6fe): `main.rs` is coverage-excluded, so anything that lives there is
//! free to drift and no test can notice. What kept this arm there was that it
//! never returned — it parked forever — and bl-269a is exactly what dissolved
//! that: with a stop the loop ends, so the whole face is an ordinary function a
//! test drives to completion.
//!
//! There is one face now (bl-7942), so V5.4's *"nothing here is a second
//! implementation"* has nothing left to hold apart: `main.rs` is one call into
//! this one, which is exactly what a coverage-excluded file should hold of a
//! face.

use super::Engine;
use crate::ui_state::SystemClock;
use crate::xdg::Env;
use std::sync::Arc;

/// What a `yog` that could not become the engine exits with (bl-1d9b) — the
/// two boot refusals' one code. It is a *usage* class, `sysexits.h`'s `EX_*`
/// range like [`world::seat::REFUSED`](crate::world::seat::REFUSED), and it is
/// deliberately not `0`: a supervisor restarting a unit, a script chaining a
/// launch, and a human reading `echo $?` all learn the same thing from it, and
/// none of them learned anything before.
pub const NO_ENGINE: i32 = 69;

impl Engine {
    /// Run the engine until a §8.5 stop is asked for, and answer the process's
    /// exit code. Converge the world's tool shims (§8.4, bl-44a5 — the world
    /// `PATH` names that dir unconditionally, so every face that hands the
    /// world out seeds it), catch the signal *before* the engine exists so a
    /// stop during boot is not lost, then park. Returns when the engine has
    /// stopped and joined every thread it spawned; the caller's only remaining
    /// act is to exit with what this answers.
    ///
    /// **A boot that refuses exits non-zero, naming what could not be had**
    /// (bl-1d9b): a world another engine already holds, or an address this box
    /// cannot listen on. The saying is here rather than in
    /// [`boot`](Engine::boot) because one face owns stderr and the exit code
    /// together — the same argument bl-dc14 made for the wire's refusal when
    /// there were two faces to make it to.
    pub fn serve(ambient: &Env, overrides: &[(String, String)]) -> i32 {
        crate::world::tools::seed(ambient);
        super::stop::catch();
        match Self::boot(
            &crate::world::compose(ambient),
            overrides,
            Arc::new(SystemClock),
        ) {
            Ok(engine) => {
                engine.park_until_stopped();
                0
            }
            Err(refusal) => {
                eprintln!("yog: {refusal}");
                NO_ENGINE
            }
        }
    }
}

#[cfg(test)]
mod tests;
