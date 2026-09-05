//! **What settings exist** — the §9.5 field tables, one per file yog has a
//! grammar for. The sibling half of [`super`], which is *how* a setting is read
//! and written; here is only the enumeration.
//!
//! Adding a setting is a [`FieldSpec`] row; adding a file is one [`schema_for`]
//! arm plus its table. Nothing else in the pane changes — which is the whole
//! point of splitting the enumeration out of the mechanism (the fleet-cadence
//! settings of bl-3381 land here as rows).

use crate::app::cadence;
use crate::model_pick::grammar::{PROVIDERS_YAML, ROLES};

/// How one setting is edited (§9.5). Four kinds, because four are what the
/// files declare: a reference to a brazen provider row, a bounded number, one
/// of litany's inline flow sequences, and a scalar whose value set belongs to
/// the far side (a role's model id) rather than to yog.
///
/// There is deliberately no boolean and no closed-enum kind: **no file these
/// surfaces reach declares one.** The two enumerated settings in config mode —
/// the config-branch origin (advance/orphan) and the §16.3 marks mode — are
/// yog's own verb selectors, not file values, and already wear radio controls.
/// A kind with no member would be mechanism without a setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    /// A brazen provider-row reference: picked from brazen's live table, and
    /// faulted by the one row judgement every §9 site shares.
    Provider,
    /// A whole number the file's contract bounds.
    Number { min: u64, max: u64 },
    /// litany's inline flow sequence (`[a, b]`), edited as its member names.
    List,
    /// A scalar yog holds no vocabulary for.
    Text,
}

/// One field of an entry, with the words the pane paints beside it.
#[derive(Debug, Clone, Copy)]
pub struct FieldSpec {
    pub name: &'static str,
    pub control: Control,
    pub help: &'static str,
}

/// A file yog can present as controls: the column-0 block its entries hang
/// under, and the fields one entry carries.
#[derive(Debug, Clone, Copy)]
pub struct Schema {
    pub file: &'static str,
    pub block: &'static str,
    pub fields: &'static [FieldSpec],
}

const ROLE_FIELDS: &[FieldSpec] = &[
    FieldSpec {
        name: "provider",
        control: Control::Provider,
        help: "the brazen provider row this role dispatches through",
    },
    FieldSpec {
        name: "model",
        control: Control::Text,
        help: "the wire id this role dispatches on — with the row beside it, the \
               whole of the binding (nothing declares it elsewhere; an id that \
               does not exist is caught at the first live call)",
    },
    FieldSpec {
        name: "tools",
        control: Control::List,
        help: "the tool names this role may call (litany's vocabulary, not \
               yog's); yog grants `message` and `dispatch` at creation",
    },
];

/// The clock's periods (§7.2, bl-3381): the bounds are [`cadence`]'s own, so
/// the control and the worker-side parse cannot disagree on what is legal.
const CADENCE_FIELDS: &[FieldSpec] = &[
    FieldSpec {
        name: cadence::DEBOUNCE_MS,
        control: Control::Number {
            min: cadence::DEBOUNCE_BOUNDS.0,
            max: cadence::DEBOUNCE_BOUNDS.1,
        },
        help: "how long a changed workspace coalesces before it re-derives — \
               lower re-renders sooner under a storm, 0 turns coalescing off",
    },
    FieldSpec {
        name: cadence::CHEAP_SWEEP_MS,
        control: Control::Number {
            min: cadence::CHEAP_SWEEP_BOUNDS.0,
            max: cadence::CHEAP_SWEEP_BOUNDS.1,
        },
        help: "the cheap sweep: enumerations, watch reconcile and liveness \
               re-probes, and the window's idle wake-up floor — the beat most \
               of yog's freshness rides on",
    },
    FieldSpec {
        name: cadence::FULL_SWEEP_MS,
        control: Control::Number {
            min: cadence::FULL_SWEEP_BOUNDS.0,
            max: cadence::FULL_SWEEP_BOUNDS.1,
        },
        help: "the full sweep: re-derive everything, the backstop that bounds \
               staleness when a filesystem event is lost",
    },
];

/// `providers.yaml`'s settings (§9.3): one entry per declared role.
pub const ROLES_SCHEMA: Schema = Schema {
    file: PROVIDERS_YAML,
    block: ROLES,
    fields: ROLE_FIELDS,
};

/// `cadence.yaml`'s settings (§7.2, bl-3381): the one `watcher` entry today; a
/// future clock consumer is a sibling entry, a row and not a rebuild.
pub const CADENCE_SCHEMA: Schema = Schema {
    file: cadence::CADENCE_YAML,
    block: cadence::BLOCK,
    fields: CADENCE_FIELDS,
};

/// The schema for a file basename, or `None` when yog has no reader for it —
/// which is the raw-text fallback, not a failure: `models.yaml` (since
/// bl-9c8a, its one yog-read fact having moved to the step record),
/// `workflows/*.yaml`, `souls/**` prose and litany's own
/// `workflow.yaml`/`manifest.yaml` are documents yog declines to interpret
/// (§9.5), and a form over a shape yog guessed at would be exactly the second
/// authority §9 forbids.
pub fn schema_for(file_name: &str) -> Option<Schema> {
    match file_name {
        PROVIDERS_YAML => Some(ROLES_SCHEMA),
        cadence::CADENCE_YAML => Some(CADENCE_SCHEMA),
        _ => None,
    }
}
