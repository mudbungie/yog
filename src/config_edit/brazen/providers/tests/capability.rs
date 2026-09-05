//! What a row's tool capability says about a yog turn (§9.4) — the refusal
//! (bl-3d22), read since bl-b6c9 off brazen's own `tools` column instead of a
//! total match over `ProtocolId`, and the dead step's route back to the same
//! column (bl-5252). The parent owns the columns; this owns the judgement over
//! one of them, exactly as `providers/capability.rs` does in production.

use super::super::{dialect_decline, provider_rows};
use super::LIVE_LISTING;

/// bl-3d22, migrated by bl-b6c9. The tool capability is read off brazen's own
/// `tools` column — a fact about the row's dialect, computed by the crate that
/// owns the encoder, never about a row NAME. `claude-code` is the one shipped
/// row whose dialect declines tools; every other dialect in the table carries
/// them.
#[test]
fn the_tool_capability_is_read_off_brazens_own_column() {
    let rows = provider_rows(LIVE_LISTING);
    assert_eq!(rows[2].protocol, "claude_code");
    assert_eq!(rows[2].tools, Some(false));
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

/// A `bz` older than 0.0.12 serves no `tools` key, and a degraded row may carry
/// none either. That is **no answer**, not a refusal: no surface may refuse on
/// the strength of a question that went unanswered. The old shape of this
/// guarantee was the total match itself — a compile error on a new upstream
/// dialect — which judged only the dialects this build was compiled against;
/// the column judges every dialect brazen ships, and an absent column judges
/// nothing at all.
#[test]
fn an_absent_tools_column_answers_nothing() {
    let rows = provider_rows(
        r#"{"providers":[{"name":"x","protocol":"mystery_wire","auth":"none"},
                         {"name":"y","protocol":"claude_code","auth":"none"}]}"#,
    );
    assert_eq!(rows[0].tools, None);
    assert_eq!(rows[0].tools_blocked(), None);
    assert_eq!(
        rows[1].tools_blocked(),
        None,
        "a dialect this build knows is still not judged by this build"
    );
}

/// A dialect this build's brazen was never compiled against is judged all the
/// same, which is the whole point of the migration: the column carries the
/// answer, so an unknown spelling that DECLINES is refused and one that carries
/// tools is not.
#[test]
fn an_unknown_dialect_is_judged_by_the_column() {
    let rows = provider_rows(
        r#"{"providers":[{"name":"x","protocol":"mystery_wire","auth":"none","tools":false},
                         {"name":"y","protocol":"other_wire","auth":"none","tools":true}]}"#,
    );
    let why = rows[0].tools_blocked().expect("the column declined");
    assert!(why.starts_with("mystery_wire declares no tools"), "{why}");
    assert_eq!(rows[1].tools_blocked(), None);
}

/// bl-5252, on the same column since bl-b6c9. A dead step's own words name the
/// dialect; the table answers for it, and the sentence is the one the picker
/// gives, so the banner and the picker cannot word one refusal two ways.
#[test]
fn a_dead_steps_words_route_back_to_the_same_column() {
    let rows = provider_rows(LIVE_LISTING);
    let line = "provider error (ParseInput) on provider row \"claude-code\": \
                claude_code carries no tool declarations; use the `anthropic` row for tools";
    let why = dialect_decline(line, &rows).expect("the decline names its dialect");
    assert_eq!(why, rows[2].tools_blocked().expect("the same sentence"));
}

/// The scan is exact and case-sensitive on brazen's own spelling, so the row
/// NAME — hyphen, the thing an operator's config and every other failure line
/// carry — answers nothing, and a tool-carrying dialect that names itself
/// answers nothing either.
#[test]
fn a_row_name_and_a_capable_dialect_route_nowhere() {
    let rows = provider_rows(LIVE_LISTING);
    assert_eq!(
        dialect_decline("unknown provider `claude-code`", &rows),
        None
    );
    assert_eq!(
        dialect_decline("anthropic_messages requires max_tokens", &rows),
        None
    );
}

/// An empty table is one more unanswered question — the same rule `plan` keeps
/// for a table that carries no such row at all.
#[test]
fn an_empty_table_routes_nothing() {
    assert_eq!(
        dialect_decline("claude_code carries no tool declarations", &[]),
        None
    );
}
