//! Reading `models.yaml` back: the one number anything reads out of the table
//! (§5.1 #35's denominator), and the older entry shapes the read still takes.
//!
//! **What used to be here, and why it is not** (bl-3ffa). This file also held
//! `declared`/`unknown_rows` — which entries the file declares and which of them
//! name a provider row brazen lacks (bl-53be). That chain's only consumer was the
//! §9.2 Apply gate, so it judged a field whose one reader was the refusal; the
//! chain and the gate are gone together. The row judgement that survives is over
//! `providers.yaml`'s live pointer and lives in `plan.rs`, beside the gate it
//! shares a wording with (bl-d9cb re-pointed it there).

use super::SEEDED_MODELS;
use crate::model_pick::grammar::context_windows;
use std::collections::BTreeMap;

/// The live file as the operator's world carried it: three entries, two of them
/// declaring no window at all, all of them naming the provider row the block
/// used to carry — the shape a file written before bl-3ffa still has on disk.
const LIVE_MODELS: &str = "# header\n\nmodels:\n  claude-sonnet-5:\n    provider: anthropic\n    \
     model_id: claude-sonnet-5\n    context_window: 1000000\n  claude-haiku-4-5:\n    \
     provider: anthropic\n    model_id: claude-haiku-4-5\n  gpt-5.4:\n    provider: codex\n    \
     model_id: gpt-5.4\n";

/// The §5.1 #35 denominator: the window an entry declares, keyed on the **wire
/// id** a step's `request.json` names rather than on the alias the entry is filed
/// under. Since bl-d9cb this is the ONLY number anything reads out of the
/// `models:` table — lernie reads none of it, and the picker writes none of it.
///
/// Both fixtures carry the retired `provider:`/`capabilities:` lines, which is
/// the point of reading them here: bl-3ffa stopped WRITING those fields and
/// changed no read, so an operator's existing file keeps answering with the
/// number it always did.
#[test]
fn reads_the_context_window_each_entry_declares_keyed_on_the_wire_id() {
    assert_eq!(
        context_windows(SEEDED_MODELS),
        BTreeMap::from([("gpt-5.4".to_owned(), 400_000)])
    );
    // Two of the live file's three entries declare no window at all — and an
    // undeclared window is absent, never a default, so no figure is rendered
    // against a number nobody wrote.
    assert_eq!(
        context_windows(LIVE_MODELS),
        BTreeMap::from([("claude-sonnet-5".to_owned(), 1_000_000)])
    );
}

/// An alias entry keys the map on its `model_id`; a window that is zero, not a
/// number, or on a file with no `models:` block at all declares nothing.
#[test]
fn an_undeclarable_window_is_absent_rather_than_guessed() {
    let aliased = "models:\n  sonnet:\n    provider: anthropic\n    \
         model_id: claude-sonnet-5\n    context_window: 200000\n";
    assert_eq!(
        context_windows(aliased),
        BTreeMap::from([("claude-sonnet-5".to_owned(), 200_000)])
    );
    let junk = "models:\n  a:\n    context_window: 0\n  b:\n    context_window: lots\n  \
         c:\n    provider: codex\n";
    assert!(context_windows(junk).is_empty());
    assert!(context_windows("roles:\n  worker:\n    provider: codex\n").is_empty());
    // The shapes the grammar declines are read as declaring nothing, not as a
    // fault: an absent, inline or flow-styled `models:` yields an empty map.
    assert!(context_windows("").is_empty());
    assert!(context_windows("models: {}\n").is_empty());
    assert!(context_windows("modelsx:\n  m:\n    context_window: 1\n").is_empty());
    assert!(context_windows("models:\n  m: { context_window: 1 }\n").is_empty());
}
