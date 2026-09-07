//! **The box's own four** (bl-28f4): the material, the endpoint, the listener,
//! and the identity every substrate spawn signs with. Split from
//! [`doctor`](super) on the seam the gesture's own address draws — these answer
//! for a box with no workspace at all.

use super::Row;
use crate::boundary::dispatch::Deps;
use crate::wire::material::{self, ADDRESS, REMEDY, Role};

/// The four, in the order an operator meets them.
pub(super) fn rows(deps: &Deps) -> Vec<Row> {
    let dir = material::dir(&deps.world);
    vec![wire(&dir), address(&dir), listener(deps), git(deps)]
}

/// The material itself — [`read_dir`](material::read_dir)'s three answers, said
/// once. A box with nothing is not broken (the next boot mints it); a box with
/// half is, and the read's own sentence names every file that is missing.
fn wire(dir: &std::path::Path) -> Row {
    match material::read_dir(dir, Role::Server) {
        Ok(Some(_)) => Row::ok(
            "wire",
            format!("{} holds a CA and this box's leaves", dir.display()),
        ),
        Ok(None) => Row::bad(
            "wire",
            format!("{} holds no wire material", dir.display()),
            format!(
                "`{REMEDY}` mints it, and the engine's own boot mints it for a box that has none"
            ),
        ),
        Err(half) => Row::bad(
            "wire",
            half,
            format!(
                "`FORCE=1 {REMEDY}` re-founds the directory; a rotation distrusts every \
                 certificate already issued"
            ),
        ),
    }
}

/// The endpoint the engine binds and a seat dials. A `:0` is the request a
/// self-provisioning boot writes, and it is the one address nothing downstream
/// can use: no seat can dial it and no enrolment can put it in a QR.
fn address(dir: &std::path::Path) -> Row {
    let stated = std::fs::read_to_string(dir.join(ADDRESS))
        .unwrap_or_default()
        .trim()
        .to_owned();
    let dialable = stated
        .rsplit_once(':')
        .is_some_and(|(host, port)| !host.is_empty() && !port.is_empty() && port != "0");
    if dialable {
        return Row::ok(
            "address",
            format!("{} names {stated}", dir.join(ADDRESS).display()),
        );
    }
    Row::bad(
        "address",
        if stated.is_empty() {
            format!("{} names no address", dir.join(ADDRESS).display())
        } else {
            format!(
                "{stated} is a request, not an endpoint: only the listener learns what a `:0` \
                 became, and it becomes something else at the next boot"
            )
        },
        format!(
            "`WIRE_HOST=<host> WIRE_PORT=<port> {REMEDY}` states it over the CA already here, \
             distrusting nothing — then restart the engine"
        ),
    )
}

/// **What this process actually bound**, which is the fact `address` cannot
/// give: the file holds a request. It is also the answer the boot's stderr line
/// gives once and an operator under a supervisor cannot get back.
fn listener(deps: &Deps) -> Row {
    match deps.caller.listening.address() {
        Some(bound) => Row::ok("listener", format!("listening on {bound}")),
        None => Row::bad(
            "listener",
            "this answer did not come from a listening engine".to_owned(),
            "start `yog` — its boot says what it bound on stderr, and says there too when \
             another process won the port"
                .to_owned(),
        ),
    }
}

/// The identity every substrate spawn signs with. A box with none dies at the
/// first commit a start makes, with git's own words rather than a named
/// prerequisite — so the doctor asks before the conversation does.
fn git(deps: &Deps) -> Row {
    // **The file the REMEDY names is the file the check reads.** `HOME` and
    // `GIT_CONFIG_GLOBAL` are both stated, at this world's own home
    // ([`Deps::home`] — the home every substrate spawn signs under), so the row
    // answers about `~/.gitconfig`: the one `git config --global` writes and
    // the one the sentence below tells the operator to write. Inherited, an
    // ambient `GIT_CONFIG_GLOBAL` outranks `HOME` and the doctor would be
    // reporting on a file its own remedy does not touch. The system config is
    // left alone, so a box configured there still passes.
    let read = |key: &str| {
        crate::git_env::output(
            crate::git_env::git()
                .env("HOME", &deps.home)
                .env("GIT_CONFIG_GLOBAL", deps.home.join(".gitconfig"))
                .args(["config", "--get", key]),
        )
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .filter(|value| !value.is_empty())
    };
    match (read("user.name"), read("user.email")) {
        (Some(name), Some(_)) => Row::ok("git", format!("commits are authored as {name}")),
        _ => Row::bad(
            "git",
            "this box has no git identity".to_owned(),
            "`git config --global user.name \"<name>\"` and `git config --global user.email \
             <address>` — every workspace and every step is a commit"
                .to_owned(),
        ),
    }
}
