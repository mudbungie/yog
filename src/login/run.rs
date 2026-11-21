//! **One sign-in's command lines, and the view they fold into** (REMOTE §8.3;
//! split from the module at §12's cap on the seam between *what a sign-in is
//! spelled as* — the words, the view, the run-by-hand command, which stay
//! there — and *the children that run it*). A sign-in is one command line for
//! a row with a device endpoint and two for a row without one: the first
//! prints the authorize URL and exits waiting, [`LoginRun::redeem`] fires the
//! second with the code a human copied, and both fold into the one
//! [`LoginView`] a lane reads.

use std::path::{Path, PathBuf};

use super::{LoginView, by_hand, login_args};
use crate::cli_outbound::{Cli, Streamed, StreamedOutcome, StreamedPoll, stderr_text};
use crate::opslog::{self, OpEntry, Origin};

/// A live `bz --login` run: the streamed child (§8) plus the [`LoginView`] the
/// shell paints. Held at the invoking surface as instance-local RAM (§5.3); the
/// child is SIGTERM'd on drop (closing the surface aborts the device flow —
/// consistent with a device code being for the human at *this* keyboard).
pub struct LoginRun {
    streamed: Streamed,
    view: LoginView,
    /// The resolved argv — the `ops.jsonl` outcome row and the run-by-hand
    /// fallback both read it (single source: the logged and the shown command
    /// never diverge).
    argv: Vec<String>,
    /// The §8.3 run-by-hand spelling ([`by_hand`]), minted at spawn while the
    /// workspace is in hand.
    by_hand: String,
    /// The row this run signs in — what [`redeem`](Self::redeem) names again.
    provider: String,
    /// Where the current command line's own output begins in `view.lines`:
    /// its stdout is read for the authorize URL and its stderr is its
    /// `ops.jsonl` row, and neither may reach back into the command before.
    from: usize,
    state_root: PathBuf,
    ts: String,
}

/// Fire one command line of `provider`'s sign-in as the streamed-piped class
/// (§8): stdin null (nothing here reads TTY input — a device flow completes
/// through a code the human types elsewhere, a pasted-code one through a
/// second command line), stdout/stderr piped for live line-buffering. A spawn
/// failure (bz absent) appends a synthetic `ops.jsonl` line (§4.2) and returns
/// the error, so no attempt is ever un-logged (§7.3).
fn fire(
    bz: &Cli,
    provider: &str,
    code: Option<&str>,
    state_root: &Path,
    ts: &str,
) -> std::io::Result<(Streamed, Vec<String>)> {
    let args = login_args(provider, code);
    let mut argv = vec![bz.binary().display().to_string()];
    argv.extend(args.iter().cloned());
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    match bz.run(&words) {
        Ok(stream) => Ok((Streamed::new(stream), argv)),
        Err(spawn) => {
            let entry = OpEntry::synthetic_failure(
                ts.to_owned(),
                argv,
                String::new(),
                spawn.to_string(),
                Origin::World,
                bz.client(),
            );
            opslog::append(state_root, &entry)?;
            Err(std::io::Error::other(spawn))
        }
    }
}

/// Start `provider`'s sign-in: `bz --login --provider <provider>`, in whichever
/// flow that row's own brazen serves (module doc). `ts` is the wall-clock
/// stamp minted at the shell boundary, kept clock-free here.
pub fn start(
    bz: &Cli,
    provider: &str,
    state_root: &Path,
    ts: &str,
    workspace: Option<&Path>,
) -> std::io::Result<LoginRun> {
    let (streamed, argv) = fire(bz, provider, None, state_root, ts)?;
    Ok(LoginRun {
        streamed,
        view: LoginView::default(),
        argv,
        by_hand: by_hand(workspace, provider),
        provider: provider.to_owned(),
        from: 0,
        state_root: state_root.to_path_buf(),
        ts: ts.to_owned(),
    })
}

impl LoginRun {
    /// The view-model the shell paints this frame (§8.3). Owned per rule 2; the
    /// device flow is a handful of lines, so the clone is negligible.
    pub fn view(&self) -> LoginView {
        self.view.clone()
    }

    /// Non-blocking: drain what the child has produced since the last frame into
    /// the view, and on exit finalize (outcome + fallback + the one ops row).
    /// Returns whether the run is **still live** — `false` once settled, so the
    /// shell can drop it. Idempotent after settle (the guard short-circuits, and
    /// [`Streamed::poll`] itself stays `Pending`): never a second outcome row.
    pub fn poll(&mut self) -> bool {
        if self.view.outcome.is_some() || self.view.authorize.is_some() {
            return false;
        }
        match self.streamed.poll() {
            StreamedPoll::Lines(lines) => {
                self.view.lines.extend(lines);
                true
            }
            StreamedPoll::Pending => true,
            StreamedPoll::Done(outcome) => {
                self.finalize(outcome);
                false
            }
        }
    }

    /// **Hand a waiting run the code a human copied** (REMOTE §8.3, bl-0127):
    /// fire `bz --login --provider <row> --code=<pasted>` as this run's second
    /// command line. Its lines append to the ones already said and its exit is
    /// the run's outcome, so the lane a seat holds tells one sign-in from the
    /// URL to the result. The code is checked by the brazen that printed the
    /// URL, against a verifier that never left its box — nothing here judges
    /// it. A spawn failure leaves the run waiting exactly as it stood.
    pub fn redeem(&mut self, bz: &Cli, code: &str, ts: &str) -> std::io::Result<()> {
        let (streamed, argv) = fire(bz, &self.provider, Some(code), &self.state_root, ts)?;
        self.streamed = streamed;
        self.argv = argv;
        ts.clone_into(&mut self.ts);
        self.from = self.view.lines.len();
        self.view.authorize = None;
        Ok(())
    }

    /// Fold one command line's exit into the view and append its single
    /// `ops.jsonl` row (§4.2): exit + that command's stderr, `stdout` blank
    /// (the stream converged live, never re-logged line-by-line). The logged
    /// stderr is derived from the very lines the pane painted, so the log and
    /// the surface can never name different text.
    ///
    /// **A zero exit that wrote to stdout is not a sign-in** (module doc): the
    /// line is the authorize URL, the run stands waiting for a code, and no
    /// outcome is stated. Every other exit settles the run, and a non-zero one
    /// sets the §8.3 fallback. An append io error has nowhere else to be
    /// recorded, so it is dropped (mirrors `Stream`'s best-effort cleanup) —
    /// the live view already showed the run.
    fn finalize(&mut self, outcome: StreamedOutcome) {
        self.view.lines.extend(outcome.lines);
        let said = self.view.lines.get(self.from..).unwrap_or_default();
        let url = said.iter().rev().find(|line| !line.err);
        let url = url.map(|line| line.text.clone());
        let entry = OpEntry {
            ts: self.ts.clone(),
            argv: self.argv.clone(),
            cwd: String::new(),
            exit: outcome.exit,
            stdout: String::new(),
            stderr: stderr_text(said),
            // The §8.3 login pane paints its own lines and its own
            // run-by-hand fallback (§7.3, bl-48f8) — the composer has no
            // business breaking that news.
            origin: Origin::World,
            client: crate::registry::Client::default(),
        };
        match (outcome.exit, url) {
            (0, Some(url)) => self.view.authorize = Some(url),
            (exit, _) => {
                self.view.outcome = Some(exit);
                self.view.fallback = (exit != 0).then(|| self.by_hand.clone());
            }
        }
        let _ = opslog::append(&self.state_root, &entry);
    }

    /// Build a run over an already-wired [`Streamed`] — the seam the unit tests
    /// drive [`poll`](Self::poll)/[`finalize`](Self::finalize) through
    /// deterministically (the real spawn path is [`start`], covered by S0-T5).
    #[cfg(test)]
    pub(crate) fn from_streamed(streamed: Streamed, argv: Vec<String>, state_root: &Path) -> Self {
        Self {
            streamed,
            view: LoginView::default(),
            by_hand: by_hand(Some(Path::new("/ws")), "openai"),
            provider: "openai".to_owned(),
            from: 0,
            argv,
            state_root: state_root.to_path_buf(),
            ts: "TS".to_owned(),
        }
    }
}
