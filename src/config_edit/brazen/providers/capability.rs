//! What a row's tool capability says about a yog turn (DESIGN §9.4) — the read
//! keyed on brazen's own `tools` column, split off [`super`] at §12's line
//! budget along the seam the module already had: the parent is the table's
//! *projection* (the columns, the credential model, the rendered row), and this
//! is the judgement over one of those columns.
//!
//! **The tool capability is brazen's, and this file no longer re-derives it**
//! (bl-b6c9). It used to: [`ProviderRow::tools_blocked`] parsed the `protocol`
//! column back into brazen's `ProtocolId` and answered over a **total match**,
//! so a new upstream dialect failed to compile until an arm was added here. The
//! declaration is a column since brazen 0.0.12 (upstream bl-5053) — `shapes()`
//! per dialect, proved there against each dialect's own `encode` — so the match
//! is deleted and the answer is read. A total match could never have been the
//! right shape anyway: it can only judge the dialects THIS build was compiled
//! against, and the compile error it bought arrives at the wrong moment (a pin
//! bump), on the wrong side (yog's), for a fact brazen already knows.
//!
//! **It stays a ROW read, never a table of row NAMES** (bl-3d22, the reasoning
//! that survives the migration intact): `claude-code` is a name, `claude_code`
//! is a protocol, and only the second is a fact about request shape. The column
//! is per row for exactly that reason — brazen computes it off the row, and a
//! row is what an operator picks.
//!
//! It exists because row EXISTENCE never established request-shape
//! compatibility, which §9.4 used to claim it did: `/model worker claude-code
//! <id>` passed the row gate, advanced both config halves, and the next worker
//! start died at brazen's encoder before any network call — *"claude_code
//! carries no tool declarations; use the `anthropic` row for tools"*. Every yog
//! turn declares at least the `clients` tool
//! ([`crate::tool_host::Injection::tools`] returns it unconditionally, and
//! litany splices the injection into every canonical request), so a
//! tool-less dialect can serve no litany ROLE at all.
//!
//! **The same judgement answers the failure that already happened** (bl-5252).
//! Gating the picker never reached a config written BEFORE the gate, or written
//! by hand through the §9.1 editor, which is the operator's own authority: that
//! step still dies at encode, and the §7.3 banner offered it Dismiss and nothing
//! else, because [`crate::config_edit::fault`] keyed on litany's `Config`-kind
//! wrapper and brazen stamps every dialect decline `ErrorKind::ParseInput`. So
//! [`dialect_decline`] reads the dead step's own words back to the SAME column
//! a pick is refused on — one judgement, two readers, and the banner and the
//! picker cannot word the refusal differently.
//!
//! **An unanswered question refuses nothing.** A row whose `tools` key this
//! build's `bz` does not serve (any brazen before 0.0.12) answers `None`, which
//! is *no answer* rather than a refusal — the discipline
//! [`is_unknown_row`](crate::model_pick::grammar::is_unknown_row) applies to a
//! table that did not answer, and the reason the column is read as an
//! `Option<bool>` rather than a bool.
//!
//! **The context caveat retired here** (bl-b6c9, was bl-671d). It stated that
//! `ollama_chat` "declares no context size", over a second total match on
//! `ProtocolId`, and both halves of that stopped being yog's to say. brazen
//! 0.0.10 (upstream bl-f19d) folds a row's `body_defaults` `extra` one
//! namespace deep, so an `options.num_ctx` a row states reaches the request;
//! brazen 0.0.13 (upstream bl-c655) makes a row's own `context_windows` table
//! the last rung of the window a turn reports, and stamps whichever rung
//! answered onto every `Usage` event. So the window is a ROW's statement now,
//! not a dialect's silence — and **no column publishes it**, so yog cannot read
//! whether a given row states one. A caveat asserted on a dialect while the
//! fact belongs to a row is a statement on the strength of a question that went
//! unanswered, which is the one thing this file's own discipline forbids; and
//! it reached no surface in any case — nothing painted it, and
//! [`ProviderRowView`](super::ProviderRowView) never carried it. The operator's
//! next move survives where it is always true rather than per row: DESIGN §9.4
//! states both row syntaxes, and `tests/brazen_ollama_context.rs` keeps
//! measuring the behaviour that prose rests on.

use super::ProviderRow;

impl ProviderRow {
    /// Why this row can carry no litany **role**, or `None` when it can
    /// (bl-3d22, migrated to brazen's column by bl-b6c9). Read by the §9.4 pick
    /// gate ([`plan`](crate::model_pick::plan)) and by the picker's provider
    /// control, which offers the row unselectably with this sentence beside it.
    ///
    /// Three answers, and the third is the load-bearing one.
    /// [`tools`](ProviderRow::tools) `Some(false)` is refused, naming the
    /// protocol the row speaks. `Some(true)` is fine. `None` — a `bz` that
    /// serves no such column — is *no answer* rather than a refusal.
    pub fn tools_blocked(&self) -> Option<String> {
        (self.tools == Some(false)).then(|| blocked_words(&self.protocol))
    }
}

/// The refusal in words, given the dialect the row speaks. One home, so the
/// picker's gate and [`dialect_decline`]'s route cannot word it differently.
fn blocked_words(protocol: &str) -> String {
    format!(
        "{protocol} declares no tools — every yog turn carries at least the \
         `clients` tool, so this row can serve no role"
    )
}

/// Why the dialect a **failed step's own words** name can serve no role, or
/// `None` when they name none (bl-5252) — the route from a dead step back to
/// the same column [`ProviderRow::tools_blocked`] reads, which
/// [`crate::config_edit::fault`] pairs with the §9.1 route.
///
/// **The signal is the dialect naming ITSELF, and that is brazen's own habit.**
/// Every decline the `claude_code` encoder writes leads with its `ProtocolId`
/// spelling — *"`claude_code` carries no tool declarations…"*, *"…cannot express
/// a tool_choice"*, *"…is single-turn"*, *"…accepts only text content in
/// {slot}"* — one family from one `reject` helper, all four stamped
/// `ErrorKind::ParseInput`, so no error KIND separates them from a malformed
/// image block. So the scan is over whole `[a-z0-9_]` tokens, answered by the
/// table, which makes it narrow in the only way that matters: it is **exact and
/// case-sensitive on brazen's spelling**, so the row NAME `claude-code` —
/// hyphen, the thing an operator's config and every other failure line carry —
/// answers nothing, and a tool-capable dialect that names itself
/// (*"anthropic_messages requires max_tokens"*) answers nothing either.
///
/// **`rows` is the table, and it is the judgement — not a join** (bl-b6c9).
/// Since the answer moved to brazen's column there is nowhere else to ask it:
/// the fact is per row, and every row speaking one dialect answers alike
/// because brazen computes the column off the dialect's `shapes()`. So the
/// first row whose protocol the text names answers for the dialect. A table
/// that carries no such row is one more unanswered question, exactly as an
/// empty table gates nothing in [`plan`](crate::model_pick::plan).
pub fn dialect_decline(text: &str, rows: &[ProviderRow]) -> Option<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .find_map(|token| {
            rows.iter()
                .find(|row| row.protocol == token)
                .and_then(ProviderRow::tools_blocked)
        })
}
