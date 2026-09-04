//! **Writing an executable file is a CHILD's job, never this process's**
//! (bl-fd28, generalized to production by bl-e6c9) — the whole of the ETXTBSY
//! discipline, in the one place the crate spells it.
//!
//! `fs::write` on a file holds a write fd for the length of the write. A `fork`
//! on ANY thread copies that fd into a child that keeps it until its own `exec`
//! completes, and an `exec` of that file inside the window is **ETXTBSY**. The
//! window cannot be closed from the fork side: [`super`]'s `cfg(test)` lock
//! covered only yog's own forks, and yog links `balls`, `litany` and `brazen`,
//! each of which forks `git` on its own account (measured, and measured out —
//! [`super`]'s module doc carries both numbers).
//!
//! bl-fd28 closed it on the side that owns it, for test fixtures. **The engine
//! had the same hazard and kept it**: `world::tools::ensure_shim` wrote the
//! world's shims with `fs::write` + `set_permissions` and a caller exec'd one
//! immediately after — yog composes a world's shims and then runs them, and yog
//! forks from every thread. Reproduced at bl-fd28's own recipe with the
//! `world::tools`/`world::tests` beats folded into the filter, 16 workers x 70
//! iterations: **7 ETXTBSY failures**, every one of them a shim exec.
//!
//! So there is one helper and it is production's: [`write_exec`]. The fd lives
//! in `sh`, which holds it for as long as `cat` runs and never for a moment in
//! this process, so a peer fork — in yog or in any crate yog links, at any
//! moment — has nothing of ours to copy. `rules/no-hand-chmod.yml` refuses the
//! hand-rolled spelling everywhere in `src`, so the discipline is structural
//! rather than a convention the next executable forgets.
//!
//! Two shapes were rejected. A **retry on ETXTBSY** turns a hazard into a
//! production loop and leaves the window open for whoever does not spin. A
//! **write-then-rename** does not work at all: a rename does not change the
//! inode, so the copied write fd still refers to the file the caller execs.
//!
//! The body goes down a pipe rather than an argv word, so nothing here has an
//! `ARG_MAX`. The pipe is [`io::pipe`] rather than `Stdio::piped()` for a
//! smaller reason that is worth the two `drop`s: `Child::stdin` is an `Option`
//! that can only be `Some` here, and an owned pipe has no unreachable arm to
//! answer for. A body large enough to fill the pipe buffer before `cat` drains
//! it would deadlock; a shim is a few hundred bytes and a fixture is a script.
//! `sh`'s stderr is captured rather than inherited, so the reason a write
//! failed rides the error instead of the caller's terminal.

use std::io::{self, Write as _};
use std::path::Path;
use std::process::Stdio;

/// The one recipe: read the body from stdin, then set the executable bits. Both
/// acts are the child's, which is the point — this process opens nothing.
const RECIPE: &str = r#"cat > "$1" && chmod 755 "$1""#;

/// Write `body` to `path` and mark it `0755` — **executable by all, writable
/// only by the owner** — entirely inside a child process. The crate's one way
/// to create an executable file, in production and in tests alike
/// (`crate::test_support::write_exec` is this function with the error turned
/// into a panic, which is all a fixture wants).
pub(crate) fn write_exec(path: &Path, body: &str) -> io::Result<()> {
    let (reader, mut pipe) = io::pipe()?;
    let mut cmd = super::command(Path::new("sh"));
    cmd.arg("-c")
        .arg(RECIPE)
        .arg("sh")
        .arg(path)
        .stdin(reader)
        .stderr(Stdio::piped());
    let child = super::spawn(&mut cmd)?;
    // The command still owns the read end it was handed; `cat` sees EOF only
    // once every copy is closed, so both this one and the writer must go before
    // the wait — otherwise the child never exits and neither does the caller.
    drop(cmd);
    let written = pipe.write_all(body.as_bytes());
    drop(pipe);
    let out = child.wait_with_output()?;
    written?;
    if out.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "writing the executable {}: sh exited {} — {}",
        path.display(),
        out.status,
        String::from_utf8_lossy(&out.stderr).trim()
    )))
}

#[cfg(test)]
mod tests {
    use super::write_exec;
    use std::os::unix::fs::PermissionsExt as _;

    /// The whole contract in one beat: the file lands with the body it was
    /// given, and it RUNS — the bit that makes it an executable is set.
    #[test]
    fn the_written_file_carries_the_body_and_runs() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("greet");
        write_exec(&script, "#!/bin/sh\nprintf 'hi %s' \"$1\"\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(&script).unwrap(),
            "#!/bin/sh\nprintf 'hi %s' \"$1\"\n"
        );
        assert_eq!(
            std::fs::metadata(&script).unwrap().permissions().mode() & 0o777,
            0o755
        );
        let out = crate::git_env::output(crate::git_env::command(&script).arg("there")).unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout), "hi there");
    }

    /// The child's failure is the caller's error, named by path. A redirect
    /// into a directory that does not exist is the cheapest one to stage, and
    /// it is the real shape of the failure — a tools dir nobody created.
    #[test]
    fn a_child_that_cannot_write_is_reported_with_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let nowhere = dir.path().join("absent").join("shim");
        let err = write_exec(&nowhere, "#!/bin/sh\n").unwrap_err();
        let said = err.to_string();
        assert!(said.contains(&nowhere.display().to_string()), "{said}");
        assert!(said.contains("sh exited"), "{said}");
        // The child's own reason rides back, which is the whole point of
        // capturing its stderr rather than letting it out on the terminal.
        assert!(
            said.split(" — ").nth(1).is_some_and(|why| !why.is_empty()),
            "{said}"
        );
    }
}
