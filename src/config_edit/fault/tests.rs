//! The config-kind classifier and its sentence (bl-dd7f).

use super::{config_remedy, failing_row, looks_config};

/// The falsifying line from bl-9b52, verbatim: it classifies, and the remedy
/// names the row brazen could not resolve.
#[test]
fn the_failing_dispatch_of_bl_9b52_earns_a_remedy_that_names_its_row() {
    let line = "lernie prompt: provider error (Config): unknown provider `openai-chatgpt`";
    assert!(looks_config(line));
    assert_eq!(failing_row(line).as_deref(), Some("openai-chatgpt"));
    let remedy = config_remedy(line).expect("a config fault has a way out");
    assert!(remedy.contains("openai-chatgpt"), "{remedy}");
    assert!(remedy.contains("config.toml"), "{remedy}");
}

/// The pinned lernie names the row it routed to in its own wrapper (its
/// `AdapterError`), so that shape is read too — one classifier over both.
#[test]
fn the_row_is_read_from_lernies_own_wrapper_as_well() {
    let line = "provider error (Config) on provider row \"codex\": no such row";
    assert_eq!(failing_row(line).as_deref(), Some("codex"));
    assert!(config_remedy(line).is_some_and(|r| r.contains("codex")));
}

/// A config fault that names no row still earns the remedy — the file is the
/// answer either way, and absence is a value rather than a branch out.
#[test]
fn a_config_fault_with_no_row_still_names_the_file() {
    let line = "lernie prompt: provider error (config): the table could not be read";
    assert_eq!(failing_row(line), None);
    let remedy = config_remedy(line).expect("still config-shaped");
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
        assert!(!looks_config(other), "{other}");
        assert_eq!(config_remedy(other), None, "{other}");
    }
}

/// A quote that never closes yields no row rather than the rest of the line —
/// a half-parsed name sends the operator looking for something that is not
/// there.
#[test]
fn an_unclosed_quote_names_nothing() {
    let line = "unknown provider `openai-chatgpt";
    assert!(looks_config(line));
    assert_eq!(failing_row(line), None);
    let remedy = config_remedy(line).expect("still config-shaped");
    assert!(!remedy.contains("openai-chatgpt"), "{remedy}");
}

/// An empty quote is not a name either.
#[test]
fn an_empty_quote_names_nothing() {
    assert_eq!(failing_row("unknown provider ``"), None);
}
