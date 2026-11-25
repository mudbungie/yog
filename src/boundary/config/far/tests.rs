//! The server's reads against a registered machine's brazen (REMOTE §5.7,
//! bl-7467), asked through the real chokepoint over a real mailbox: a fixture
//! foot parks on its read and answers canned brazen listings.

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::json;
use tempfile::tempdir;

use super::super::tests::{ask, quiet};
use super::*;
use crate::boundary::Query;
use crate::boundary::config::Read;
use crate::boundary::dispatch::Caller;
use crate::boundary::reply::{ConfigAnswer, Place, Placed, Reply};
use crate::config_edit::brazen::{CATALOG, ProviderRowView, TABLE, provider_rows};
use crate::registry::mailbox::Mailbox;
use crate::test_support::foot::{mailbox, parked, printed, seat, serve};

/// One far listing: an oauth row that is signed in and declares a device
/// endpoint, in brazen's own `--list-providers --json` spelling.
const LISTING: &str = r#"{"providers":[{"name":"sub","protocol":"anthropic_messages",
    "auth":"oauth2","credential":"stored","effort":true,"device":"rfc8628"}]}"#;

/// A machine that is working, or between jobs, is asked (bl-3b4b, bl-d9ad).
mod busy;
/// A pushing host is never busy (REMOTE §5.8).
mod push;

fn ws() -> PathBuf {
    crate::test_support::fixture_workspace()
}

fn name() -> String {
    crate::naming::leaf(&ws())
}

/// The fixture world with `laptop` registered in its workspace advertising
/// `bz`, over a mailbox the test's foot shares.
fn world(root: &Path) -> (Deps, Mailbox) {
    let mail = mailbox();
    let deps = Deps {
        caller: Caller {
            mailbox: mail.clone(),
            ..Caller::default()
        },
        ..quiet(root)
    };
    seat(&deps.state_root, "laptop", &name(), &["bz"]);
    (deps, mail)
}

fn at(client: &str, row: &str) -> Place {
    let (client, row) = (client.to_owned(), row.to_owned());
    Place { client, row }
}

fn rows_of(reply: Result<Reply, String>) -> Vec<Placed<ProviderRowView>> {
    match reply {
        Ok(Reply::Config(ConfigAnswer::Providers(rows) | ConfigAnswer::Catalog(rows))) => rows,
        other => panic!("a listing answers rows: {other:?}"),
    }
}

fn providers(deps: &Deps) -> Vec<Placed<ProviderRowView>> {
    rows_of(ask(
        deps,
        &Query::Config(Read::Providers { workspace: name() }),
    ))
}

fn models(deps: &Deps, provider: &str) -> Result<Reply, String> {
    ask(
        deps,
        &Query::Config(Read::Models {
            workspace: name(),
            provider: provider.to_owned(),
        }),
    )
}

/// The server's own rows, then the serving machine's with their names
/// qualified — that machine's credential fact, in the one row shape — and the
/// catalog likewise. What crossed is the same listing command line. **Each
/// far row states its machine and its own spelling beside the name** (bl-471d)
/// and the server's own state neither.
#[test]
fn a_serving_machine_appends_its_rows_qualified() {
    let root = tempdir().unwrap();
    let (deps, mail) = world(root.path());
    let (foot, asked) = serve(&mail, "laptop", vec![Some(printed(LISTING)); 2]);

    parked(&mail, "laptop");
    let rows = providers(&deps);
    assert_eq!(rows.first().map(|row| row.name.as_str()), Some("acme"));
    assert_eq!(rows.first().and_then(|row| row.at.clone()), None);
    assert_eq!(
        rows.last(),
        Some(&Placed {
            row: ProviderRowView {
                name: "sub@laptop".to_owned(),
                fact: "auth oauth2 · signed in".to_owned(),
                blocked: None,
                effort: true,
                priority: false,
            },
            at: Some(at("laptop", "sub")),
        })
    );
    assert_eq!(asked.recv().unwrap(), json!({ "args": TABLE }));

    parked(&mail, "laptop");
    let catalog = rows_of(ask(
        &deps,
        &Query::Config(Read::Catalog { workspace: name() }),
    ));
    assert!(
        catalog
            .iter()
            .any(|row| row.name == "sub@laptop" && row.at == Some(at("laptop", "sub")))
    );
    assert_eq!(asked.recv().unwrap(), json!({ "args": CATALOG }));
    foot.join().unwrap();
}

/// A machine that is asleep — registered, advertising, neither parked on its
/// read nor working — is not asked: no rows, no wait, and nothing left queued
/// for it. Nor is
/// one that is awake and advertises no `bz`.
#[test]
fn a_machine_that_is_not_serving_adds_no_rows_and_no_wait() {
    let root = tempdir().unwrap();
    let (deps, mail) = world(root.path());
    seat(&deps.state_root, "phone", &name(), &["shell"]);
    let _reading = mail.reading("phone").unwrap();

    let before = Instant::now();
    let rows = providers(&deps);
    assert!(before.elapsed().as_secs() < 2, "nobody was waited for");
    assert_eq!(
        rows.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(),
        ["acme"]
    );
    let asleep = models(&deps, "sub@laptop").unwrap_err();
    assert!(asleep.contains("\"laptop\" is not there"), "{asleep}");
}

/// `models` naming a machine runs the listing there, with the row
/// unqualified; that brazen's own failure is the refusal, in its words.
#[test]
fn a_qualified_model_listing_runs_on_the_machine_it_names() {
    let root = tempdir().unwrap();
    let (deps, mail) = world(root.path());
    let failed = Capture {
        stdout: String::new(),
        stderr: "bz: provider `sub` is not signed in\n".to_owned(),
        exit_code: 77,
    };
    let listed = printed(r#"{"models":[{"id":"big","default":true},{"id":"small"}]}"#);
    let (foot, asked) = serve(&mail, "laptop", vec![Some(listed), Some(failed)]);

    parked(&mail, "laptop");
    assert_eq!(
        models(&deps, "sub@laptop"),
        Ok(Reply::Config(ConfigAnswer::Models(vec![
            "big".to_owned(),
            "small".to_owned()
        ])))
    );
    assert_eq!(
        asked.recv().unwrap(),
        json!({ "args": ["--list-models", "--provider", "sub", "--json"] })
    );
    parked(&mail, "laptop");
    assert_eq!(
        models(&deps, "sub@laptop"),
        Err("bz: provider `sub` is not signed in".to_owned())
    );
    foot.join().unwrap();
}

/// A machine that took the call and said nothing inside the bound has the read
/// queued on it, and the sentence says so; one that advertises no `bz` is
/// refused before any wait.
#[test]
fn a_machine_that_does_not_answer_is_named() {
    let root = tempdir().unwrap();
    let (deps, mail) = world(root.path());
    seat(&deps.state_root, "phone", &name(), &["shell"]);
    let (foot, _asked) = serve(&mail, "laptop", vec![None]);

    parked(&mail, "laptop");
    let silent = models(&deps, "sub@laptop").unwrap_err();
    assert!(
        silent.contains("queued, not refused") && silent.contains("\"laptop\""),
        "{silent}"
    );
    let stale = models(&deps, "sub@phone").unwrap_err();
    assert!(stale.contains("advertises no tool \"bz\""), "{stale}");
    foot.join().unwrap();
}

/// **A qualified name is judged by its qualification**: registered here and
/// advertising `bz` is the whole test, awake or not. A machine that advertises
/// no `bz`, one registered nowhere here, and a bare row gain nothing; a name
/// already in the table is not doubled; and an unanswered table stays one.
#[test]
fn the_config_gates_accept_a_row_on_a_machine_that_serves_bz() {
    let root = tempdir().unwrap();
    let (deps, _mail) = world(root.path());
    seat(&deps.state_root, "phone", &name(), &["shell"]);
    let table = provider_rows(LISTING);
    let names = ["x@laptop", "x@phone", "x@desktop", "bare", "sub"];
    let judged_names: Vec<String> = judged(&deps, &ws(), table, &names)
        .into_iter()
        .map(|row| row.name)
        .collect();
    assert_eq!(judged_names, ["sub", "x@laptop"]);
    assert!(judged(&deps, &ws(), Vec::new(), &names).is_empty());
    assert_eq!(named(&deps, &ws(), "x@laptop"), Some(at("laptop", "x")));
}
