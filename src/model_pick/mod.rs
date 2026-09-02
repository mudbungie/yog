//! The model picker (DESIGN §9.4) — the pure half.
//!
//! yog's answer to *"I'm talking to gpt-5.4. How do I change that?"*. The
//! picker is not a fourth config editor: it is the §9.3 lineage write reached by
//! two dropdowns over brazen's own facts.
//!
//! **It was two writes and is one** (bl-d9cb). The pick used to compose §9.2 and
//! §9.3 in a normative order, because litany's cross-check refused a config
//! whose `roles.<r>.model` was not declared in the global `models.yaml`. That
//! check is retired upstream (litany's bl-35e2): a role's `providers.yaml`
//! assignment is the single home of its (provider row, model id) pointer, so
//! there is one file to write and no order to get wrong.
//!
//! This module holds everything that can be decided without touching disk,
//! network or a face: the [`grammar`] both config files are read and written
//! through,
//! the [`query`] view-model over the live `bz --list-models` run, the [`pick`]
//! one gesture produces and the gates it passes, the [`header`] row the
//! conversation wears (the pair its
//! two dropdowns show, plus the §9.4 clause naming a conversation that resolves
//! something else), //! sentences the surface paints — kept here rather than in
//! the excluded shell so the scope claim the UI makes is testable, and so it
//! has exactly one home.

pub mod grammar;
pub mod header;
/// One operator choice, the three gates it passes, and the `providers.yaml`
/// text it produces (§9.4).
pub mod pick;
pub mod query;
/// The §9.4 tuning pair — a role's effort level and its priority lane, the two
/// optional assignment fields litany 0.0.6 reads (bl-23bd).
pub mod tuning;

#[cfg(test)]
pub(crate) mod tests;

pub use grammar::{GrammarError, RoleModel};
pub use header::{
    ConfigPoint, ConfigTip, ModelRow, RETARGET_EXIT, RETARGET_HOVER, birth_row, conversation_row,
    row_role,
};
pub use pick::{Pick, PickError, plan, role_fault};
pub use tuning::{Effort, LEVELS, Tuning};

/// The role that talks to you (litany's `WORKER_ROLE`) — the one the
/// conversation's model row shows and writes, because that is the question
/// being asked; the picker's role strip re-scopes the same two dropdowns.
pub const WORKER_ROLE: &str = "worker";

/// The config branch a pick advances — `litany config <ws>`'s own default
/// (§9.3). A differently-named lineage is edited through the §9.3 surface; the
/// picker deliberately offers no branch chooser. Named here rather than in a
/// frontend because both seats that fire a pick — the §11 pane and the §8.5
/// [`PickModel`](crate::boundary::Action::PickModel) variant — must name the
/// same lineage or they are two pickers.
pub const BRANCH: &str = "default";

/// The file the role assignment lives in, relative to the config checkout.
pub const PROVIDERS: &str = "providers.yaml";

/// Where the picker's provider dropdown lands, **and what it had to leave
/// behind to get there** (bl-bd89, amended by bl-dd7f).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scoped {
    /// The row the model list is asked of and a pick is written for.
    pub row: String,
    /// The role's own row, when brazen does not have it — the row the
    /// conversation actually dispatched through, and the reason its first turn
    /// died. `None` whenever the dropdown shows the role's own row, which is
    /// every healthy case.
    pub stranded: Option<String>,
}

impl Scoped {
    /// The sentence the picker paints when it had to steer (bl-dd7f): the row
    /// that failed, named, beside the live one now selected. The wording lives
    /// here, beside the substitution that makes it true, so the dropdown and
    /// the sentence cannot state different rows.
    pub fn strand_note(&self) -> Option<String> {
        self.stranded.as_ref().map(|was| {
            format!(
                "this conversation was dispatched through {was}, which brazen does not \
                 have — {} is selected instead; add or rename the row in config.toml to \
                 keep it",
                self.row
            )
        })
    }
}

/// The provider row the picker queries and writes for a role (bl-bd89): the
/// role's own row when brazen has it, otherwise brazen's first row.
///
/// A role stranded on a row brazen no longer has — renamed, or dropped from
/// `config.toml` — is precisely the state the picker exists to leave, and
/// asking a dead row for its models can only re-report the strand. An **empty**
/// table is no answer rather than an empty one (brazen could not be asked), so
/// it steers nothing and the role's own row stands.
///
/// **The substitution is a fact the caller is told, not one it has to notice**
/// (bl-dd7f, ruled at bl-9b52). It used to return the bare row, so a
/// conversation whose first turn died on `unknown provider \`openai-chatgpt\``
/// showed a picker reading `anthropic` — brazen's first row — and nothing said
/// the swap had happened: the operator read the picker as a report of what ran.
/// Steering is still right; steering silently was not. So the answer carries
/// both halves ([`Scoped`]) and the seat says the second one.
pub fn default_row(current: &str, rows: &[String]) -> Scoped {
    match rows.first() {
        Some(first) if !rows.iter().any(|r| r == current) => Scoped {
            row: first.clone(),
            stranded: Some(current.to_owned()),
        },
        _ => Scoped {
            row: current.to_owned(),
            stranded: None,
        },
    }
}

/// The scope sentence the picker paints **at the point of change** (§9.4, as
/// inverted by bl-e654). It says the thing the operator would otherwise get
/// wrong, and what that is has changed sides: the write is still
/// workspace-wide, but the conversation in front of them does **not** keep the
/// policy it forked off — it follows this lineage's head, so the pick lands on
/// it at its next step along with every other conversation here.
///
/// `short_oid` is the head the pick advances **from**, which is what makes the
/// sentence checkable against the row beside it rather than a promise.
pub fn scope_sentence(workspace_leaf: &str, branch: &str, short_oid: &str) -> String {
    format!(
        "changes config/{branch} for the whole {workspace_leaf} workspace — \
         every conversation following it, this one included, picks it up at \
         its next step; advances from {short_oid}"
    )
}

/// The scope sentence the same picker paints when it is opened from the §11
/// **birth-config block** — the surface for a conversation not started yet.
///
/// [`scope_sentence`] is about conversations that already exist and will feel
/// this write at their next step, which is not a fact the birth block has.
/// What the birth block must say instead is the one thing the operator would
/// otherwise get wrong:
/// **there is no per-conversation pick to make.** litany's `litany prompt`
/// takes no config argument and resolves the head of `config/<branch>` itself,
/// so a start-time pick *is* the workspace default moving — the same write the
/// §9.4 picker always did, made one gesture before the start instead of after it.
pub fn birth_sentence(workspace_leaf: &str, branch: &str) -> String {
    format!(
        "this moves the {workspace_leaf} workspace default too: picking here \
         advances config/{branch}, and every conversation started next — \
         including the one you are about to start — is born on it"
    )
}

/// The note beside the write, naming the one file a click touches (§9.4), so the
/// operator can see what it is about to do before it does it.
///
/// It named two files and their order until bl-d9cb, and both halves of that
/// sentence were false at the pin: litany reads no global `models:` table, so the
/// declaration was inert and its ordering rule protected nothing.
pub const WRITE_NOTE: &str = "writes the role's provider and model into providers.yaml on this workspace's \
     config branch, through `litany config`";
