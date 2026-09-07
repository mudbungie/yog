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
    assert_eq!(verdict(&keyless, &[], WORKER_ROLE), Some(Unready::Wall));
    assert_eq!(
        verdict(&[row("acme", "api_key", "stored")], &[], WORKER_ROLE),
        None
    );
    assert_eq!(
        verdict(
            &[row(
                "acme",
                "api_key",
                "a-spelling-this-build-never-heard-of"
            )],
            &[],
            WORKER_ROLE
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
        verdict(&rows, &roles, WORKER_ROLE),
        Some(Unready::Uncredentialed {
            row: rows[1].clone(),
        })
    );
    // …and the same wall readies once the row the roles name holds one.
    let signed = [rows[0].clone(), row("anthropic", "api_key", "stored")];
    assert_eq!(verdict(&signed, &roles, WORKER_ROLE), None);
}

/// A keyless row a role names is the operator's own hand, so it readies the
/// wall; `missing` is the one spelling that refuses, whoever named the row.
#[test]
fn a_role_naming_a_keyless_row_readies_the_wall() {
    let rows = [row("ollama", "none", NOT_REQUIRED)];
    assert_eq!(
        verdict(&rows, &[role("worker", "ollama")], WORKER_ROLE),
        None
    );
    assert_eq!(verdict(&rows, &[], WORKER_ROLE), Some(Unready::Wall));
}

/// **The fired role is the whole scope** — for a start that names none, that
/// is `worker`, because this door births roots and roots are workers. A
/// `compactor` or a `reviewer` pointing at a row that is not here does not
/// refuse the fire: litany's shipped template declares the reviewer unbound,
/// so refusing on it would refuse a wall that runs for hours over a role
/// nothing dispatches. Those fail at their own moment, with a cause.
#[test]
fn only_the_fired_role_gates_the_fire() {
    let rows = [row("acme", "api_key", "stored")];
    assert_eq!(
        verdict(
            &rows,
            &[
                role("worker", "acme"),
                role("compactor", "gone"),
                role("reviewer", "also-gone"),
            ],
            WORKER_ROLE
        ),
        None
    );
    // …and a wall whose roles name only rows that are not here refuses on the
    // worker's, whatever else the file declares.
    assert_eq!(
        verdict(
            &rows,
            &[role("compactor", "acme"), role("worker", "gone")],
            WORKER_ROLE
        ),
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
        WORKER_ROLE,
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
        WORKER_ROLE,
    );
    assert!(oauth.ends_with("/login openai-chatgpt"), "{oauth}");
    // A row that is not here at all is not a sign-in (bl-21e9): the remedy is
    // the table, and the sentence never says `/login`.
    let absent = refusal::say(
        &Unready::Undeclared {
            provider: "claude-session-direct".to_owned(),
        },
        &rows,
        WORKER_ROLE,
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
    let wall = refusal::say(&Unready::Wall, &rows, WORKER_ROLE);
    assert!(wall.starts_with("sign in first"), "{wall}");
    assert!(
        wall.ends_with(
            "sign in with /login: openai-chatgpt; \
                        write a key with /config brazen: anthropic"
        ),
        "{wall}"
    );
}

/// **The wall's rows are partitioned by the act each one takes** (bl-8523).
/// The flat list offered `/login <provider>` over all of them — including the
/// two brazen ships keyless, where `/login` answers *nothing to log in*, and
/// one of those two can serve no role either, so it could ready a wall by no
/// act at all and was offered anyway. Four groups, in the order an operator
/// should read them, and a group with no rows is not printed.
#[test]
fn the_wall_offers_each_row_the_act_that_would_ready_it() {
    let mut rows = vec![
        row("openai-chatgpt", "oauth2", "missing"),
        row("anthropic", "api_key", "missing"),
        row("google", "api_key", "missing"),
        row("ollama", "none", NOT_REQUIRED),
        row("claude-code", "none", NOT_REQUIRED),
    ];
    // `claude_code` declares no tools, so that row can serve no role — the
    // column brazen answers, not a name this file knows.
    rows[4].tools = Some(false);
    rows[4].protocol = "claude_code".to_owned();
    let said = refusal::say(&Unready::Wall, &rows, WORKER_ROLE);
    assert!(
        said.ends_with(
            "sign in with /login: openai-chatgpt; \
             write a key with /config brazen: anthropic, google; \
             name in a role with /model <role> <provider> <model-id>: ollama; \
             can serve no role: claude-code"
        ),
        "{said}"
    );
    // A wall with one kind of row prints one group and no empty ones.
    let one = refusal::say(&Unready::Wall, &rows[3..4], WORKER_ROLE);
    assert!(
        one.ends_with("name in a role with /model <role> <provider> <model-id>: ollama"),
        "{one}"
    );
}

/// **A start that names a role is judged on THAT role** (bl-9ced, litany
/// 0.0.12's `prompt --role`). The two directions are the whole defect the
/// threading closes: a `planner` start whose row is empty must be refused even
/// though `worker`'s row is signed in, and a `planner` start whose row is
/// signed in must pass even though `worker`'s is not. Reading `worker` in
/// either case is a verdict about a row the fire never resolves.
#[test]
fn the_role_the_start_names_is_the_one_the_door_reads() {
    let rows = [
        row("anthropic", "api_key", "stored"),
        row("openai-chatgpt", "oauth2", "missing"),
    ];
    let roles = [
        role("worker", "anthropic"),
        role("planner", "openai-chatgpt"),
    ];
    assert_eq!(
        verdict(&rows, &roles, WORKER_ROLE),
        None,
        "the worker is fine"
    );
    assert_eq!(
        verdict(&rows, &roles, "planner"),
        Some(Unready::Uncredentialed {
            row: rows[1].clone(),
        }),
        "and the planner is not"
    );
    // The mirror: a signed-in planner passes over an empty worker.
    let flipped = [
        row("anthropic", "api_key", "missing"),
        row("openai-chatgpt", "oauth2", "stored"),
    ];
    assert_eq!(verdict(&flipped, &roles, "planner"), None);
    assert_eq!(
        verdict(&flipped, &roles, WORKER_ROLE),
        Some(Unready::Uncredentialed {
            row: flipped[0].clone(),
        })
    );
}

/// And the sentence names that role, not `worker` — `/model worker …` would
/// send the operator to fix a row the start never resolves.
#[test]
fn a_role_shaped_refusal_names_the_role_that_fired() {
    let rows = [row("anthropic", "api_key", "missing")];
    let said = refusal::say(
        &Unready::Uncredentialed {
            row: rows[0].clone(),
        },
        &rows,
        "planner",
    );
    assert!(
        said.contains("`planner` resolves provider `anthropic`"),
        "{said}"
    );
    let absent = refusal::say(
        &Unready::Undeclared {
            provider: "gone".to_owned(),
        },
        &rows,
        "planner",
    );
    assert!(
        absent.contains("/model planner <provider> <model-id>"),
        "{absent}"
    );
}
