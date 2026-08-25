//! **Waking the face** — the injected repaint effect and its three impls. Split
//! from the watch registry at §12's budget on the seam the subject itself
//! draws: nothing here watches anything, and nothing in [`super`] knows what a
//! face is.
//!
//! It is a value rather than a second call site because
//! [`Engine`](crate::engine::Engine) is the *one* assembly both faces boot, so
//! the face's whole difference — an event loop to wake, or none — travels as
//! this (§8.5, VISION §5 V5.4).

use std::sync::Arc;

/// Effect: request an egui repaint. Injected on the LockProbe template
/// (DESIGN §12) so the derivation worker is exercised headlessly with a
/// counting double; [`EguiRepaint`] is the production impl.
pub trait Repaint: Send + Sync {
    fn request(&self);
}

/// A shared hook is a hook. What makes this load-bearing rather than plumbing:
/// [`Engine`](crate::engine::Engine) is the *one* assembly both faces boot, so
/// the face's difference — an event loop to wake, or none — has to travel as a
/// value rather than as a second call site (§8.5, VISION §5 V5.4). It is
/// **shared** rather than owned because more than one engine thread wakes the
/// face: the derivation worker when a snapshot lands, and the §7.2 live-tail
/// follower when characters do.
impl Repaint for Arc<dyn Repaint> {
    fn request(&self) {
        (**self).request();
    }
}

/// Production [`Repaint`]: wakes the egui event loop.
pub struct EguiRepaint(pub egui::Context);

impl Repaint for EguiRepaint {
    fn request(&self) {
        self.0.request_repaint();
    }
}

/// The windowless [`Repaint`] (§8.5): `yog serve` has no event loop to
/// wake — a published snapshot is simply the next thing the gesture consumer
/// reads. Doing nothing is the whole contract.
pub struct NoRepaint;

impl Repaint for NoRepaint {
    fn request(&self) {}
}
