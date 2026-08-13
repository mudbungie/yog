//! The picker's read-back of the assignment it is about to change (§9.4,
//! bl-53be): brazen's provider rows, the global `models.yaml`, and the two
//! joined into "why can this role's current model not be fired?".
//! Coverage-excluded glue — the judgement itself is
//! [`grammar::fault`](crate::model_pick::grammar::fault).

use super::{Marked, PickerState};
use crate::config_edit::FileIo;
use crate::config_edit::brazen::{BzRunner, ProviderRow, row_names};
use crate::model_pick::grammar;

/// Every declared role paired with why its current model cannot be fired
/// (§9.4): the model is undeclared in `models.yaml`, or declared on a provider
/// row brazen's table does not have — the very defect that left two dead Claude
/// entries offerable. An unusable assignment is visible at the point of change
/// instead of failing at fire.
pub(super) fn mark_roles(picker: &mut PickerState, providers_yaml: &str) -> Vec<Marked> {
    let names = row_names(&provider_rows(picker));
    let models = models_text(picker);
    grammar::roles(providers_yaml)
        .into_iter()
        .map(|r| {
            let fault = grammar::fault(&models, &names, &r.model);
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

/// The global `models.yaml` text, read once per open. A missing or unreadable
/// file is empty text — every role then reads as undeclared, which is exactly
/// what lernie says when it refuses to load such a config.
fn models_text(picker: &mut PickerState) -> String {
    if picker.models_text.is_none() {
        let raw = picker
            .io
            .read(&picker.lernie.models())
            .ok()
            .flatten()
            .unwrap_or_default();
        picker.models_text = Some(String::from_utf8_lossy(&raw).into_owned());
    }
    picker.models_text.clone().unwrap_or_default()
}
