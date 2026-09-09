//! **The wire version** (REMOTE §3, §3.2): what the integer each end writes
//! in its preface MEANS, and the record of every time it moved. Split off
//! [`super`] at §12's cap.
//!
//! **Since bl-e598 the number is a MAJOR.** It moves only on a breaking
//! change — a field removed or re-typed, a frame's meaning changed under a
//! spelling still in use, a field the engine newly REQUIRES on a request —
//! and every additive change ships without it: a new field a reader may
//! ignore, a new word in a vocabulary, a new op, a new reply kind. Those are
//! **editions**, stamped per field path in `corpus/shapes.json` by
//! `make corpus` and stated by the engine in its hello (REMOTE §3.2), so a
//! seat discovers what an engine can spell instead of being refused for a
//! field it would have ignored. The hello is still strict equality on THIS
//! number, fail-closed, with no negotiation; what changed is what a bump is
//! for. It was 13 → 18 in one week, and four of those five carried nothing
//! but additions.
//!
//! **The number this build SPEAKS is not here, and is in no Rust file at
//! all**: the repo-root `PROTOCOL` file states it and `build.rs` compiles that
//! into the constant included below (bl-3e57). Four repositories FETCH it out
//! of trees they do not build, and a Rust path is not a stable address for
//! that — bl-94a5 split `src/wire/hello.rs` in two and every consumer's release
//! gate silently stopped being able to read it. **The version is never
//! restated anywhere else** (REMOTE §3): a second home for it went five
//! versions stale before anybody noticed.
//!
//! **A second integer IS declared here** (bl-9ced): the newest major yog has
//! PUBLISHED. Nothing outside this tree fetches it — its one reader is the
//! corpus ledger, an ordinary unit test — so it needs no address of its own,
//! and it is stated rather than derived because that reader runs on a bare
//! checkout.
//!
//! **The exact-era ledger is closed and lives in [`exact`].** Versions 4 → 18
//! were taken one integer per wire-visible change; their entries moved there
//! whole when this file crossed the cap, which is the seam the ledger named
//! for itself. From 19 on, an addition's record is its edition stamp in
//! `corpus/shapes.json` and the ball that landed it; a MAJOR bump is a design
//! ball with a migration note (REMOTE §3.2), and its entry is written here.
//!
//! 18 → 19 (bl-e598): **the last bump of the old kind and the first major.**
//! Nothing on the wire moved. What moved is what the integer says: two ends
//! stating 19 agree on every path stamped at or below the floor
//! (`corpus/shapes.json`: 18) and tolerate each other past it — unknown key
//! ignored, absent post-floor key defaulted, unknown word rendered as the
//! unknown it is. That is a change to what a spelling already in use is taken
//! to say — the preface's own — and under the old rule it bumps; under the
//! new rule it is the one kind of change that still does. Taking 19 rather
//! than re-reading 18 is the transition's safety: under bl-bca2's hold no
//! engine speaking 19 publishes until every consumer main carries 19, and a
//! consumer carries 19 by the same ball that makes its reader grows-only. The
//! migration that produced the per-path record stamped every path of a shape
//! at that shape's last move — the finer history is [`exact`]'s prose — and
//! nothing mechanical reads a stamp below the floor.

// The constant itself is GENERATED, not declared: `build.rs` reads the
// repo-root `PROTOCOL` file — the number's one file-shaped home, at the one
// address a module split cannot move (bl-3e57) — and writes this item.
include!(concat!(env!("OUT_DIR"), "/protocol.rs"));

/// The closed record of the exact era, 4 → 18: prose only.
pub(crate) mod exact;

/// **The newest `PROTOCOL` yog has PUBLISHED**, and therefore the newest a
/// peer out there can be speaking (REMOTE §3, §3.2).
///
/// It exists for one reader: the corpus ledger's rule that *a field or a
/// spelling is removed or re-typed only at a MAJOR bump* — `PROTOCOL` above
/// this number is a bump in flight, and licenses the change (beside a
/// deprecation, `corpus::DEPRECATED`). Under the exact era it judged every
/// moved signature against this floor instead of against the version the
/// record was last generated at (bl-9ced), so a wave of lanes could share one
/// unreleased number; under the major it is read far more rarely, since a
/// gain never consults it at all.
///
/// **How it moves.** A lane that raises `PROTOCOL` leaves this alone unless
/// the number it is raising FROM has shipped; then it raises this to that
/// number in the same edit. `18 → 19` here is `PROTOCOL_PUBLISHED = 17`,
/// because 18 never shipped as an engine: v0.0.52 spoke 17.
///
/// **What it does not do.** It is not a compatibility window and the handshake
/// never reads it: the wire is still fail-closed on `PROTOCOL` alone, with no
/// negotiation. And it is a stated fact rather than a derived one — the
/// derivation exists (`scripts/protocol-gate.sh read` over yog's newest
/// `v<x.y.z>` tag is what `.github/workflows/release-automerge.yml` already
/// does), but it needs git and a network, and the ledger's gate is an ordinary
/// unit test that must run on a bare checkout. The residual is that a lane
/// which raises `PROTOCOL` after a release and forgets to raise this one is
/// under-strict for one wave, and the pair is read together in one file so the
/// forgetting is visible where the raise is made.
pub const PROTOCOL_PUBLISHED: u32 = 17;
