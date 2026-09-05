//! What the `protocol` column says about a yog turn (§9.4) — the two total
//! matches over brazen's own `ProtocolId`: the tool refusal (bl-3d22) and the
//! context caveat (bl-671d), plus the remedy the second one hands the
//! operator. The parent owns the columns; this owns the dialect judgement over
//! one of them, exactly as `providers/capability.rs` does in production.

use super::super::{CONTEXT_REMEDY, provider_rows};
use super::LIVE_LISTING;

/// bl-3d22. The tool capability is read off the `protocol` column through
/// brazen's own `ProtocolId` rename, over a TOTAL match — so the answer is a
/// fact about the dialect, never about a row NAME. `claude-code` is the one
/// shipped row whose dialect declines tools; every other dialect in the table
/// carries them.
#[test]
fn the_tool_capability_is_read_off_the_protocol_column() {
    let rows = provider_rows(LIVE_LISTING);
    assert_eq!(rows[2].protocol, "claude_code");
    let why = rows[2].tools_blocked().expect("claude_code declines tools");
    assert!(why.starts_with("claude_code declares no tools"), "{why}");
    assert!(why.contains("`clients` tool"), "{why}");
    assert!(why.contains("can serve no role"), "{why}");
    // The other four dialects in the live table carry tools — including
    // `claude-session-direct`, which is the same vendor over `anthropic_messages`
    // and is the row a role belongs on.
    for i in [0, 1, 3, 4] {
        assert_eq!(rows[i].tools_blocked(), None, "row {i} carries tools");
    }
}

/// A protocol spelling this build's brazen cannot name — a newer dialect, or a
/// degraded column — is **no answer**, not a refusal: no surface may refuse on
/// the strength of a question that went unanswered. The compile-time half of
/// this guarantee is the total match itself, which a new upstream variant breaks
/// until its arm is added.
#[test]
fn an_unspellable_protocol_answers_nothing_about_tools() {
    let rows = provider_rows(
        r#"{"providers":[{"name":"x","protocol":"mystery_wire","auth":"none"},{"name":"y","auth":"none"}]}"#,
    );
    assert_eq!(rows[0].tools_blocked(), None);
    assert_eq!(
        rows[1].tools_blocked(),
        None,
        "an absent column asks nothing"
    );
}

/// bl-671d. The context caveat is the same column read the same way, and its
/// answer is the *other* kind: `ollama_chat` — the `local` row here — states a
/// fact and stays pickable, and no other shipped dialect answers at all. What
/// the sentence claims about the wire is not asserted here but in
/// `tests/brazen_ollama_context.rs`, which drives the linked brazen; this pins
/// the words and the protocol they are keyed on.
#[test]
fn the_context_caveat_is_read_off_the_same_column() {
    let rows = provider_rows(LIVE_LISTING);
    assert_eq!(rows[1].protocol, "ollama_chat");
    let caveat = rows[1]
        .context_caveat()
        .expect("ollama_chat declares no context size");
    assert!(
        caveat.starts_with("ollama_chat declares no context size"),
        "{caveat}"
    );
    assert!(
        caveat.contains("the server's own default governs"),
        "{caveat}"
    );
    assert!(
        caveat.contains("tool payload alone can exhaust it"),
        "{caveat}"
    );
    for i in [0, 2, 3, 4] {
        assert_eq!(
            rows[i].context_caveat(),
            None,
            "row {i} carries its own context size"
        );
    }
}

/// The remedy is the operator's next move, not yog's: the ONE config line that
/// makes an `ollama_chat` row carry a context, in the file that authors a row.
/// It was two lines and a restated output cap until brazen 0.0.10 folded the
/// row's `extra` one namespace deep (upstream bl-f19d), and the line the
/// operator types is asserted here so the sentence cannot drift from the
/// behaviour `tests/brazen_ollama_context.rs` leg two measures.
#[test]
fn the_context_remedy_is_the_one_line_that_lands() {
    let remedy = CONTEXT_REMEDY;
    assert!(
        remedy.contains("body_defaults = { options = { num_ctx = <your context> } }"),
        "{remedy}"
    );
    assert!(
        !remedy.contains("unsupported_body_keys"),
        "the recipe no longer clears the typed cap: {remedy}"
    );
    assert!(
        remedy.contains("num_predict") && remedy.contains("`num_ctx` sizing the input window"),
        "the two limits stay distinct in the operator's words: {remedy}"
    );
}

/// An unnameable spelling answers nothing about the context either — the same
/// discipline as the tool read, on the same parse.
#[test]
fn an_unspellable_protocol_answers_nothing_about_context() {
    let rows =
        provider_rows(r#"{"providers":[{"name":"x","protocol":"mystery_wire","auth":"none"}]}"#);
    assert_eq!(rows[0].context_caveat(), None);
}
