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
//! - *Is it there?* The mailbox's own two readings — it is parked on its
//!   read (`serving`), or it is working on a job this server handed it and
//!   still believes in (`busy`). A parked machine is asked. A busy one cannot
//!   be — a foot runs one call at a time — so it answers with **what it last
//!   said to the same read**, the mailbox's one replica of it. A machine that
//!   is neither is asleep or gone: no rows, no wait.
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
use crate::boundary::reply::{Place, Placed};
use crate::bz_host::far::{TOOL, machines, split};
use crate::config_edit::brazen::{BzOutcome, ProviderRow, ProviderRowView, provider_rows};
use crate::registry::mailbox::{Call, Capture};
use crate::registry::{Client, tools};

/// How long a listing may take: 120 looks, 25 ms apart — three seconds.
const WAITS: u32 = 120;
const TICK: Duration = Duration::from_millis(25);

/// Read `<row>@<client>` off `provider` against the clients registered in
/// `workspace`: the row as that machine spells it, and the machine.
pub(crate) fn named(deps: &Deps, workspace: &Path, provider: &str) -> Option<Place> {
    let registered = machines(&deps.state_root, &crate::naming::leaf(workspace));
    split(provider, &registered).map(|(row, client)| Place { client, row })
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
        let far = named(deps, workspace, name).is_some_and(|at| advertises(deps, &at.client));
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

/// `bz <line>` as each of `clients` answers it, in the order asked. A machine
/// parked on its read is asked now, all at once, and what comes back inside
/// the bound is remembered; one that did not answer — it is busy, or it took
/// the ask and is still on it — answers with what it last said to this read,
/// while the mailbox believes it is working. The rest answer nothing, and
/// are not waited for.
fn asked(deps: &Deps, now: i64, clients: &[String], line: &[&str]) -> Vec<(String, Capture)> {
    let (mail, by) = (&deps.caller.mailbox, deps.caller.client.name());
    let mut open: Vec<(Call, Option<String>, Option<Capture>)> = clients
        .iter()
        .map(|client| {
            let call = Call {
                client: client.clone(),
                tool: TOOL.to_owned(),
                input: json!({ "args": line }),
                cwd: None,
            };
            let invocation = mail.serving(client).then(|| mail.post(now, &by, &call));
            (call, invocation, None)
        })
        .collect();
    for _ in 0..WAITS {
        for (call, invocation, got) in open.iter_mut().filter(|(_, _, got)| got.is_none()) {
            *got = invocation
                .as_deref()
                .and_then(|id| mail.collect(now, &by, id, None).ok())
                .and_then(|standing| standing.capture);
            if let Some(fresh) = got.as_ref() {
                mail.remember(now, call, fresh);
            }
        }
        if open
            .iter()
            .all(|(_, invocation, got)| invocation.is_none() || got.is_some())
        {
            break;
        }
        std::thread::sleep(TICK);
    }
    open.into_iter()
        .filter_map(|(call, _, got)| {
            let got = got.or_else(|| mail.known(now, &call))?;
            Some((call.client, got))
        })
        .collect()
}

/// One row listing (`TABLE` or `CATALOG`) from every machine of `workspace`
/// that advertises `bz` and is there — parked or busy — each row's name qualified and its two
/// halves stated beside it — the machine is known here, so no reader has to
/// read it back off the name. The rows describe **that machine's** credentials
/// as its own brazen reports them.
pub(crate) fn rows(
    deps: &Deps,
    now: i64,
    workspace: &Path,
    line: &[&str],
) -> Vec<Placed<ProviderRowView>> {
    let hosts: Vec<String> = machines(&deps.state_root, &crate::naming::leaf(workspace))
        .into_iter()
        .filter(|client| advertises(deps, client))
        .collect();
    asked(deps, now, &hosts, line)
        .into_iter()
        .flat_map(|(client, capture)| {
            provider_rows(&capture.stdout).into_iter().map(move |row| {
                let name = format!("{}@{client}", row.name);
                let at = Some(Place {
                    client: client.clone(),
                    row: row.name.clone(),
                });
                let row = ProviderRow { name, ..row }.view();
                Placed { row, at }
            })
        })
        .collect()
}

/// `bz <line>` as the one machine a name selected answers it — a row's model
/// listing — or the sentence saying why that machine did not. The asker named
/// the machine, so its silence is a refusal here where in [`rows`] it is an
/// absence; and **only a machine that is not there is refused as absent**: a
/// busy one answers from what it last said, and is called busy when it has
/// never said it.
pub(crate) fn run(deps: &Deps, now: i64, client: &str, line: &[&str]) -> Result<BzOutcome, String> {
    if !advertises(deps, client) {
        return Err(format!(
            "client {client:?} advertises no tool {TOOL:?} right now"
        ));
    }
    let mail = &deps.caller.mailbox;
    let parked = mail.serving(client);
    let (_, capture) = asked(deps, now, &[client.to_owned()], line)
        .pop()
        .ok_or_else(|| {
            if parked {
                format!(
                    "client {client:?} did not answer within {} seconds",
                    (TICK * WAITS).as_secs()
                )
            } else if mail.busy(now, client) {
                format!(
                    "client {client:?} is busy with another call, and it has not answered \
                     this read before — ask again when that call ends"
                )
            } else {
                format!(
                    "client {client:?} is not there right now — it is asleep, or gone; \
                     /clients says when it last connected"
                )
            }
        })?;
    Ok(BzOutcome {
        success: capture.exit_code == 0,
        stdout: capture.stdout,
        stderr: capture.stderr,
    })
}

#[cfg(test)]
mod tests;
