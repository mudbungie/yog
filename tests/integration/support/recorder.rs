//! The fake substrate **recorder** binary and its read-back parser (STORIES
//! "Test harness") — split out of [`super`] to stay under the 300-line cap.
//!
//! The recorder is the `editor_roundtrip` idiom generalized: a script
//! that appends argv+env+cwd to a NUL-delimited log and plays a canned
//! stdout/exit per verb, injected at the dispatch API as `Cli::new(path)`.

#![allow(dead_code)]
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// One recorded spawn of a fake binary: the exact argv **after** the binary
/// (arg 0 is the verb), the working directory the child ran in, and the env
/// vars it observed (the world-nesting set, when stood, plus `PATH`).
///
/// `cwd` is the **physical** (symlink-resolved) directory — the script records
/// `pwd -P`. Compare it against [`canon`] of the expected path, never against a
/// raw `TempDir` path: see [`canon`].
#[derive(Debug, Clone)]
pub struct Invocation {
    pub argv: Vec<String>,
    pub cwd: String,
    pub env: BTreeMap<String, String>,
}

/// The physical (symlink-resolved) form of `p` as a string — the only form a
/// recorded [`Invocation::cwd`] can be compared to.
///
/// A path is not a string: one directory has many spellings, and only the
/// canonical one is unique. On macOS a `tempfile::TempDir` lives under
/// `/var/folders/…` and `/var` is a symlink to `/private/var`, so the logical
/// path a test hands to the spawn and the physical path the child's `pwd -P`
/// reports name one directory by two strings. Canonicalizing **both** sides
/// compares directories rather than spellings; it is a no-op where the logical
/// and physical spellings already agree (Linux CI). Mirrors the
/// `cli_outbound` unit tests, which canonicalize both sides for the same reason.
pub fn canon(p: &Path) -> String {
    fs::canonicalize(p).unwrap().to_string_lossy().into_owned()
}

/// A fake substrate binary: records every spawn to a NUL-delimited log and
/// plays a canned stdout/exit per verb. Built once per test, injected via
/// `Cli::new(recorder.path())`.
pub struct Recorder {
    path: PathBuf,
    log: PathBuf,
    cases: Vec<(String, String, String, i32)>,
    new_arm: bool,
}

/// litany's own `template/providers.yaml` (the pinned engine): the whole
/// worker tool pool, `message` + `dispatch` included — yog grants nothing on
/// top (§8.1, bl-7fc8).
pub const TEMPLATE_PROVIDERS: &str = "roles:\n  worker:\n    provider: anthropic\n    \
     model: claude-sonnet-5\n    tools: [apply_patch, bash, cd, dispatch, load_skill, message, \
     multi_tool, read_file]\n  compactor:\n    provider: anthropic\n    model: claude-haiku-4-5\n";

/// The `new)` arm [`Recorder::authoring_workspaces`] injects: `git init --bare
/// -b config/default`, then the orphan first config commit carrying the
/// template `providers.yaml` — litany `template::scaffold` in shell.
const AUTHOR_WORKSPACE: &str = r#"new)
  d=$2
  mkdir -p "$d/repo.git"
  git init -q --bare -b config/default "$d/repo.git"
  git -C "$d/repo.git" worktree add -q --orphan -b config/default "$d/.author"
  printf '%s' '__PROVIDERS__' > "$d/.author/providers.yaml"
  git -C "$d/.author" add -A
  git -C "$d/.author" -c user.email=t@t.local -c user.name=T -c commit.gpgsign=false \
    commit -q -m 'config: init [config/default]'
  git -C "$d/repo.git" worktree remove "$d/.author"
  exit 0;;
"#;

impl Recorder {
    /// A recorder named `name` (`litany`/`bl`/`bz`) writing its log beside the
    /// script under `dir`. With no cases every verb defaults to exit 0, empty
    /// stdout.
    pub fn new(dir: &Path, name: &str) -> Self {
        Self {
            path: dir.join(name),
            log: dir.join(format!("{name}.log")),
            cases: Vec::new(),
            new_arm: false,
        }
    }

    /// Canned response for a verb (`$1`): print `stdout`, exit `code`.
    #[must_use]
    pub fn on(self, verb: &str, stdout: &str, code: i32) -> Self {
        self.on_err(verb, stdout, "", code)
    }

    /// Make the `new` verb **materialize** the workspace litany's own `new`
    /// authors (ARCH §2.2): a bare `repo.git` on `config/default` whose one
    /// orphan-root commit carries the template `providers.yaml`. Any start
    /// whose flow reads the workspace back needs it — a `new` that only
    /// recorded itself would leave nothing on disk to read.
    #[must_use]
    pub fn authoring_workspaces(mut self) -> Self {
        self.new_arm = true;
        self
    }

    /// Canned response with a `stderr` body too — a failed verb whose diagnostic
    /// must ride back verbatim (gate/close failures, a failed `prime`; §4.2/§8.2).
    #[must_use]
    pub fn on_err(mut self, verb: &str, stdout: &str, stderr: &str, code: i32) -> Self {
        self.cases
            .push((verb.to_owned(), stdout.to_owned(), stderr.to_owned(), code));
        self
    }

    /// Write the executable recorder script (0755) and return its path — the
    /// value handed to `Cli::new`.
    pub fn path(&self) -> PathBuf {
        let mut arms = String::new();
        if self.new_arm {
            arms.push_str(&AUTHOR_WORKSPACE.replace("__PROVIDERS__", TEMPLATE_PROVIDERS));
        }
        for (verb, stdout, stderr, code) in &self.cases {
            let out = stdout.replace('\'', "'\\''");
            let err = stderr.replace('\'', "'\\''");
            let _ = writeln!(
                arms,
                "{verb}) printf '%s' '{out}'; printf '%s' '{err}' 1>&2; exit {code};;"
            );
        }
        let script = SCRIPT
            .replace("__LOG__", &self.log.to_string_lossy())
            .replace("__ARMS__", &arms);
        // Never a bare `fs::write`: the script is about to be exec'd, and a
        // write fd on it in *this* process is the ETXTBSY race peer test
        // threads lose (see `tests/support/write_exec.rs`).
        super::write_exec::write_exec(&self.path, &script);
        self.path.clone()
    }

    /// Every recorded spawn, oldest-first.
    pub fn invocations(&self) -> Vec<Invocation> {
        parse(&fs::read(&self.log).unwrap_or_default())
    }

    /// Poll until at least `n` invocations are recorded (a **detached** child —
    /// the `litany prompt` — writes its log asynchronously), up to a ~2 s bound,
    /// then return them. Sequential piped verbs are already present on entry.
    pub fn wait(&self, n: usize) -> Vec<Invocation> {
        for _ in 0..200 {
            let inv = self.invocations();
            if inv.len() >= n {
                return inv;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        self.invocations()
    }
}

/// The recorder script body: append this spawn's physical cwd (`C`, `pwd -P` —
/// `pwd` alone may echo an inherited logical `PWD`), each argument
/// (`A`), and each present world-env var + `PATH` (`E`) as NUL-terminated
///
/// **The var list is what this probe can see, so a var missing from it is a
/// fact no test in this crate can assert** (bl-bf79): `YOG_WALL` was absent,
/// and every workspace-bound spawn asserted here was therefore asserted blind
/// to whether it carried the workspace's wall at all. Adding a var to a spawn
/// means adding it here, or the coverage is a coverage of the other vars.
/// tagged fields, close the record (`Z`), then play the canned per-verb
/// response. `printf` only — no fork to race the coverage ptrace engine.
const SCRIPT: &str = r#"#!/bin/sh
{
  printf 'C%s\000' "$(pwd -P)"
  for a in "$@"; do printf 'A%s\000' "$a"; done
  for k in LITANY_HOME XDG_STATE_HOME YOG_WALL YOG_NAME EDITOR YOG_EDIT_SRC PATH; do
    eval "v=\${$k-}"
    [ -n "$v" ] && printf 'E%s=%s\000' "$k" "$v"
  done
  printf 'Z\000'
} >> '__LOG__'
case "${1-}" in
__ARMS__*) exit 0;;
esac
"#;

/// Parse the NUL-delimited recorder log into ordered [`Invocation`]s. Fields are
/// NUL-terminated and tagged by their first byte — `C`wd, `A`rgument, `E`nv
/// `KEY=VAL`, `Z` (record terminator). The trailing empty field after the final
/// NUL matches no tag and falls through.
fn parse(bytes: &[u8]) -> Vec<Invocation> {
    let mut out = Vec::new();
    let (mut argv, mut cwd, mut env) = (Vec::new(), String::new(), BTreeMap::new());
    for field in bytes.split(|&b| b == 0) {
        let field = String::from_utf8_lossy(field);
        let field: &str = &field;
        if let Some(a) = field.strip_prefix('A') {
            argv.push(a.to_owned());
        } else if let Some(c) = field.strip_prefix('C') {
            c.clone_into(&mut cwd);
        } else if let Some(kv) = field.strip_prefix('E') {
            if let Some((k, v)) = kv.split_once('=') {
                env.insert(k.to_owned(), v.to_owned());
            }
        } else if field == "Z" {
            out.push(Invocation {
                argv: std::mem::take(&mut argv),
                cwd: std::mem::take(&mut cwd),
                env: std::mem::take(&mut env),
            });
        }
    }
    out
}
