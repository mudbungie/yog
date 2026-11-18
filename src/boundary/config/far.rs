//! **A registered machine's brazen, asked by the server's own reads** (REMOTE
//! §5.7, *The listings and the sign-in are the same runner*; bl-7467).
//!
//! A provider's name is `<row>` — this server's linked brazen — or
//! `<row>@<client>`: that row on a machine registered in the workspace. The
//! `bz` arm routes the calls a *child* makes ([`crate::bz_host::far`]); this
//! is the same reading of the same name for the reads the server answers in
//! its own process, which hold the mailbox by handle and so post to it
//! directly. Nothing here deposits a gesture: an answer that waited on the
//! `gestures/` inbox from the thread that consumes it would wait on itself.
//!
//! **Three questions, three predicates.**
//!
//! - *Which machine does this name select?* [`named`] — the workspace's
//!   registered clients, the longest that fits. The arm's own reading.
//! - *May a config name it?* [`judged`] — the machine is registered here and
//!   advertises `bz`. Both are files (§5), so the answer does not depend on
//!   whether the machine is awake, and the row half is that machine's to
//!   refuse at the call.
//! - *Can it answer a listing now?* The mailbox's own `serving` — it is parked
//!   on its read at this moment. A machine that is asleep or busy contributes
//!   no rows and costs no wait.
//!
//! **The wait is bounded and paid on the answering thread** — three seconds
//! for every machine asked at once. A foot's `complete` arrives on that foot's
//! own connection thread (`wire::listener`), never on the one that waits here,
//! so the wait cannot hold up the post it is waiting for. Asked through the
//! `gestures/` inbox the thread is the deposit consumer, and the cost is the
//! one `invocations` already states: other deposits converge after it.
//!
//! **A listing nobody collected is left where it stands.** Its machine posts
//! the capture to a slot no one polls again, and the mailbox's hour sweep
//! takes it; there is no cancel on this leg and none is added.

use std::path::Path;
use std::time::Duration;

use serde_json::json;

use crate::boundary::dispatch::Deps;
use crate::bz_host::far::{TOOL, machines, split};
use crate::config_edit::brazen::{BzOutcome, ProviderRow, provider_rows};
use crate::registry::mailbox::{Call, Capture};
use crate::registry::{Client, tools};

/// How long a listing may take: 120 looks, 25 ms apart — three seconds.
const WAITS: u32 = 120;
const TICK: Duration = Duration::from_millis(25);

/// Read `<row>@<client>` off `provider` against the clients registered in
/// `workspace`: the row as that machine spells it, and the machine.
pub(crate) fn named(deps: &Deps, workspace: &Path, provider: &str) -> Option<(String, String)> {
    let registered = machines(&deps.state_root, &crate::naming::leaf(workspace));
    split(provider, &registered)
}

/// Does `client` advertise the `bz` tool — the durable half of *can serve*.
fn advertises(deps: &Deps, client: &str) -> bool {
    Client::parse(client).is_ok_and(|client| {
        tools::read(&deps.state_root, &client)
            .iter()
            .any(|tool| tool.name == TOOL)
    })
}

/// **A qualified name is judged by its qualification**: `rows` with one more
/// row for each of `names` that selects a machine registered here and
/// advertising `bz`, so every gate that asks *does this row exist* (§9.4's
/// pick, §9.5's provider control, the `prompt` door) accepts it awake or not.
/// The added row states nothing but its name — no credential it lacks, no
/// capability it was asked about — because none of that was asked.
///
/// An empty table is still no answer (brazen could not be asked), and stays
/// empty: a table of one stand-in would refuse every bare row beside it.
pub(crate) fn judged(
    deps: &Deps,
    workspace: &Path,
    mut rows: Vec<ProviderRow>,
    names: &[&str],
) -> Vec<ProviderRow> {
    if rows.is_empty() {
        return rows;
    }
    for name in names {
        let far = named(deps, workspace, name).is_some_and(|(_, client)| advertises(deps, &client));
        if far && !rows.iter().any(|row| row.name == *name) {
            rows.push(ProviderRow {
                name: (*name).to_owned(),
                protocol: String::new(),
                auth: String::new(),
                credential: String::new(),
                effort: false,
                priority: false,
                tools: None,
                device: String::new(),
            });
        }
    }
    rows
}

/// Run `bz <line>` on each of `clients` that is parked on its read right now,
/// all at once, and answer what came back inside the bound — in the order
/// asked. A machine that is not serving is not asked.
fn asked(deps: &Deps, now: i64, clients: &[String], line: &[&str]) -> Vec<(String, Capture)> {
    let (mail, by) = (&deps.caller.mailbox, deps.caller.client.name());
    let mut open: Vec<(String, String, Option<Capture>)> = clients
        .iter()
        .filter(|client| mail.serving(client))
        .map(|client| {
            let call = Call {
                client: client.clone(),
                tool: TOOL.to_owned(),
                input: json!({ "args": line }),
                cwd: None,
            };
            (client.clone(), mail.post(now, &by, &call), None)
        })
        .collect();
    for _ in 0..WAITS {
        for (_, invocation, got) in open.iter_mut().filter(|(_, _, got)| got.is_none()) {
            *got = mail
                .collect(now, &by, invocation, None)
                .ok()
                .and_then(|standing| standing.capture);
        }
        if open.iter().all(|(_, _, got)| got.is_some()) {
            break;
        }
        std::thread::sleep(TICK);
    }
    open.into_iter()
        .filter_map(|(client, _, got)| Some((client, got?)))
        .collect()
}

/// One row listing (`TABLE` or `CATALOG`) from every machine of `workspace`
/// that advertises `bz` and is serving, each row's name qualified. The rows
/// describe **that machine's** credentials as its own brazen reports them.
pub(crate) fn rows(deps: &Deps, now: i64, workspace: &Path, line: &[&str]) -> Vec<ProviderRow> {
    let hosts: Vec<String> = machines(&deps.state_root, &crate::naming::leaf(workspace))
        .into_iter()
        .filter(|client| advertises(deps, client))
        .collect();
    asked(deps, now, &hosts, line)
        .into_iter()
        .flat_map(|(client, capture)| {
            provider_rows(&capture.stdout)
                .into_iter()
                .map(move |row| ProviderRow {
                    name: format!("{}@{client}", row.name),
                    ..row
                })
        })
        .collect()
}

/// `bz <line>` run on the one machine a name selected — a row's model listing,
/// or the catalog a sign-in's flow is read off — or the sentence saying why
/// that machine did not answer. The asker named the machine, so its silence
/// is a refusal here where in [`rows`] it is an absence.
pub(crate) fn run(deps: &Deps, now: i64, client: &str, line: &[&str]) -> Result<BzOutcome, String> {
    if !advertises(deps, client) {
        return Err(format!(
            "client {client:?} advertises no tool {TOOL:?} right now"
        ));
    }
    if !deps.caller.mailbox.serving(client) {
        return Err(format!(
            "client {client:?} is not waiting for work right now — it is asleep, or busy \
             with another call; /clients says when it last connected"
        ));
    }
    let (_, capture) = asked(deps, now, &[client.to_owned()], line)
        .pop()
        .ok_or_else(|| {
            format!(
                "client {client:?} did not answer within {} seconds",
                (TICK * WAITS).as_secs()
            )
        })?;
    Ok(BzOutcome {
        success: capture.exit_code == 0,
        stdout: capture.stdout,
        stderr: capture.stderr,
    })
}

#[cfg(test)]
mod tests;
