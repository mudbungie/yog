//! **The windowless face, whole** (DESIGN §8.5, VISION §4.8, REMOTE §8):
//! [`Engine::serve`], which is what `yog serve` *is*.
//!
//! It is here and not in `main.rs` for that file's own standing reason
//! (bl-f6fe): `main.rs` is coverage-excluded, so anything that lives there is
//! free to drift from the face beside it and no test can notice. What kept
//! this arm there was that it never returned — it parked forever — and bl-269a
//! is exactly what dissolved that: with a stop the loop ends, so the whole face
//! is an ordinary function a test drives to completion.
//!
//! The window's arm is still `main.rs`'s, because eframe's loop is, and the two
//! remain one call to [`Engine::boot`] with a different repaint hook — VISION
//! V5.4's "nothing here is a second implementation".

use super::Engine;
use crate::ui_state::SystemClock;
use crate::watch::NoRepaint;
use crate::xdg::Env;
use std::sync::Arc;

impl Engine {
    /// Run the engine with no window until a §8.5 stop is asked for. Converge
    /// the world's tool shims (§8.4, bl-44a5 — the world `PATH` names that dir
    /// unconditionally, so every face that hands the world out seeds it), catch
    /// the signal *before* the engine exists so no boot window swallows one,
    /// then park. Returns when the engine has stopped and joined every thread
    /// it spawned; the caller's only remaining act is to exit.
    pub fn serve(ambient: &Env, overrides: &[(String, String)]) {
        crate::world::tools::seed(ambient);
        super::stop::catch();
        Self::boot(
            &crate::world::compose(ambient),
            overrides,
            None,
            Arc::new(SystemClock),
            Arc::new(NoRepaint),
        )
        .park_until_stopped();
    }
}

#[cfg(test)]
mod tests;
