//! The conversation's **model row** (§11) and its §9.4 apart clause. "Header"
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
//! **What the dropdowns show is what they write, and now also what this
//! conversation runs.** The pair shown is the config lineage tip's. That used
//! to need an argument — a control displaying the frozen pair would report a
//! write it just made as a no-op — and since litany's follow-the-tip ruling it
//! needs none: control resolves the lineage's head at every step boundary
//! (bl-e654, upstream bl-403b), so the tip's pair *is* the conversation's pair,
//! and a pick lands on the conversation in front of the operator at its next
//! step.
//!
//! **The clause survived the inversion by changing sides** (bl-e654). It is
//! still one derived inequality over two oids the caller already holds — the
//! commit this conversation resolves (§5.1 #17, now the followed one) against
//! the workspace lineage's tip, which the §7 snapshot already carries — and
//! still nothing is stored. What differs is what the inequality MEANS. It can
//! no longer mean *frozen behind*, because following is the default; it can
//! only mean this conversation does not resolve this workspace's lineage — it
//! is held on a divergence, or it follows another one. Both are the states
//! `retarget` exists for, which is why the one exit rides here.
//!
//! **Why a clause at all, still.** bl-9786's lesson outlived the doctrine it
//! was filed under: the scope sentence is read at the moment of the *write* and
//! the surprise arrives at the moment of the *read*, so the fact belongs on the
//! row an operator comes back to, not in a caption they saw once.

use super::{WORKER_ROLE, grammar};

/// The workspace's config-lineage tip (§2.2) as a **seat** holds it (REMOTE
/// §9.4, bl-1eb0): the commit `litany prompt` forks the next conversation off.
/// Two strings, because two strings are all the picker asks of it — the row
/// labels with the short oid and reads `providers.yaml` at the full one — and
/// because the `CommitNode` this replaced is a git-derivation record a face
/// holding no repository cannot be handed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigTip {
    pub oid: String,
    pub short_oid: String,
}

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
/// explains the scope, and — only when this conversation does not resolve this
/// workspace's lineage — the clause naming what it resolves instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRow {
    /// The provider row the config branch tip assigns — the provider dropdown's
    /// selection.
    pub provider: String,
    /// The model id the tip assigns — the model dropdown's selection.
    pub model: String,
    pub hover: String,
    /// Derived, never stored: the sentence naming the pair this conversation
    /// actually resolves, and with it [`RETARGET_EXIT`], offered exactly here
    /// and nowhere else — a conversation following this lineage already runs
    /// what the dropdowns show, so there is nothing to settle.
    pub apart: Option<String>,
}

impl ModelRow {
    /// Whether this conversation resolves something other than this
    /// workspace's config lineage tip — the one condition [`RETARGET_EXIT`] is
    /// offered under.
    pub fn is_apart(&self) -> bool {
        self.apart.is_some()
    }
}

/// What a dropdown shows when the `providers.yaml` it reads declares no such
/// role: the pair is absent, and absence is a value.
const NO_ROW: &str = "(none)";

/// **The clause's one exit** (bl-2d19, left alone by bl-e654): litany's own
/// `retarget` verb, which re-forks this conversation onto the lineage this
/// workspace runs and replays every commit it has made onto that base — so its
/// whole history survives the move.
///
/// It used to stand beside a second, *new conversation uses the current
/// config*, and bl-9786 ruled them a strip of peers with this one leading
/// because discarding a history is the larger act. The peer is **gone**
/// (bl-e654): it was the escape from a freeze, and there is no freeze to
/// escape — a conversation that merely wants the current config already has
/// it. What is left here is a conversation that resolves a different lineage or
/// none, and starting a new one settles nothing about this one.
pub const RETARGET_EXIT: &str = "settle this conversation onto this workspace's config lineage";

/// What pressing that says (§11 discoverability rule 3, which is also where its
/// keyboard spelling is named): the verb, its timing, and the one thing an
/// operator must know before spending it — that nothing is thrown away.
pub const RETARGET_HOVER: &str = "Mark this conversation to be re-forked onto the config lineage \
    this workspace runs (litany retarget) — you do not need this for an edit to \
    the lineage it already follows, which reaches it on its own. It keeps every \
    message: the re-fork lands at the conversation's next step, with the work it \
    has already done replayed on top. Type /retarget to do it without the mouse.";

/// Compose the conversation's model row from the config commit it **resolves**
/// (§5.1 #17's followed answer) and the workspace's config-lineage tip. The
/// dropdowns show `tip`, because that is both what a pick moves and — for every
/// conversation following this lineage — what this conversation runs. `resolved`
/// is named only when the two have parted, which is now the abnormal case.
pub fn conversation_row(resolved: &ConfigPoint, tip: &ConfigPoint, role: &str) -> ModelRow {
    let (provider, model) = tip.pair(role);
    let hover = format!(
        "the {role} this workspace's conversations run — picking here advances \
         the lineage at once, and every conversation following it takes the new \
         pair at its next step. This one resolves {}",
        resolved.short_oid
    );
    let apart = (resolved.oid != tip.oid).then(|| {
        let (its_provider, its_model) = resolved.pair(role);
        format!(
            "this conversation resolves {its_provider} · {its_model} at {}, not this \
             workspace's lineage",
            resolved.short_oid
        )
    });
    ModelRow {
        provider,
        model,
        hover,
        apart,
    }
}

/// Compose the §11 birth-config block's model row from the config branch's
/// **head** — the commit a fresh root forks off. No apart clause: a
/// conversation that does not exist yet cannot resolve anything else.
///
/// That head is not a yog choice: litany's `litany prompt <repo>
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
             new conversation forks the head of config/{branch} ({}) and follows \
             that lineage from then on — litany takes no per-conversation config, so \
             changing the model before the start advances that head",
            tip.short_oid
        ),
        apart: None,
    }
}

/// The role a bare row reports and writes — the one that talks to you
/// ([`WORKER_ROLE`]), which the picker's role strip re-scopes while it is open.
pub fn row_role(selected: Option<&str>) -> String {
    selected.unwrap_or(WORKER_ROLE).to_string()
}
