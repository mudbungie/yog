//! The two config **rows** the §11 surfaces wear — *derived*, not painted. Both
//! are settings rows and so sit at the bottom of the conversation surface
//! ([`super::super::settings`], the settings-seat ruling, bl-2e18); the widgets
//! that show them are [`super::seat`], and the sentences are composed and tested
//! in [`crate::model_pick::header`]. What lives here is the derivation and the
//! memos that keep it off the repaint path.
//!
//! **The row is the selection** (bl-cd2a: the whole line becomes
//! `<provider> - <model>` and nothing else). So a row
//! is no longer a sentence to paint: it is the pair the two dropdowns show and
//! write, plus the hover and — only when this conversation has parted from the
//! workspace default — the drift clause beside them (bl-9786).
//!
//! - **The conversation's row** is derived from (agent tip, config tip): what
//!   the default assigns now, and what this conversation is frozen on.
//! - **The birth block's row** is derived from the config tip alone, because no
//!   agent exists yet to freeze against (bl-824e).
//!
//! Each derivation costs several `git show` spawns, which is the whole reason
//! the memos exist; both re-derive exactly when the oids they were taken from
//! move (§5.3 memoized derived snapshots), or when the role strip re-scopes the
//! row onto another role.

use super::ram::{BirthMemo, FrozenMemo};
use super::{BRANCH, PROVIDERS, PickerState};
use crate::config_edit::branch::config_file;
use crate::model_pick::{ConfigPoint, ConfigTip, ModelRow, birth_row, conversation_row, row_role};
use std::path::Path;

/// The conversation's model row, memoized: the pair the workspace default
/// assigns — what a pick here advances — with the freeze on hover and the drift
/// clause when the governing commit has parted from the tip. `None` only when
/// the conversation has no resolvable governing commit, which is the one case
/// with nothing to say.
///
/// Returns the governing short oid beside it: the pane scopes its write claim
/// with exactly that, so one derivation answers both questions.
pub(super) fn conversation_row_of(
    ws: &Path,
    tip_oid: &str,
    config_tip: Option<&ConfigTip>,
    picker: &mut PickerState,
) -> Option<(String, ModelRow)> {
    let key = (
        tip_oid.to_owned(),
        config_tip.map(|c| c.oid.clone()).unwrap_or_default(),
        row_role(picker.role.as_deref()),
    );
    if picker.frozen.as_ref().is_none_or(|m| m.key != key) {
        picker.frozen = memoize(ws, tip_oid, config_tip, key);
    }
    picker
        .frozen
        .as_ref()
        .map(|m| (m.short_oid.clone(), m.row.clone()))
}

/// Derive that row once per (agent tip, config-lineage tip, role) — the several
/// git spawns the memo exists to keep off the repaint path. A snapshot with no
/// config lineage yet is its own governing commit: nothing has moved, so the row
/// shows what the conversation runs on and claims no drift.
fn memoize(
    ws: &Path,
    tip_oid: &str,
    config_tip: Option<&ConfigTip>,
    key: (String, String, String),
) -> Option<FrozenMemo> {
    let gov = crate::config_edit::branch::governing_config(ws, tip_oid).ok()?;
    let governing = point(ws, gov.oid, gov.short_oid.clone());
    let tip = config_tip.map_or_else(
        || governing.clone(),
        |c| point(ws, c.oid.clone(), c.short_oid.clone()),
    );
    let row = conversation_row(&governing, &tip, &key.2);
    Some(FrozenMemo {
        key,
        row,
        short_oid: gov.short_oid,
    })
}

/// The §11 birth-config block's model row (bl-824e): the pair a conversation
/// started here right now would be born on, on the same two dropdowns the
/// conversation row wears — one picker, two seats, never two implementations.
/// `None` when the snapshot carries no config lineage yet: a workspace with no
/// config to fork paints nothing rather than a row about nothing.
pub(super) fn birth_row_of(
    ws: &Path,
    config_tip: Option<&ConfigTip>,
    picker: &mut PickerState,
) -> Option<ModelRow> {
    let tip = config_tip?;
    let key = (tip.oid.clone(), row_role(picker.role.as_deref()));
    if picker.birth.as_ref().is_none_or(|memo| memo.key != key) {
        let at = point(ws, tip.oid.clone(), tip.short_oid.clone());
        let row = birth_row(&at, &key.1, BRANCH);
        picker.birth = Some(BirthMemo { key, row });
    }
    picker.birth.as_ref().map(|memo| memo.row.clone())
}

/// One config commit's `providers.yaml`, read from its tree. An unreadable file
/// is the empty string — the oid beside it is still worth showing (§7.3).
fn point(ws: &Path, oid: String, short_oid: String) -> ConfigPoint {
    let raw = config_file(ws, &oid, PROVIDERS).unwrap_or_default();
    ConfigPoint {
        oid,
        short_oid,
        providers_yaml: String::from_utf8_lossy(&raw).into_owned(),
    }
}
