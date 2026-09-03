//! **The throwaway box `reconcile.sh` is driven over** (bl-4e3c) — a scratch
//! `$HOME`, a `deploy.env`, and shims for the four programs the reconciler and
//! `verify.sh` reach for: `curl`, `docker`, `systemctl` and `journalctl` (plus
//! `sleep`, so the proof's bounded eight seconds cost the suite nothing).
//!
//! Split from the beats at §12's budget, on the seam the `leak_gate` suite
//! already runs on: what a drive *sets up* is one subject, what it must then
//! *see* is another.
//!
//! **The scripts under test are the real ones, driven where they live** —
//! `scripts/deploy/reconcile.sh`, which resolves the real `verify.sh` beside
//! itself. Not even a copy: nothing here can pass by agreeing with an
//! out-of-date restatement, which is the discipline `leak_gate` keeps for the
//! scanner, and a script this suite never writes is a script it can never
//! collide with (see [`shims`]).
//!
//! **The box is a state machine, not a canned transcript.** `systemctl restart`
//! copies `deploy.env`'s `YOG_IMAGE` into the shim's notion of what container
//! runs, and `docker inspect`/`docker exec` answer from *that* — so a rollback
//! is proved by the box ending up serving the prior tag, not by a script having
//! been called. A tag named in `bad-tags` crash-loops, exactly as a bad release
//! does, which is what makes `verify.sh` fail for a real reason.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

/// This repository, whose deploy scripts are the subject.
pub fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The registry spelling `reconcile.sh` holds as its one constant.
pub const PACKAGE: &str = "ghcr.io/mudbungie/yog";

/// One shim, written as a `sh` script and made executable.
fn shim(bin: &Path, name: &str, body: &str) {
    let path = bin.join(name);
    crate::write_exec::write_exec(&path, &format!("#!/bin/sh\nset -eu\n{body}\n"));
}

/// **The shims are written ONCE for the whole binary**, inside this `OnceLock`'s
/// initializer, so after the first test they are never written again. That was
/// this file's own answer to ETXTBSY before bl-fd28 gave the tree a general one
/// (`tests/support/write_exec.rs`: the fixture is written by a child, so no
/// descriptor exists here for a peer fork to copy). Both hold; the `OnceLock`
/// also earns its keep as plain memoization. The per-test files below are only
/// ever `cat`-ed, never exec'd, so they are outside the class entirely.
///
/// `CARGO_TARGET_TMPDIR` rather than a leaked `TempDir`: cargo owns it, so
/// nothing is stranded in `/tmp` per run.
fn shims() -> &'static Path {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let bin = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("reconcile-shims");
        fs::create_dir_all(&bin).unwrap();
        write_shims(&bin);
        bin
    })
}

/// A scratch box seated on `image`, with the identity lines a real
/// `seat.sh` writes — they are not decoration, one beat proves they survive.
pub fn box_at(image: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("home/.config/yog")).unwrap();
    fs::write(
        root.join("home/.config/yog/deploy.env"),
        format!(
            "YOG_IMAGE={image}\n\
             GIT_AUTHOR_NAME=someone\nGIT_AUTHOR_EMAIL=someone@example.com\n\
             GIT_COMMITTER_NAME=someone\nGIT_COMMITTER_EMAIL=someone@example.com\n"
        ),
    )
    .unwrap();
    // The box boots already serving what it is seated on.
    fs::write(root.join("container-image"), format!("{image}\n")).unwrap();
    dir
}

fn write_shims(bin: &Path) {
    // Every shim logs its argv, so a beat can assert what was NOT done — which
    // is the whole of what a deferral is.
    let log = "printf '%s\\n' \"$*\" >> \"$BOX/calls\"";

    // The registry, answering body-then-status as `curl -w '\n%{http_code}'`
    // does — because "this package publishes nothing yet" and "this package
    // refuses you" want opposite responses and only the status separates them.
    shim(
        bin,
        "curl",
        &format!(
            "{log}\n\
             for a in \"$@\"; do case $a in\n\
               *'/token?'*)\n\
                 if [ -f \"$BOX/token.json\" ]; then cat \"$BOX/token.json\"; printf '\\n200'\n\
                 else printf '\\n401'; fi; exit 0 ;;\n\
               */tags/list)\n\
                 if [ -f \"$BOX/tags-status\" ]; then \
                   printf '\\n%s' \"$(cat \"$BOX/tags-status\")\"; exit 0; fi\n\
                 if [ -f \"$BOX/tags.json\" ]; then cat \"$BOX/tags.json\"; printf '\\n200'\n\
                 else printf '\\n404'; fi; exit 0 ;;\n\
             esac; done\n\
             printf '\\n000'"
        ),
    );

    // The container engine, answering from the box's own state.
    shim(
        bin,
        "docker",
        &format!(
            "{log}\n\
             image=$(cat \"$BOX/container-image\")\n\
             case ${{1:-}} in\n\
               pull) if [ -f \"$BOX/pull-fails\" ]; then exit 1; fi; exit 0 ;;\n\
               inspect) printf '%s\\n' \"$image\" ;;\n\
               exec)\n\
                 case $* in\n\
                   *'yog gesture'*) [ -f \"$BOX/workspaces.json\" ] || exit 1\n\
                     exec cat \"$BOX/workspaces.json\" ;;\n\
                   *'yog --version'*) printf 'yog %s\\n' \
                     \"$(printf '%s' \"${{image##*:}}\" | sed 's/-[0-9a-f]\\{{7,\\}}$//')\" ;;\n\
                   *wire/address*) printf 'the-wire\\n' ;;\n\
                   *s_client*) printf 'Server certificate\\n' ;;\n\
                 esac ;;\n\
             esac\n\
             exit 0"
        ),
    );

    // The user manager. A restart is what moves the box: it seats whatever
    // `deploy.env` currently names. A tag in `bad-tags` never comes up.
    shim(
        bin,
        "systemctl",
        &format!(
            "{log}\n\
             case $* in\n\
               *restart*)\n\
                 sed -n 's/^YOG_IMAGE=//p' \"$HOME/.config/yog/deploy.env\" | tail -n 1 \
                   > \"$BOX/container-image\" ;;\n\
               *is-active*)\n\
                 image=$(cat \"$BOX/container-image\")\n\
                 if [ -f \"$BOX/bad-tags\" ] && grep -qxF \"$image\" \"$BOX/bad-tags\"; then\n\
                   printf 'failed\\n'; exit 3\n\
                 fi\n\
                 printf 'active\\n' ;;\n\
             esac\n\
             exit 0"
        ),
    );
    shim(bin, "journalctl", "exit 0");
    // `verify.sh`'s one bounded sleep, which a test must not actually spend.
    shim(bin, "sleep", "exit 0");
}

/// Scripted registry answers: the anonymous pull token and the tag listing.
pub fn registry(dir: &TempDir, tags: &[&str]) {
    fs::write(dir.path().join("token.json"), "{\"token\":\"anon\"}").unwrap();
    let quoted: Vec<String> = tags.iter().map(|t| format!("\"{t}\"")).collect();
    fs::write(
        dir.path().join("tags.json"),
        format!(
            "{{\"name\":\"mudbungie/yog\",\"tags\":[{}]}}",
            quoted.join(",")
        ),
    )
    .unwrap();
}

/// The registry answering the tag listing with `status` and no body: `404` is
/// a package nobody has published to (the standing state until bl-6b96 builds
/// the push), `401`/`403` one that has gone private.
pub fn registry_answers(dir: &TempDir, status: &str) {
    fs::write(dir.path().join("token.json"), "{\"token\":\"anon\"}").unwrap();
    fs::write(dir.path().join("tags-status"), status).unwrap();
}

/// The engine's answer to `{"op":"workspaces"}`, as `yog gesture` prints it:
/// one compact JSON line. Omit the call entirely to model an engine that does
/// not answer at all.
pub fn boundary_says(dir: &TempDir, body: &str) {
    fs::write(dir.path().join("workspaces.json"), format!("{body}\n")).unwrap();
}

/// An idle world: rows, none of them running, and no `stale` note.
pub const IDLE: &str = "{\"ok\":true,\"kind\":\"workspaces\",\"rows\":\
    [{\"workspace\":\"home\",\"running\":false,\"agents\":2}]}";

/// Tags that crash-loop on this box, so `verify.sh` fails on them.
pub fn crash_loops(dir: &TempDir, tags: &[&str]) {
    fs::write(
        dir.path().join("bad-tags"),
        format!("{}\n", tags.join("\n")),
    )
    .unwrap();
}

/// One reconcile pass. Returns `(exit code, stdout + stderr)`.
pub fn reconcile(dir: &TempDir) -> (i32, String) {
    let root = dir.path();
    let path = format!(
        "{}:{}",
        shims().display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = yog::git_env::command(&repo().join("scripts/deploy/reconcile.sh"))
        .env("PATH", path)
        .env("HOME", root.join("home"))
        .env("BOX", root)
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code().unwrap_or(-1), said)
}

/// `deploy.env` as the pass left it.
pub fn deploy_env(dir: &TempDir) -> String {
    fs::read_to_string(dir.path().join("home/.config/yog/deploy.env")).unwrap()
}

/// The value of one `deploy.env` key, or `""` when the key is absent.
pub fn key(dir: &TempDir, name: &str) -> String {
    deploy_env(dir)
        .lines()
        .filter_map(|l| l.strip_prefix(&format!("{name}=")))
        .next_back()
        .unwrap_or_default()
        .to_owned()
}

/// The tag the box is actually serving now — the fact a rollback is proved by.
pub fn serving(dir: &TempDir) -> String {
    fs::read_to_string(dir.path().join("container-image"))
        .unwrap()
        .trim()
        .to_owned()
}

/// Every shim invocation, in order.
pub fn calls(dir: &TempDir) -> Vec<String> {
    fs::read_to_string(dir.path().join("calls"))
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}
