//! What a row's `protocol` column says about a yog turn (DESIGN §9.4) — the
//! two reads keyed on brazen's own `ProtocolId`, split off [`super`] at §12's
//! line budget along the seam the module already had: the parent is the table's
//! *projection* (three columns, the credential model, the rendered row), and
//! this is the dialect judgement over one of those columns.
//!
//! **`protocol` is the TOOL capability** (bl-3d22). The column carries brazen's
//! own `ProtocolId` spelling, and `ProtocolId` is a public, closed enum on
//! brazen's library surface — so [`ProviderRow::tools_blocked`] parses the
//! column back into that enum through brazen's own serde rename and answers
//! over a **total match**, which a new upstream dialect fails to compile until
//! its arm is added. That is the whole reason the judgement is here rather than
//! a table of row NAMES: `claude-code` is a name, `claude_code` is a protocol,
//! and only the second is a fact about request shape.
//!
//! It exists because row EXISTENCE never established request-shape
//! compatibility, which §9.4 used to claim it did: `/model worker claude-code
//! <id>` passed the row gate, advanced both config halves, and the next worker
//! start died at brazen's encoder before any network call — *"claude_code
//! carries no tool declarations; use the `anthropic` row for tools"*. Every yog
//! turn declares at least the `clients` tool
//! ([`crate::tool_host::Injection::tools`] returns it unconditionally, and
//! lernie splices the injection into every canonical request), so a
//! tool-less dialect can serve no lernie ROLE at all.
//!
//! **The same judgement answers the failure that already happened** (bl-5252).
//! Gating the picker never reached a config written BEFORE the gate, or written
//! by hand through the §9.1 editor, which is the operator's own authority: that
//! step still dies at encode, and the §7.3 banner offered it Dismiss and nothing
//! else, because [`crate::config_edit::fault`] keyed on lernie's `Config`-kind
//! wrapper and brazen stamps every dialect decline `ErrorKind::ParseInput`. So
//! [`dialect_decline`] reads the dead step's own words into the same match a row
//! is read into — the route, not a second table.
//!
//! **The upstream ask is already filed, and is brazen bl-5053** ("publish
//! per-row capability declines (tools, multi-turn) on the read surface"): brazen
//! projects no capability column — `--list-providers` serves `name`/`protocol`/
//! `auth`/`credential`, and the `Protocol` trait that owns the rejection is
//! crate-private, so the authoritative answer cannot be asked for. That ball was
//! filed off THIS defect seen once before, and its own words name the cost:
//! *"yog's model picker validated provider=claude-code for a tool-bearing worker
//! role (it only checks the row exists in `bz --list-providers`), and every
//! session call then died at encode"*. It was never gated on yog's side, so it
//! recurred. When brazen serves the column, the match below is deleted and the
//! column read in its place — that is the whole migration.
//!
//! **The refusal is per-ROLE, and removes nothing from the catalog.** The VISION
//! §4.9 alignment monitor's check is structurally tool-less
//! ([`crate::monitor::check`]: *"`bz` takes no tool flag, so tool-lessness here
//! is structural rather than a promise"*) and pins a MODEL rather than a provider
//! row, so a tool-less dialect is a legitimate target for it. What cannot be a
//! lernie role's row is not thereby useless.
//!
//! **The second read is a CAVEAT, not a refusal** (bl-671d, §9.4). A dialect
//! whose request declares no context size hands that number to the server, and
//! yog cannot see what the server chose — so the honest answer is the fact and
//! the remedy, at the seat where the row is picked, and never a gate. It is the
//! same discipline [`is_unknown_row`](crate::model_pick::grammar::is_unknown_row)
//! applies to a table that did not answer: no surface may refuse on the strength
//! of a question that went unanswered. `ollama_chat` is the dialect
//! ([`ProviderRow::context_caveat`]), and it is the *whole* of what the wire
//! carries: the pin's own encoder maps the output cap to `options.num_predict`
//! and emits no `options.num_ctx` at all — `tests/brazen_ollama_context.rs`
//! drives the linked brazen and asserts that body, so the sentence below is
//! true because a test says so and fails the day brazen changes it. **The
//! upstream ask is brazen bl-f19d** (a first-class context declaration, or a
//! config passthrough that composes with the typed `options` instead of being
//! dropped by it); when it lands, this read and [`CONTEXT_REMEDY`] go together.

use super::ProviderRow;

/// The operator's way to give an `ollama_chat` row an explicit context, in the
/// one file that authors a row (§9.1's editor) — the hover beside
/// [`ProviderRow::context_caveat`], because the fact fits a line and the recipe
/// does not.
///
/// Both halves are load-bearing and both were measured against the linked
/// brazen (`tests/brazen_ollama_context.rs`). Clearing the typed cap is what
/// lets a nested `options` object through: `encode` inserts the typed `options`
/// FIRST and folds config passthrough with `or_insert`, so a `body_defaults`
/// `options` beside a typed `max_tokens` is dropped **whole and silently** —
/// which is why the recipe restates the output cap inside the object it hands
/// over.
pub const CONTEXT_REMEDY: &str = "give the row an explicit context in this workspace's brazen \
     config.toml: `unsupported_body_keys = [\"max_tokens\"]` together with \
     `body_defaults = { options = { num_ctx = <your context>, num_predict = <your output cap> } }`. \
     Restate the output cap inside that object — clearing the typed one is what lets the object \
     reach the wire, and a nested `options` beside a typed cap is dropped whole and silently.";

impl ProviderRow {
    /// Why this row can carry no lernie **role**, or `None` when it can
    /// (bl-3d22). Read by the §9.4 pick gate
    /// ([`plan`](crate::model_pick::plan)) and by the picker's provider control,
    /// which offers the row unselectably with this sentence beside it.
    ///
    /// Three answers, and the third is the load-bearing one. A dialect that
    /// declines tools is refused, naming its protocol. A dialect that carries
    /// them is fine. A spelling **this build's brazen does not know** is *no
    /// answer* rather than a refusal — the same discipline
    /// [`is_unknown_row`](crate::model_pick::grammar::is_unknown_row) applies to
    /// an unanswerable table: no surface may refuse on the strength of a
    /// question that went unanswered.
    ///
    /// It is a column read into [`dialect_blocked`], which is the judgement
    /// itself and is also what [`dialect_decline`] reads a dead step into.
    pub fn tools_blocked(&self) -> Option<String> {
        dialect_blocked(&self.protocol)
    }

    /// What this row's dialect leaves to the server that a yog turn needs, or
    /// `None` when it leaves nothing (bl-671d). Stated beside a row that stays
    /// selectable — see the module note on why this is not a gate — with
    /// [`CONTEXT_REMEDY`] as its hover.
    ///
    /// One dialect answers today. `ollama_chat`'s request declares no context
    /// size, so the Ollama server's own default governs rather than the model's
    /// capacity, and a yog turn's tool payload alone can exhaust a small one:
    /// the drive this came from spent 4095 input tokens on the payload and
    /// finished on `length` having generated a single token, against a model
    /// whose own context was two hundred times larger. The same total match as
    /// [`Self::tools_blocked`], for the same reason.
    pub fn context_caveat(&self) -> Option<String> {
        match protocol_id(&self.protocol)? {
            brazen::ProtocolId::OllamaChat => Some(format!(
                "{} declares no context size — the request carries the output cap and \
                 nothing else, so the server's own default governs rather than the \
                 model's capacity, and a turn's tool payload alone can exhaust it",
                self.protocol
            )),
            brazen::ProtocolId::AnthropicMessages
            | brazen::ProtocolId::OpenAiChat
            | brazen::ProtocolId::OpenAiResponses
            | brazen::ProtocolId::GoogleGenAi
            | brazen::ProtocolId::ClaudeCode => None,
        }
    }
}

/// Why a dialect can carry no lernie role, keyed on the `protocol` **spelling**
/// rather than on a row — the one match, which [`ProviderRow::tools_blocked`]
/// reads a column into and [`dialect_decline`] reads a dead step's own words
/// into. Two callers, one arm per dialect: a second table would be the thing
/// bl-3d22 refused to build.
fn dialect_blocked(protocol: &str) -> Option<String> {
    match protocol_id(protocol)? {
        brazen::ProtocolId::ClaudeCode => Some(format!(
            "{protocol} declares no tools — every yog turn carries at least the \
             `clients` tool, so this row can serve no role"
        )),
        brazen::ProtocolId::AnthropicMessages
        | brazen::ProtocolId::OpenAiChat
        | brazen::ProtocolId::OpenAiResponses
        | brazen::ProtocolId::GoogleGenAi
        | brazen::ProtocolId::OllamaChat => None,
    }
}

/// Why the dialect a **failed step's own words** name can serve no role, or
/// `None` when they name none (bl-5252) — the route from a dead step back to
/// [`dialect_blocked`], which [`crate::config_edit::fault`] pairs with the §9.1
/// route.
///
/// **The signal is the dialect naming ITSELF, and that is brazen's own habit.**
/// Every decline the `claude_code` encoder writes leads with its `ProtocolId`
/// spelling — *"`claude_code` carries no tool declarations…"*, *"…cannot express
/// a tool_choice"*, *"…is single-turn"*, *"…accepts only text content in
/// {slot}"* — one family from one `reject` helper, all four stamped
/// `ErrorKind::ParseInput`, so no error KIND separates them from a malformed
/// image block. So the scan is over whole `[a-z0-9_]` tokens, judged by the
/// match above, which makes it narrow in the only way that matters: it is
/// **exact and case-sensitive on brazen's spelling**, so the row NAME
/// `claude-code` — hyphen, the thing an operator's config and every other
/// failure line carry — answers nothing, and a tool-capable dialect that names
/// itself (*"anthropic_messages requires max_tokens"*) answers nothing either.
/// A dialect this build cannot name is an unanswered question, as everywhere
/// else here.
pub fn dialect_decline(text: &str) -> Option<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .find_map(dialect_blocked)
}

/// A `protocol` spelling parsed back into brazen's own registry key, through
/// brazen's own serde rename — never a second table of spellings beside it.
/// `None` for a spelling this build cannot name, which is a *newer* brazen
/// or a degraded column, and in both cases an unanswered question.
fn protocol_id(protocol: &str) -> Option<brazen::ProtocolId> {
    serde_json::from_value(serde_json::Value::String(protocol.to_owned())).ok()
}
