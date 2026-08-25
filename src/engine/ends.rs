//! **The four ends a face takes, minted in one act** (REMOTE §1.2, §3, §8.2,
//! §9.8) — split out of [`Engine::boot`](super::Engine::boot) at §12's
//! per-file budget, on the seam the boot's own prose already declared: each of
//! these four is minted *"for the read path's reason exactly"*, because both
//! ends of a loopback wire belong to this one assembly and a face takes the far
//! one or does not.
//!
//! That reason is the whole subject. The engine mints a channel pair, adopts
//! the frame's half into the model, and holds the other half until whichever
//! face asked for it — a window takes all four, a `yog serve` takes none, and
//! nothing else in `boot` has that shape. Taking is what makes them
//! exactly-once: an [`Option`] emptied by the take, and for the entry channels
//! a `Vec` emptied the same way, so a second window is refused by the ends
//! rather than by a gate holding the same fact twice.

use crate::AppModel;
use crate::xdg::Env;

/// Everything [`Engine`](super::Engine) holds on a face's behalf. Not `pub`:
/// the fields are reached through the engine's own hand-overs
/// (`engine/window.rs`), which is where "taken, not shared" is argued.
pub(crate) struct Ends {
    /// The asker's half of the window's read path (REMOTE §1.2, bl-ae05).
    /// Taken by [`asker`](super::Engine::asker) — a window takes it and a `yog
    /// serve` never does, which is the whole difference between the two faces.
    pub(crate) link: Option<crate::wire::link::LinkEnd>,
    /// **The same, once per §8.2 entry** (bl-670c): what each entry channel is,
    /// and the end its own asker answers on. Minted in one act with the model's
    /// half ([`channels::compose`](crate::wire::channels::compose)), so the
    /// pairing is the composition's rather than a join two lists keep in step.
    ///
    /// Taken by [`window_wire`](super::Engine::window_wire) and **not gated**:
    /// a second call already answers `None` on the loopback channel's own end,
    /// which is the one wire a window cannot be a window without, so a second
    /// gate here would be the same fact with a second home.
    pub(crate) entries: Vec<crate::wire::channels::EntryEnd>,
    /// The follow lane's engine-side end (REMOTE §3, bl-73e7) — one lane per
    /// engine, and a `yog serve` never takes it.
    pub(crate) tail: Option<crate::wire::lane::TailEnd>,
    /// The poster's half of the window's **act** path (REMOTE §9.8, bl-4841),
    /// taken by [`poster`](super::Engine::poster).
    pub(crate) post: Option<crate::wire::post::Outbox>,
}

impl Ends {
    /// Mint all four against `world`, adopting each frame-side half into
    /// `model` as it is made. One function because they are one act: a model
    /// that held three of the four halves would be a window with a surface
    /// nobody can answer.
    pub(crate) fn mint(world: &Env, model: &mut AppModel) -> Self {
        // The window's read path (REMOTE §1.2 as ruled, bl-ae05): the frame's
        // half goes to the model, the asker's half is held for whichever face
        // takes it. Minted unconditionally — a `yog serve` simply never asks
        // for the other end, and a model whose link nobody answers is the same
        // code path as a surface whose answer has not landed yet.
        let (link, link_end) = crate::wire::link::pair();
        // …and one channel per §8.2 entry beside it (bl-028a), composed here
        // because the engine is what holds the world. Zero entries is the local
        // channel alone — the general path with empty inputs. Since bl-670c the
        // composition hands back the far end of every entry channel too, so a
        // window can put an asker on each: one thread, one seat and one slice
        // per channel, which is what makes an unreachable entry cost only its
        // own rows.
        let (channels, entries) = crate::wire::channels::compose(world, link);
        model.adopt_wire(channels);
        // The follow lane's two ends (REMOTE §3, bl-73e7). The §7.2 live tail
        // used to be a follower thread on this engine writing into the model's
        // own RAM; it is a held wire read now, so what the engine mints is a
        // channel pair and the face takes the far end or does not.
        let (tail, tail_end) = crate::wire::lane::pair();
        model.adopt_tail(tail);
        // The act path's two ends (REMOTE §9.8, bl-4841).
        let (post, post_end) = crate::wire::post::pair();
        model.adopt_post(post);
        Self {
            link: Some(link_end),
            entries,
            tail: Some(tail_end),
            post: Some(post_end),
        }
    }
}
