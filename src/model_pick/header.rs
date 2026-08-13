//! The conversation's **model row** (§11) and its §9.4 drift clause. "Header"
//! here names the row's shape — what the two dropdowns show, plus the hover and
//! the clause around them — not a seat: the row sits in the bottom settings
//! rows since the settings-seat ruling (§11, bl-2e18).
//!
//! **The row is the selection** (bl-cd2a). The operator's ruling, verbatim:
//! *"Right there, I want essentially that whole line changed to: `<provider> -
//! <model>`. That's it."* So the line carries no `model ·` prefix, no `frozen
//! at <oid>`, and no *change…* button — it carries the pair, in two live
//! dropdowns, and the facts that used to crowd it ride the hover.
//!
//! **What the dropdowns show is what they write: the workspace default.** A
//! conversation is frozen on the config commit its branch forked off, and that
//! is the invariant, not the bug (§9.4) — but a control that displayed the
//! frozen pair would report a write it just made as a no-op, because the tip
//! moves and the freeze cannot. So the pair shown is the config branch tip's,
//! and the freeze is stated beside it exactly when the two have parted.
//!
//! **Drift is derived, never stored.** It is the inequality of two oids the
//! caller already holds: the conversation's governing commit (§5.1 #17) and the
//! workspace's config-lineage tip, which the §7 snapshot already carries. No
//! field records it and no write follows from it — and in particular there is
//! no mid-conversation adoption, which would break the very freeze the clause
//! exists to explain (bl-9786).

use super::{WORKER_ROLE, grammar};

/// One config commit as the row reads it: the oid it is, and the
/// `providers.yaml` its tree carries. Two of these — the governing commit and
/// the workspace default's tip — are the whole input to [`conversation_row`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPoint {
    pub oid: String,
    pub short_oid: String,
    pub providers_yaml: String,
}

impl ConfigPoint {
    /// The role's assignment as this commit makes it — the pair the dropdowns
    /// show. A file with no such role is a value, not an error: the row still
    /// paints, saying so in the one place a missing pair can be said.
    fn pair(&self, role: &str) -> (String, String) {
        grammar::roles(&self.providers_yaml)
            .into_iter()
            .find(|r| r.role == role)
            .map_or_else(
                || (NO_ROW.to_string(), NO_ROW.to_string()),
                |r| (r.provider, r.model),
            )
    }
}

/// The model row as painted (§11): what the two dropdowns show, the hover that
/// explains the freeze, and — only when the workspace default has moved past
/// this conversation — the clause naming what this one is actually frozen on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRow {
    /// The provider row the config branch tip assigns — the provider dropdown's
    /// selection.
    pub provider: String,
    /// The model id the tip assigns — the model dropdown's selection.
    pub model: String,
    pub hover: String,
    /// Drift, derived (never stored): the sentence naming the frozen pair, and
    /// with it the exit affordance, offered exactly here and nowhere else — an
    /// undrifted conversation already runs the current config, so it has
    /// nothing to escape.
    pub drift: Option<String>,
}

impl ModelRow {
    /// Whether the workspace default has parted from this conversation — the
    /// one condition [`NEW_CONVERSATION_EXIT`] is offered under.
    pub fn drifted(&self) -> bool {
        self.drift.is_some()
    }
}

/// What a dropdown shows when the `providers.yaml` it reads declares no such
/// role: the pair is absent, and absence is a value.
const NO_ROW: &str = "(none)";

/// The one honest exit (bl-9786), labelled with what it does rather than with
/// where it goes: the affordance only focuses the composer's existing
/// new-conversation verb, so it is a pointer at a gesture, not a second way to
/// start a conversation.
pub const NEW_CONVERSATION_EXIT: &str = "new conversation uses the current config";

/// Compose the conversation's model row from its governing config and the
/// workspace's current config-lineage tip. The dropdowns show `tip`, because
/// that is what a pick moves; `governing` is what this conversation is frozen
/// on, and it is named only when the two have parted.
pub fn conversation_row(governing: &ConfigPoint, tip: &ConfigPoint, role: &str) -> ModelRow {
    let (provider, model) = tip.pair(role);
    let hover = format!(
        "the {role} this workspace's next conversation will be born on — picking \
         here advances the workspace default at once. This conversation stays \
         frozen on the config commit it forked off ({}); a conversation's policy \
         never changes under it",
        governing.short_oid
    );
    let drift = (governing.oid != tip.oid).then(|| {
        let (was_provider, was_model) = governing.pair(role);
        format!(
            "this conversation is frozen on {was_provider} · {was_model} at {}",
            governing.short_oid
        )
    });
    ModelRow {
        provider,
        model,
        hover,
        drift,
    }
}

/// Compose the §11 birth-config block's model row from the config branch's
/// **head** — the commit a fresh root forks off. No drift clause: a
/// conversation that does not exist yet cannot have parted from anything.
///
/// That head is not a yog choice: lernie 0.0.3's `lernie prompt <repo>
/// <message>` takes no config argument and resolves
/// `ConfigSource::ConfigBranch("config/default")` internally, so "which config
/// is this conversation born on" has exactly one answer and the hover says it.
pub fn birth_row(tip: &ConfigPoint, role: &str, branch: &str) -> ModelRow {
    let (provider, model) = tip.pair(role);
    ModelRow {
        provider,
        model,
        hover: format!(
            "the {role} the conversation you are about to start will be born on: a \
             new conversation forks the head of config/{branch} ({}) and is frozen \
             there for its whole life — lernie takes no per-conversation config, so \
             changing the model before the start advances that head",
            tip.short_oid
        ),
        drift: None,
    }
}

/// The role a bare row reports and writes — the one that talks to you
/// ([`WORKER_ROLE`]), which the picker's role strip re-scopes while it is open.
pub fn row_role(selected: Option<&str>) -> String {
    selected.unwrap_or(WORKER_ROLE).to_string()
}
