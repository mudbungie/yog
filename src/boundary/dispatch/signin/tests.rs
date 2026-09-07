//! The door's fold and its three sentences, each on its own row.

use super::*;
use crate::config_edit::brazen::NOT_REQUIRED;

fn row(name: &str, auth: &str, credential: &str) -> ProviderRow {
    ProviderRow {
        name: name.to_owned(),
        protocol: String::new(),
        auth: auth.to_owned(),
        credential: credential.to_owned(),
        effort: false,
        priority: false,
        tools: Some(true),
        device: String::new(),
    }
}

fn role(role: &str, provider: &str) -> RoleModel {
    RoleModel {
        role: role.to_owned(),
        provider: provider.to_owned(),
        model: "m".to_owned(),
        effort: None,
        priority: false,
    }
}

/// A wall whose lineage declares no worker: the bl-2291 predicate, unchanged.
/// Any credential of any spelling readies it, and a keyless row does not —
/// brazen merges its built-ins under every config, so a predicate they
/// satisfied would be one nothing ever fails.
#[test]
fn a_lineage_that_declares_no_worker_falls_back_to_the_wall() {
    let keyless = [
        row("ollama", "none", NOT_REQUIRED),
        row("acme", "api_key", "missing"),
    ];
    assert_eq!(verdict(&keyless, &[]), Some(Unready::Wall));
    assert_eq!(verdict(&[row("acme", "api_key", "stored")], &[]), None);
    assert_eq!(
        verdict(
            &[row(
                "acme",
                "api_key",
                "a-spelling-this-build-never-heard-of"
            )],
            &[]
        ),
        None,
        "every spelling but missing and not-required is a credential"
    );
}

/// **The defect** (bl-58e7): a credentialed row the roles do not name readies
/// nothing. The old fold answered `any row is credentialed` and let the fire
/// through; the conversation was born dead on the row the roles DO name.
#[test]
fn a_credentialed_row_the_roles_do_not_name_readies_nothing() {
    let rows = [
        row("sideline", "bearer", "ambient"),
        row("anthropic", "api_key", "missing"),
    ];
    let roles = [role("worker", "anthropic")];
    assert_eq!(
        verdict(&rows, &roles),
        Some(Unready::Uncredentialed {
            row: rows[1].clone(),
        })
    );
    // …and the same wall readies once the row the roles name holds one.
    let signed = [rows[0].clone(), row("anthropic", "api_key", "stored")];
    assert_eq!(verdict(&signed, &roles), None);
}

/// A keyless row a role names is the operator's own hand, so it readies the
/// wall; `missing` is the one spelling that refuses, whoever named the row.
#[test]
fn a_role_naming_a_keyless_row_readies_the_wall() {
    let rows = [row("ollama", "none", NOT_REQUIRED)];
    assert_eq!(verdict(&rows, &[role("worker", "ollama")]), None);
    assert_eq!(verdict(&rows, &[]), Some(Unready::Wall));
}

/// **The worker is the whole scope** — this door births roots and roots are
/// workers. A `compactor` or a `reviewer` pointing at a row that is not here
/// does not refuse the fire: litany's shipped template declares the reviewer
/// unbound, so refusing on it would refuse a wall that runs for hours over a
/// role nothing dispatches. Those fail at their own moment, with a cause.
#[test]
fn only_the_worker_gates_the_fire() {
    let rows = [row("acme", "api_key", "stored")];
    assert_eq!(
        verdict(
            &rows,
            &[
                role("worker", "acme"),
                role("compactor", "gone"),
                role("reviewer", "also-gone"),
            ]
        ),
        None
    );
    // …and a wall whose roles name only rows that are not here refuses on the
    // worker's, whatever else the file declares.
    assert_eq!(
        verdict(&rows, &[role("compactor", "acme"), role("worker", "gone")]),
        Some(Unready::Undeclared {
            provider: "gone".to_owned(),
        })
    );
}

/// The three sentences. Each names its own act, and the two role-shaped ones
/// name the role and its provider — the whole of what the operator has to fix.
#[test]
fn each_refusal_names_its_own_act() {
    let rows = [
        row("openai-chatgpt", "oauth2", "missing"),
        row("anthropic", "api_key", "missing"),
    ];
    // A keyed row's secret is a value in the wall's config, never a sign-in.
    let keyed = refusal::say(
        &Unready::Uncredentialed {
            row: rows[1].clone(),
        },
        &rows,
    );
    assert!(keyed.starts_with("sign in first"), "{keyed}");
    assert!(
        keyed.contains("`worker` resolves provider `anthropic`"),
        "{keyed}"
    );
    assert!(keyed.ends_with("/config brazen"), "{keyed}");
    // An oauth row's is `bz --login`, which is the only row `/login` serves.
    let oauth = refusal::say(
        &Unready::Uncredentialed {
            row: rows[0].clone(),
        },
        &rows,
    );
    assert!(oauth.ends_with("/login openai-chatgpt"), "{oauth}");
    // A row that is not here at all is not a sign-in (bl-21e9): the remedy is
    // the table, and the sentence never says `/login`.
    let absent = refusal::say(
        &Unready::Undeclared {
            provider: "claude-session-direct".to_owned(),
        },
        &rows,
    );
    assert!(!absent.contains("/login"), "{absent}");
    assert!(absent.contains("declares no such row"), "{absent}");
    assert!(absent.contains("/config brazen"), "{absent}");
    assert!(
        absent.contains("/model worker <provider> <model-id>"),
        "{absent}"
    );
    assert!(
        absent.ends_with("(rows here: openai-chatgpt, anthropic)"),
        "{absent}"
    );
    // And the wall's own sentence stands where the lineage names no role.
    let wall = refusal::say(&Unready::Wall, &rows);
    assert!(wall.starts_with("sign in first"), "{wall}");
    assert!(
        wall.ends_with("(rows: openai-chatgpt, anthropic)"),
        "{wall}"
    );
}
