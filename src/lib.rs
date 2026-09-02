//! **The yog server** — the standalone engine of the four-component split
//! (REMOTE §12): holder of the world, the balls and the conversations, with no
//! UI and no local execution.
//!
//! One world, one engine. The binary boots [`engine::Engine`], which derives
//! every workspace from disk, answers the §8.5 control [`boundary`] over the
//! REMOTE §9.5 [`wire`], and drives `litany`/`bl`/`bz` as children of the
//! nested [`world`]. Every seat — the `lernie` window, an android client, a
//! `yog gesture` from an agent's own bash — is a client of that boundary and
//! of nothing else; **yog paints nothing** (bl-7942 severed the egui face into
//! its own crate, REMOTE §8).
//!
//! **`docs/DESIGN.md` is the authority** for the state inventory, the attention
//! model, and the module map; the module docs below stay terse and defer to it.
//! The shape in brief: [`app`] holds the [`AppModel`] and its per-tick
//! derivation over the pure view-model modules — [`git_tree`], the [`nav`]
//! roster, [`attention`], [`projects`]/[`binding`], [`start`], [`ui_state`],
//! and the inspector projections ([`transcript`], [`steps_view`],
//! [`inboxview`], [`budgets`]) — and [`boundary`] is the one surface every act
//! and every read crosses, serialized by each module's own `wire`.
//! [`cli_outbound`] execs the binaries; [`actions`] holds the
//! message/stop/scan/close/unclaim/create/update verb surface; [`delete`] the
//! §3.6 unmaking; [`badge`] is what is left of the palette — the words a
//! derived row says, never a colour.
//!
//! The crate root is deliberately declaration-only. A root carrying a
//! coverable `impl` or `fn` accrues an llvm-cov phantom uncovered region on
//! its header line each time the `pub mod` list above it grows and shifts
//! byte offsets (this cost 99.90% coverage after the Y2/Y7/Y15 folds). With
//! all coverable code in submodules, new `pub mod` lines have no root-level
//! line to mis-attribute, so coverage stays at 100% as modules land.

pub mod actions;
/// The §6 attention strip escalated to the desktop (bl-e160) — what a decision
/// queue row becomes when the window is buried, and the one spawn that says it.
pub mod alert;
pub mod app;
pub mod attention;
/// What a derived row **says** about a fact — the badge vocabulary that is
/// all a server keeps of the §11 palette (bl-7942).
pub mod badge;
pub mod binding;
pub mod board;
pub mod boundary;
pub mod budgets;
/// The embedded brazen host (§16.7 W10) — internal, not library surface.
pub(crate) mod bz_host;
pub mod cli_outbound;
pub mod config_edit;
/// How full a conversation's context is (§5.1 #35) — the latest step's prompt
/// against the window `models.yaml` declares. Fullness, not spend.
pub mod context;
/// The capability control (§8.6, VISION §4.11) — the adjudicator litany's
/// tool-control seam consults before every granted tool invocation.
pub mod control;
pub mod delete;
/// Where to cut a string that will not fit (QUALITY G1, L4) — one rule, cut
/// where the information is not. Machine strings only; prose keeps its head.
pub(crate) mod elide;
/// The one assembly a bare `yog` boots (VISION §5 V5) — model, worker, bridge,
/// gesture consumer, monitor sentry, fleet pilot and the wire listener.
pub mod engine;
/// The VISION §4.10 mutating fan — N isolated candidate attempts over one
/// delivery obligation, materialized through balls' attempt capability.
pub mod fan;
pub mod files_view;
/// Named deterministic world states a client harness can dial and render
/// (bl-8741) — the fixture roster, its writer and the `yog fixture` verb.
pub mod fixture;
/// The VISION §4.3 armed loop — off until the operator arms it per workspace.
pub mod fleet;
pub mod fork;
pub mod fs_watcher;
pub mod git_env;
pub mod git_tree;
pub mod inboxview;
pub mod login;
pub mod model_pick;
pub mod monitor;
pub mod multiplex;
pub mod names;
pub mod naming;
pub mod nav;
pub mod opslog;
pub mod projects;
pub mod rail;
/// The REMOTE §4 client registry (bl-8bbc): who participates in which
/// workspace, and each client's own per-seat home.
pub mod registry;
/// The §3.9 attempt science projection (VISION §4.10 item 7): one derived row
/// per delivery attempt, joining frozen inputs, refs, usage and outcome.
pub mod science;
/// I3's scratch temp (§2, §5.2): its one naming, and the startup sweep of
/// leftovers — internal, not library surface.
pub(crate) mod scratch;
pub mod search;
pub mod spend;
pub mod start;
pub mod state;
pub mod steps_view;
/// yog's litany tool injection (REMOTE §5, bl-c907) — the `clients` tool, the
/// agent's durable loaded set, and the router the executor consults.
pub mod tool_host;
pub mod transcript;
pub mod ui_state;
pub mod watch;
/// The client/server wire (REMOTE §9.5, bl-b6fa) — the engine's mTLS listener,
/// a seat's transport, and the framing between them.
pub mod wire;
pub mod workdiff;
pub mod world;
pub mod xdg;

pub use app::{AppModel, Args, Roots};

#[cfg(test)]
pub(crate) mod test_support;
