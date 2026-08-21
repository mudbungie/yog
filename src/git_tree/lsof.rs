//! macOS `lsof` liveness backend (DESIGN §10): a pure `lsof -F` parser, a
//! trait-injected runner seam, and a `#[cfg(target_os = "macos")]` spawn shim.
//!
//! Linux answers both liveness questions from `/proc` ([`super::fd_probe`],
//! [`super::lock_probe`]); macOS has no `/proc`, so it parses `lsof` output. Per
//! §10 the *parser is pure and platform-independent* — compiled and covered on
//! Linux from recorded fixtures — and **only** the spawn shim is `macos`; that
//! shim is the sole region tarpaulin excludes from the Linux denominator
//! (empirically confirmed: a `cfg(target_os = "macos")` region is not compiled
//! into the instrumented binary, so it cannot be counted).
//!
//! # `lsof -F` field grammar (argv `lsof -F pan -- <path>`)
//!
//! `-F` emits one `<tag><value>` line per field (`man lsof`, "OUTPUT FOR OTHER
//! PROGRAMS"). The tags we select and consume:
//!
//! - `p` — **process ID**; begins a *process set* (always the set's first line).
//! - `f` — **file descriptor**; begins a *file set* (one open file). `pan` does
//!   not name `f`, but lsof still delimits file sets; we treat any `f` line as a
//!   fresh-file reset for robustness across GNU/BSD builds.
//! - `a` — **file access mode** (`man lsof`, field `a`): `r` read, `w` write,
//!   `u` read *and* write; space / `-` unknown. A *writer* is `a` = `w` or `u`.
//! - `n` — **file name**. Compared against the canonicalized target
//!   ([`LsofProbe::observe`] canonicalizes before it asks; lsof resolves its
//!   own name — canonicalize-both-sides, mirroring the procfs backends).
//!
//! Within a file set `a` precedes `n`, so the access seen since the last file
//! boundary (`f`, `p`, or the previous `n`) is that file's mode. **Confidence:**
//! `-F` is lsof's stable inter-program contract, identical in shape on Linux and
//! macOS/BSD, so the grammar is documented, not guessed; the parser is also
//! tolerant of the `f` field being present or absent.
//!
//! # Failure semantics (§10)
//!
//! `lsof` absent, erroring, or emitting non-field output ⇒ [`Probe::Unknown`]
//! (renders the Y4 "?" badge) — never a false definite. `lsof` exits 1 for both
//! "no match" and real errors, so status alone cannot disambiguate: the shim
//! maps a spawn failure and a non-empty stderr to Unknown, while empty stdout is
//! the definite "no holder" ([`Probe::Free`]).
//!
//! The one error lsof reports that is **not** uncertainty is a target that does
//! not exist, and [`LsofProbe::observe`] settles it before spawning anything —
//! see its doc for why that and the canonical spelling are one question.

use super::probe::{LockProbe, Probe, WriterProbe};
use std::path::Path;

/// What an `lsof -F` scan saw about the queried target: whether *any* process
/// holds it open (the lock question) and whether any holder has it open for
/// *write* (the response.json writer question).
struct Sightings {
    any_holder: bool,
    any_writer: bool,
}

/// Parse `lsof -F pan` output, deciding [`Sightings`] for `target`. Returns
/// `None` when the bytes are not `-F` field output — invalid UTF-8, or
/// non-empty content with no process (`p`) set — which the caller maps to
/// [`Probe::Unknown`]. Empty output is a definite "no holder", not an error.
fn parse(output: &[u8], target: &Path) -> Option<Sightings> {
    let text = std::str::from_utf8(output).ok()?;
    let mut saw_process = false;
    let mut writer = false; // access mode of the file set in progress
    let mut seen = Sightings {
        any_holder: false,
        any_writer: false,
    };
    for line in text.lines() {
        let mut chars = line.chars();
        let Some(tag) = chars.next() else { continue };
        let value = chars.as_str();
        match tag {
            'p' => {
                saw_process = true;
                writer = false;
            }
            'f' => writer = false,
            'a' => writer = matches!(value.chars().next(), Some('w' | 'u')),
            'n' => {
                if Path::new(value) == target {
                    seen.any_holder = true;
                    seen.any_writer |= writer;
                }
                writer = false;
            }
            _ => {}
        }
    }
    // Non-empty output that is not a process set is lsof error text, not a
    // definite negative (§10): degrade to Unknown rather than read it as Free.
    if !saw_process && !text.trim().is_empty() {
        return None;
    }
    Some(seen)
}

/// Injected `lsof` runner (the trait-injection seam, DESIGN §10 "the
/// trait-injection pattern is the template for every new effect"): yields the
/// raw stdout of `lsof -F pan -- <target>`, or `None` when lsof could not be
/// observed at all (absent / errored). Keeping the spawn behind this seam is
/// what lets the parser and probe be 100 %-covered on Linux with a recorded
/// fixture and a fake runner; only the real macOS runner below is cfg'd out.
pub(super) trait LsofRunner {
    fn run(&self, target: &Path) -> Option<Vec<u8>>;
}

/// The macOS liveness backend: answers both probe questions by parsing an
/// injected [`LsofRunner`]'s output. One struct implements both traits because
/// a single `lsof` invocation (over the relevant target) evidences both.
pub(super) struct LsofProbe<R: LsofRunner> {
    runner: R,
}

impl<R: LsofRunner> LsofProbe<R> {
    pub(super) fn new(runner: R) -> Self {
        Self { runner }
    }

    /// Run lsof over `target` and reduce the scan to a tri-state via `held`
    /// (the per-question predicate over [`Sightings`]). A runner that could not
    /// observe, or output that is not `-F` field data, is [`Probe::Unknown`]
    /// (§10 — never a false definite).
    ///
    /// The target is resolved **before** it is asked about, which decides two
    /// things the probe was getting wrong on the platform it exists for
    /// (bl-1015, measured on a `macos-14` runner):
    ///
    /// - **Both sides canonical.** lsof prints the fully-resolved name of every
    ///   fd it finds, and on macOS every temp path resolves (`/var/folders/…` →
    ///   `/private/var/folders/…`, `/tmp` → `/private/tmp`). Asking about the
    ///   unresolved spelling made [`parse`]'s name comparison fail against the
    ///   very fd it had just been handed, so a held `response.json` read
    ///   [`Probe::Free`] and no model call ever showed as in flight there.
    /// - **An unresolvable target is [`Probe::Free`], definitely.** Nothing can
    ///   hold a path that is not there, which is exactly what the procfs
    ///   backends answer for one (no fd points at it). lsof instead *errors* on
    ///   an absent path, and an error is indistinguishable from lsof being
    ///   broken — so every agent with no inbox directory and every agent with
    ///   no step yet came back [`Probe::Unknown`], wearing the §10 "?" and
    ///   counting as live at the §3.6 delete gate. Asking the filesystem first
    ///   keeps the answer definite and costs one `stat`.
    fn observe(&self, target: &Path, held: impl FnOnce(&Sightings) -> bool) -> Probe {
        let Ok(target) = std::fs::canonicalize(target) else {
            return Probe::Free;
        };
        let out = self.runner.run(&target);
        match out.as_deref().and_then(|o| parse(o, &target)) {
            Some(seen) if held(&seen) => Probe::Held,
            Some(_) => Probe::Free,
            None => Probe::Unknown,
        }
    }
}

impl<R: LsofRunner> LockProbe for LsofProbe<R> {
    fn lock_state(&self, inbox_dir: &Path) -> Probe {
        self.observe(inbox_dir, |s| s.any_holder)
    }
}

impl<R: LsofRunner> WriterProbe for LsofProbe<R> {
    fn writer_state(&self, path: &Path) -> Probe {
        self.observe(path, |s| s.any_writer)
    }
}

/// The production `lsof` runner: spawns the real binary. macOS-only — the sole
/// code excluded from Linux coverage (§10 CI: "nothing but the lsof spawn shim
/// is cfg'd out").
#[cfg(target_os = "macos")]
pub(super) struct SystemLsof;

#[cfg(target_os = "macos")]
impl LsofRunner for SystemLsof {
    fn run(&self, target: &Path) -> Option<Vec<u8>> {
        // `-F pan`: p = PID, a = access mode, n = name (`man lsof`). `--`
        // guards a path beginning with `-`. lsof exits 1 for both "no match"
        // and error, so status is not the failure signal: a spawn failure or a
        // non-empty stderr is Unknown; empty stdout is the definite "no holder".
        let out = crate::git_env::output(
            crate::git_env::command(Path::new("lsof"))
                .args(["-F", "pan", "--"])
                .arg(target),
        )
        .ok()?;
        if !out.status.success() && !out.stderr.is_empty() {
            return None;
        }
        Some(out.stdout)
    }
}

/// Construct the macOS liveness probe: the real `lsof` runner behind a 2 s TTL
/// cache (DESIGN §10). `super::GitTree::from_repo` selects this by `cfg` on
/// macOS; Linux uses the `/proc` probes instead.
#[cfg(target_os = "macos")]
pub(super) fn system_probe()
-> super::probe_cache::TtlCache<LsofProbe<SystemLsof>, crate::ui_state::SystemClock> {
    super::probe_cache::TtlCache::new(LsofProbe::new(SystemLsof), crate::ui_state::SystemClock)
}

#[cfg(test)]
mod tests;
