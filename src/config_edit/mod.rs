//! Config-editing view-models (DESIGN §9): load → edit a RAM draft → Apply =
//! stage → (validate, where a validator exists) → hash-guard → atomic rename.
//!
//! One discipline across the file-editing surfaces. The shared write pipeline
//! — the [`FileIo`] seam, temp-in-dir staging, the concurrent-edit hash guard
//! and the atomic rename — lives in [`pipeline`], the single source of truth
//! for how an edit reaches disk (§9). Each is a thin view-model over it:
//! [`brazen`] (§9.1, raw TOML gated by `bz`) and [`lernie_global`] (§9.2, raw
//! `models.yaml`/`workflows/*.yaml` with no validator). Every editor is pure
//! over the injected seam, so Linux tarpaulin drives each transition with a
//! fake and no real disk.
//!
//! [`form`] is how those drafts are *edited* since §9.5: the settings a file
//! declares, each as its own typed control over the same draft text — the file
//! stays the single fact, the pane is a typed view of it, and the rewrite is
//! the §9.4 anchored grammar rather than a second reader of the file's shape.
//!
//! [`branch`] is the third surface (§9.3): the per-workspace config *branches*
//! (`config/*` git refs), not files on disk. Its browse half reads through the
//! env-scrubbed `git_tree::cmd` wrapper and derives each agent's governing
//! config; the `$EDITOR`-driven edit half ([`branch::edit`], Y21) is the only
//! lawful writer of `config/*` and so bypasses the [`FileIo`] pipeline
//! entirely — it stages drafts and drives `lernie config`, whose `$EDITOR`
//! callback re-enters this binary in [`apply`] shim mode to copy them over the
//! checkout. lernie commits.

mod draft;
mod effects;
mod pipeline;

pub mod apply;
pub mod branch;
pub mod brazen;
/// Where a **config-kind** failure is fixed (bl-dd7f): the classifier and the
/// sentence the §7.3 banner pairs with a route to the §9.1 editor — beside the
/// editor it points at, exactly as §8.3's auth classifier sits with Login.
pub mod fault;
pub mod form;
pub mod lernie_global;

pub(crate) use draft::Draft;
pub use effects::RealFileIo;
pub use pipeline::FileIo;
pub(crate) use pipeline::{Commit, is_pristine, load_snapshot, stage};
