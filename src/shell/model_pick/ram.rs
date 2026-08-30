//! The §9.4 picker's RAM (§5.3) — the surface's whole cross-frame state, held
//! only while the picker that asked is open, plus the write seams folded once
//! from the env snapshot. Inert: nothing here paints, and every field is
//! `pub(super)`, i.e. reachable exactly where it was when it lived in
//! [`super`] — the pane, [`super::marks`] and [`super::write`].

use crate::config_edit::brazen::{ProviderRow, RealBzRunner};
use crate::model_pick::ModelRow;
use crate::model_pick::query::Roster;
use crate::xdg::Env;

/// The conversation's model row, held with the exact facts it was derived
/// from: the agent's branch tip, the workspace's config-lineage tip, and the
/// role the row reports. Keying on all three is exact rather than a staleness
/// gamble — a fork point is immutable, the §9.4 drift clause moves only when the
/// lineage tip does (precisely the change the operator made and did not see,
/// bl-9786), and the role moves only when the pane's strip re-scopes the two
/// dropdowns (bl-cd2a).
pub(super) struct FrozenMemo {
    pub(super) key: (String, String, String),
    /// The governing commit's short oid — what the pane scopes its write claim
    /// with, so the memo answers both questions from one derivation.
    pub(super) short_oid: String,
    pub(super) row: ModelRow,
}

/// The §11 birth-config block's row, held with the config-branch head it was
/// read at and the role it reports. No fork point: a conversation that does not
/// exist yet has not parted from anything, so there is nothing here for a drift
/// clause to be about (bl-824e).
pub(super) struct BirthMemo {
    pub(super) key: (String, String),
    pub(super) row: ModelRow,
}

/// The picker's RAM (§5.3): whether it is open, which role is selected, the
/// in-flight roster run, and the last write's outcome — plus the seams the two
/// writes go through, folded once from the env snapshot.
pub struct PickerState {
    pub open: bool,
    pub(super) role: Option<String>,
    /// The brazen provider row the operator selected, once they have moved off
    /// the default (bl-bd89). `None` means "whatever
    /// [`default_row`](crate::model_pick::default_row) says for the selected
    /// role", so a role change re-defaults rather than carrying a stale row.
    pub(super) provider: Option<String>,
    /// The model id chosen from the roster dropdown, and — when the *custom
    /// model id…* entry is chosen instead — the id being typed. `custom` being
    /// `Some` is what makes the free-entry field visible, so an empty string
    /// there is a live choice rather than "nothing chosen".
    pub(super) model: Option<String>,
    pub(super) custom: Option<String>,
    pub(super) roster: Option<Roster>,
    /// The last write's sentence and the ticket its receipt lands under
    /// (REMOTE §9.8, bl-4841) — the pick and the drift exit share it, because
    /// they are one line: whichever the operator spent, this is what it means.
    pub(super) act: crate::shell::act::Held,
    /// The model line memoized on the two oids it is derived from (§5.3
    /// memoized derived snapshots). Without it the header would re-run
    /// `governing_config` (several git spawns) on every repaint.
    pub(super) frozen: Option<FrozenMemo>,
    /// The birth line memoized on the config-branch head it was read at — the
    /// same several-git-spawns problem, one key instead of two.
    pub(super) birth: Option<BirthMemo>,
    /// The config-branch tip's `providers.yaml`, read once per open rather than
    /// per frame; cleared by [`toggle`](Self::toggle) and after a write.
    pub(super) tip_providers: Option<String>,
    /// brazen's effective provider rows — the whole of "is the role's row live?"
    /// since bl-d9cb, asked once per open on the same terms as `tip_providers`
    /// (the global `models.yaml` text used to be held beside them, for a
    /// judgement over a table litany no longer loads). Held **whole**, `auth`
    /// column included: the credential fault's remedy is derived from that column
    /// (bl-91f1), so reducing them to names here would throw away the fact one
    /// paint later and buy a second read of it.
    pub(super) rows: Option<Vec<ProviderRow>>,
    pub(super) bz_runner: RealBzRunner,
    /// The wall layer (§16.2 as amended) the `--list-models` spawn is fired
    /// with, so the roster is listed against this workspace's providers and
    /// cached in this workspace's own cache.
    pub(super) wall: Vec<(String, String)>,
}

impl PickerState {
    /// Fold the write seams from `wall` — the lensed env of the workspace this
    /// picker belongs to (§16.2 as amended); the surface starts closed.
    ///
    /// There is no re-seating verb: a picker is one wall's for life, and a focus
    /// change swaps the whole state (bl-5894). Re-seating two seams in place
    /// left the open flag, the role, the half-made pick and the roster from the
    /// previous sphere on screen and actionable against this one.
    pub fn new(wall: &Env) -> Self {
        Self {
            open: false,
            role: None,
            provider: None,
            model: None,
            custom: None,
            roster: None,
            act: crate::shell::act::Held::default(),
            frozen: None,
            birth: None,
            tip_providers: None,
            rows: None,
            bz_runner: RealBzRunner::resolve(wall),
            wall: crate::world::wall::pairs_of(wall),
        }
    }

    /// The `m` binding and the model line's *change…* button (§11): opening always
    /// discards the previous answer, so the next paint re-fires the query.
    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.role = None;
        self.forget_choice();
        self.roster = None;
        self.tip_providers = None;
        self.rows = None;
        self.act.forget();
    }

    /// Drop the provider row and model chosen for the previous role, so the
    /// next one defaults off its own assignment instead of inheriting a pick
    /// that was never about it.
    pub(super) fn forget_choice(&mut self) {
        self.provider = None;
        self.model = None;
        self.custom = None;
    }
}
