//! The §9.4 pick gate's second question (bl-3d22): can the chosen provider row's
//! **protocol** carry a yog turn at all?
//!
//! The defect these cover is a capability mismatch accepted at configuration
//! time and detected at use time. `/model worker claude-code <id>` passed the
//! row gate — brazen really does ship that row — advanced both config halves,
//! and the next worker start died inside brazen's encoder before any network
//! call: *"claude_code carries no tool declarations; use the `anthropic` row for
//! tools"*. Row existence never established request-shape compatibility, which
//! is the design claim bl-3d22 amended.

use super::{SEEDED_MODELS, TEMPLATE_PROVIDERS, rows_on, table};
use crate::config_edit::brazen::ProviderRow;
use crate::model_pick::{Pick, PickError, WORKER_ROLE, plan};

fn pick(provider: &str) -> Pick {
    Pick {
        role: WORKER_ROLE.to_string(),
        provider: provider.to_string(),
        model: "sonnet".to_string(),
    }
}

fn refuse(rows: &[ProviderRow], provider: &str) -> PickError {
    plan(
        SEEDED_MODELS,
        TEMPLATE_PROVIDERS,
        rows,
        &pick(provider),
        None,
    )
    .expect_err("the row cannot serve a role")
}

/// The reproduction, at the gate. brazen ships `claude-code` as a built-in row,
/// so the table HAS it and the old gate waved it through; the pick is now
/// refused before either file is touched, naming the protocol rather than the
/// row name — the capability is a fact about the dialect, not about a spelling
/// yog recognizes.
#[test]
fn a_pick_on_a_tool_less_protocol_is_refused_before_either_file() {
    let rows = rows_on(&["claude-code"], "claude_code");
    let refused = refuse(&rows, "claude-code");
    assert_eq!(
        refused,
        PickError::Incapable {
            provider: "claude-code".to_string(),
            why: rows[0].tools_blocked().expect("the dialect declines tools"),
        }
    );
    let said = refused.to_string();
    assert!(said.contains("`claude-code` cannot serve a role"), "{said}");
    assert!(said.contains("claude_code declares no tools"), "{said}");
    assert!(said.contains("`clients` tool"), "{said}");
    assert!(said.contains("pick a tool-capable row"), "{said}");
}

/// The row gate is asked FIRST, so a row brazen lacks is still an unknown row
/// rather than an incapable one — there is no protocol to judge on a row that
/// does not exist, which is why the capability gate needs no case for it.
#[test]
fn a_row_the_table_does_not_carry_is_unknown_not_incapable() {
    assert_eq!(
        refuse(&rows_on(&["claude-code"], "claude_code"), "nope"),
        PickError::UnknownProvider {
            provider: "nope".to_string(),
        }
    );
}

/// An empty table is brazen unanswered, not brazen answering "none" — it gates
/// nothing, on the same terms as the unknown-row gate beside it. A tool-carrying
/// row is the control: the same pick, the same model id, written.
#[test]
fn an_unanswerable_table_and_a_capable_row_both_pass() {
    assert!(
        plan(
            SEEDED_MODELS,
            TEMPLATE_PROVIDERS,
            &[],
            &pick("claude-code"),
            None
        )
        .is_ok()
    );
    assert!(
        plan(
            SEEDED_MODELS,
            TEMPLATE_PROVIDERS,
            &table(&["codex"]),
            &pick("codex"),
            None
        )
        .is_ok()
    );
}

/// The custom-id entry is the one seat that can name a model brazen never
/// listed, and §9.4 used to reason it could declare an unserved model *"but
/// never an unroutable one, because the row beside it is still brazen's"*. The
/// row being brazen's is exactly what this ball falsified: a custom id on a
/// tool-less row is refused like any other pick, and a role whose own tool list
/// is EMPTY is refused too — yog's injection declares the `clients` tool
/// whatever the role elected, so an empty `tools: []` buys no exemption.
#[test]
fn neither_a_custom_id_nor_an_empty_role_tool_list_earns_an_exemption() {
    let rows = rows_on(&["claude-code"], "claude_code");
    let mut custom = pick("claude-code");
    custom.model = "some-unlisted-preview".to_string();
    assert!(matches!(
        plan(SEEDED_MODELS, TEMPLATE_PROVIDERS, &rows, &custom, None),
        Err(PickError::Incapable { .. })
    ));
    // `compactor` declares no `tools:` line at all in lernie's own template,
    // which is the emptiest role list there is.
    let mut bare_role = pick("claude-code");
    bare_role.role = "compactor".to_string();
    assert!(matches!(
        plan(SEEDED_MODELS, TEMPLATE_PROVIDERS, &rows, &bare_role, None),
        Err(PickError::Incapable { .. })
    ));
}
