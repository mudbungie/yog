//! The picker's read-back of the assignment it is about to change (§9.4,
//! bl-53be): brazen's provider rows joined onto `providers.yaml`'s roles, into
//! "why can this role not be fired?". Coverage-excluded glue — the judgement
//! itself is [`role_fault`](crate::model_pick::role_fault).

use super::{Marked, PickerState};
use crate::config_edit::brazen::{BzRunner, ProviderRow, row_names};
use crate::model_pick::{grammar, role_fault};

/// Every declared role paired with why it cannot be fired (§9.4): the row it
/// dispatches through is not one brazen's table has. An unusable assignment is
/// visible at the point of change instead of failing at fire.
///
/// **It read the global `models.yaml` too until bl-d9cb**, for a judgement about
/// a table litany no longer loads — so the picker is down to one file, on the
/// read side as well as the write side.
pub(super) fn mark_roles(picker: &mut PickerState, providers_yaml: &str) -> Vec<Marked> {
    let names = row_names(&provider_rows(picker));
    grammar::roles(providers_yaml)
        .into_iter()
        .map(|r| {
            let fault = role_fault(&names, &r);
            (r, fault)
        })
        .collect()
}

/// brazen's effective provider rows, asked once per open (§5.3). The set is
/// brazen's fact, read through the linked `--list-providers` projection so the
/// built-in rows a `config.toml` scan would miss are included.
///
/// Handed back **whole**. Two questions are asked of one read: which rows exist
/// (the dropdown and the §9.2 gate, which want [`row_names`]) and what the
/// selected row needs when it cannot answer (the credential fault's
/// [`remedy`](crate::model_pick::remedy), which wants its `auth` column).
pub(super) fn provider_rows(picker: &mut PickerState) -> Vec<ProviderRow> {
    if picker.rows.is_none() {
        picker.rows = Some(picker.bz_runner.providers());
    }
    picker.rows.clone().unwrap_or_default()
}
