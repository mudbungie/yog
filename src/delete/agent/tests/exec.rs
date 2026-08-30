//! **The two spawns** — the unlogged dry-run census and the logged removal —
//! against a fake `litany` on disk. Split from the tables at §12's budget on
//! the seam between *what the gate decides* and *what the verb costs*: above is
//! pure over an already-answered forest, here a real process is forked and its
//! argv read back.

use super::super::{Cli, census, spawn};
use super::{CHILD, ROOT};
use crate::opslog;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

/// A fake `litany` script: logs `$@` beside itself, prints `stdout`, exits
/// `code` — the `delete/exec` fixture idiom.
struct FakeLitany {
    dir: TempDir,
}

impl FakeLitany {
    fn new(stdout: &str, stderr: &str, code: i32) -> Self {
        let dir = tempdir().unwrap();
        let log = dir.path().join("argv.log");
        let path = dir.path().join("litany");
        fs::write(
            &path,
            format!(
                "#!/bin/sh\necho \"$@\" > {}\nprintf '%s\\n' '{stdout}'\nprintf '%s' '{stderr}' 1>&2\nexit {code}\n",
                log.display()
            ),
        )
        .unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        Self { dir }
    }

    fn cli(&self) -> Cli {
        Cli::new(self.dir.path().join("litany"))
    }

    fn argv(&self) -> String {
        fs::read_to_string(self.dir.path().join("argv.log"))
            .unwrap_or_default()
            .trim()
            .to_owned()
    }

    fn ws(&self) -> PathBuf {
        let ws = self.dir.path().join("ws");
        fs::create_dir_all(&ws).unwrap();
        ws
    }
}

#[test]
fn the_census_is_the_dry_run_subtree_form() {
    let fx = FakeLitany::new(
        "would delete r-aa; descendants: 1 (r-aa-c-bb); pending deposits: 2",
        "",
        0,
    );
    let ws = fx.ws();
    let census = census(&fx.cli(), &ws, ROOT).unwrap();
    assert_eq!(census.descendants, [CHILD]);
    assert_eq!(census.pending_deposits, 2);
    assert_eq!(
        fx.argv(),
        format!("delete {} {ROOT} --children --dry-run", ws.display()),
        "the census asks the substrate, never re-derives"
    );
}

#[test]
fn a_declined_or_unreadable_census_fails_closed() {
    let declined = FakeLitany::new("", "not a workspace", 2);
    let ws = declined.ws();
    assert_eq!(
        census(&declined.cli(), &ws, ROOT).unwrap_err(),
        "not a workspace"
    );
    let garbled = FakeLitany::new("all good", "", 0);
    let ws = garbled.ws();
    assert_eq!(
        census(&garbled.cli(), &ws, ROOT).unwrap_err(),
        "unrecognized delete report: all good"
    );
    let gone = Cli::new(declined.dir.path().join("no-such-litany"));
    assert!(
        census(&gone, &ws, ROOT)
            .unwrap_err()
            .contains("No such file")
    );
}

#[test]
fn the_removal_is_the_logged_litany_verb_bare_or_subtree() {
    let state = tempdir().unwrap();
    let fx = FakeLitany::new("deleted r-aa; descendants: 0; pending deposits: 0", "", 0);
    let ws = fx.ws();
    let outcome = spawn(&fx.cli(), state.path(), "TS", &ws, ROOT, false).unwrap();
    assert!(outcome.ok());
    assert_eq!(
        fx.argv(),
        format!("delete {} {ROOT}", ws.display()),
        "bare: no subtree implied"
    );

    let armed = spawn(&fx.cli(), state.path(), "TS", &ws, ROOT, true).unwrap();
    assert!(armed.ok());
    assert_eq!(
        fx.argv(),
        format!("delete {} {ROOT} --children", ws.display())
    );

    let ops = opslog::tail(state.path(), 8);
    assert_eq!(ops.len(), 2, "each removal leaves its §4.2 row");
    assert_eq!(
        &ops[0].argv[1..],
        &["delete", &ws.display().to_string(), ROOT]
    );
    assert_eq!(ops[1].argv.last().map(String::as_str), Some("--children"));
}
