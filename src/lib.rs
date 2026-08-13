//! egui/eframe frontend for the lernie agent harness — the balls-oriented
//! manager for lernie loops.
//!
//! This crate is the desktop UI: a stateless renderer over on-disk state that
//! issues user actions as `lernie` and `bl` subcommand invocations. Every
//! render is a pure function of filesystem state at the current tick, and the
//! public view-model API is reentrant, so a future `lernie-ui-web` runs
//! concurrently against the same repo without coordination.
//!
//! **`docs/DESIGN.md` is the authority** for the state inventory, the attention
//! model, the module map, and line budgets; the module docs below stay terse
//! and defer to it. The shape in brief: pure view-model modules (no egui) — the
//! per-tick [`git_tree`], the [`nav`] roster, [`attention`], [`projects`]/
//! [`binding`], [`start`], [`ui_state`], the inspector VMs ([`transcript`],
//! [`steps_view`], [`jsonview`], [`inboxview`], [`budgets`]) composed by the
//! tested [`inspector`] tab dispatch, [`composer`] (the §11 inbox-composer's
//! queue/fold-line/snap derivations, bl-929d), [`config_edit`] — fronted by the thin
//! egui glue in [`shell`] (§11). [`app`] re-exports `Args`, `Roots`,
//! `Focus`, and the multi-workspace [`AppModel`]; [`keymap`] is the pure §11
//! key → intent table; [`cli_outbound`] execs the binaries; [`actions`] holds
//! the message/stop/scan/close/unclaim/create/update verb surface; [`delete`]
//! the §3.6 unmaking (its gate, its typed-name confirmation, its plan); [`theme`]
//! is the congeries palette — the single colour/visuals authority (§11).
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
pub mod binding;
pub mod board;
pub mod boundary;
pub mod budgets;
/// The embedded brazen host (§16.7 W10) — internal, not library surface.
pub(crate) mod bz_host;
pub mod cli_outbound;
pub mod composer;
pub mod config_edit;
/// How full a conversation's context is (§5.1 #35) — the latest step's prompt
/// against the window `models.yaml` declares. Fullness, not spend.
pub mod context;
/// The capability control (§8.6, VISION §4.11) — the adjudicator lernie's
/// tool-control seam consults before every granted tool invocation.
pub mod control;
pub mod delete;
/// The one assembly both faces boot (VISION §5 V5) — model, worker, bridge,
/// gesture consumer; the window and `yog headless` differ only in what they add
/// beside it.
pub mod engine;
pub mod files_view;
/// The VISION §4.3 armed loop — off until the operator arms it per workspace.
pub mod fleet;
pub mod fork;
pub mod fs_watcher;
pub mod git_env;
pub mod git_tree;
pub mod inboxview;
pub mod inspector;
pub mod jsonview;
pub mod keymap;
pub mod login;
pub mod model_pick;
pub mod monitor;
pub mod multiplex;
pub mod names;
pub mod nav;
pub mod opslog;
pub mod projects;
pub mod rail;
pub mod search;
pub mod shell;
pub mod spend;
pub mod start;
pub mod state;
pub mod steps_view;
/// The §11 tail idiom every tail-anchored view shows through — internal, not
/// library surface.
pub(crate) mod tail;
pub mod theme;
pub mod transcript;
pub mod ui_state;
pub mod watch;
pub mod workdiff;
pub mod world;
pub mod xdg;

pub use app::{AppModel, Args, Focus, Roots};

pub(crate) mod layout;
#[cfg(test)]
pub(crate) mod paint_probe;
#[cfg(test)]
pub(crate) mod test_support;
