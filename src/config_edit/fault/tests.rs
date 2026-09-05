//! The config-kind classifier and its sentence (bl-dd7f).

use super::{ProviderRow, config_remedy, failing_row, looks_config};

/// The table the judgement is read out of (bl-b6c9): brazen's own `tools`
/// column, in the shape `provider_rows` projects a `--list-providers --json`
/// row into. Two rows and no more — the tool-less exec dialect and one that
/// carries them — because what is under test here is the classifier, not the
/// table, and `tests/brazen_claude_code_decline.rs` runs the linked brazen's
/// real listing against the same code.
fn rows() -> Vec<ProviderRow> {
    ["claude_code", "anthropic_messages"]
        .iter()
        .map(|protocol| ProviderRow {
            name: String::new(),
            protocol: (*protocol).to_owned(),
            auth: "none".to_owned(),
            credential: "not required".to_owned(),
            effort: true,
            priority: false,
            tools: Some(*protocol != "claude_code"),
            device: String::new(),
        })
        .collect()
}

/// brazen's four `claude_code` declines, verbatim from the pinned encoder's own
/// `reject` sites, inside litany's `AdapterError` wrapper — the text the §7.3
/// banner is handed. All four are `ErrorKind::ParseInput`, which is why the
/// marker table cannot see any of them.
const DECLINES: &[&str] = &[
    "provider error (ParseInput) on provider row \"claude-code\": claude_code carries no \
     tool declarations; use the `anthropic` row for tools",
    "provider error (ParseInput) on provider row \"claude-code\": claude_code cannot \
     express a tool_choice (no tools reach the CLI)",
    "provider error (ParseInput) on provider row \"claude-code\": claude_code is \
     single-turn: the request must carry exactly one user message (multi-turn replay \
     cannot ride the CLI; use the `anthropic` row)",
    "provider error (ParseInput) on provider row \"claude-code\": claude_code accepts \
     only text content in message (images/documents/tool blocks cannot ride the CLI)",
];

/// The residual bl-3d22 left: a config the picker gate never saw still dies at
/// encode, and every member of that one family now earns the same route the
/// `unknown provider` failure gets — the dialect's own reason, plus the row and
/// the file as the next move.
#[test]
fn every_request_shape_decline_earns_the_config_route() {
    for line in DECLINES {
        assert!(looks_config(line, &rows()), "{line}");
        let remedy = config_remedy(line, &rows()).expect("a dialect decline has a way out");
        assert!(remedy.contains("claude_code declares no tools"), "{remedy}");
        assert!(remedy.contains("config.toml"), "{remedy}");
        assert!(remedy.contains("model picker"), "{remedy}");
    }
}

/// The reason is the picker's own sentence, not a second wording of it: one
/// judgement, so the banner and the §9.4 provider control cannot disagree about
/// why the row is unusable.
#[test]
fn the_reason_is_the_pickers_own_sentence() {
    let why = rows()[0]
        .tools_blocked()
        .expect("the dialect declines tools");
    let remedy = config_remedy(DECLINES[0], &rows()).expect("classified");
    assert!(remedy.starts_with(&why), "{remedy}");
}

/// The narrowness that makes the marker lawful: a **row name** is not a
/// protocol. `claude-code` carries a hyphen and appears in every failure line
/// routed through that row, so keying on brazen's exact `claude_code` spelling
/// is what keeps an unrelated failure from claiming a config remedy — and an
/// unrelated `ParseInput` is left alone for the same reason.
#[test]
fn no_unrelated_failure_on_that_row_is_claimed() {
    for other in [
        "provider error (Transport) on provider row \"claude-code\": connection reset",
        "provider error (ParseInput) on provider row \"anthropic\": message accepts only \
         text content",
        "provider error (ParseInput) on provider row \"google\": google: image URLs are \
         not supported",
        "provider error (ParseInput) on provider row \"anthropic\": anthropic_messages \
         requires max_tokens",
    ] {
        assert!(!looks_config(other, &rows()), "{other}");
        assert_eq!(config_remedy(other, &rows()), None, "{other}");
    }
}

/// The falsifying line from bl-9b52, verbatim: it classifies, and the remedy
/// names the row brazen could not resolve.
#[test]
fn the_failing_dispatch_of_bl_9b52_earns_a_remedy_that_names_its_row() {
    let line = "litany prompt: provider error (Config): unknown provider `openai-chatgpt`";
    assert!(looks_config(line, &rows()));
    assert_eq!(failing_row(line).as_deref(), Some("openai-chatgpt"));
    let remedy = config_remedy(line, &rows()).expect("a config fault has a way out");
    assert!(remedy.contains("openai-chatgpt"), "{remedy}");
    assert!(remedy.contains("config.toml"), "{remedy}");
}

/// The pinned litany names the row it routed to in its own wrapper (its
/// `AdapterError`), so that shape is read too — one classifier over both.
#[test]
fn the_row_is_read_from_litanys_own_wrapper_as_well() {
    let line = "provider error (Config) on provider row \"codex\": no such row";
    assert_eq!(failing_row(line).as_deref(), Some("codex"));
    assert!(config_remedy(line, &rows()).is_some_and(|r| r.contains("codex")));
}

/// A config fault that names no row still earns the remedy — the file is the
/// answer either way, and absence is a value rather than a branch out.
#[test]
fn a_config_fault_with_no_row_still_names_the_file() {
    let line = "litany prompt: provider error (config): the table could not be read";
    assert_eq!(failing_row(line), None);
    let remedy = config_remedy(line, &rows()).expect("still config-shaped");
    assert!(remedy.contains("config.toml"), "{remedy}");
    assert!(!remedy.contains("named"), "no row to name: {remedy}");
}

/// Every other failure class is left alone — a credential decline is §8.3's,
/// a transport reset is nobody's, and routing either to the config editor
/// would be a guess with a button on it.
#[test]
fn no_other_failure_class_is_claimed() {
    for other in [
        "provider error (auth) on provider row \"anthropic\": credential expired",
        "provider error (transport) on provider row \"anthropic\": connection reset",
        "reading config from /home/u/.config/brazen/config.toml",
        "",
    ] {
        assert!(!looks_config(other, &rows()), "{other}");
        assert_eq!(config_remedy(other, &rows()), None, "{other}");
    }
}

/// A quote that never closes yields no row rather than the rest of the line —
/// a half-parsed name sends the operator looking for something that is not
/// there.
#[test]
fn an_unclosed_quote_names_nothing() {
    let line = "unknown provider `openai-chatgpt";
    assert!(looks_config(line, &rows()));
    assert_eq!(failing_row(line), None);
    let remedy = config_remedy(line, &rows()).expect("still config-shaped");
    assert!(!remedy.contains("openai-chatgpt"), "{remedy}");
}

/// An empty quote is not a name either.
#[test]
fn an_empty_quote_names_nothing() {
    assert_eq!(failing_row("unknown provider ``"), None);
}

/// An unanswered table classifies nothing (bl-b6c9). The dialect route is the
/// column's answer arriving, so a listing yog could not read — an empty table,
/// a `bz` older than the column — must leave the decline unclassified rather
/// than refuse a step on a question nobody answered. The marker way in is
/// untouched by it: a config-shaped failure is config-shaped whatever the table
/// says.
#[test]
fn an_unanswered_table_classifies_no_dialect_decline() {
    assert!(!looks_config(DECLINES[0], &[]));
    assert_eq!(config_remedy(DECLINES[0], &[]), None);
    let marker = "litany prompt: provider error (Config): unknown provider `openai-chatgpt`";
    assert!(looks_config(marker, &[]), "the marker way in is table-free");
}
