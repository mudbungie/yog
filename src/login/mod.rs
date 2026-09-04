//! The login flow (DESIGN §8.3 as amended, §15 M6 Z8): bz's one interactive
//! surface, run as the **streamed-piped** spawn class (§8's third class).
//! `bz --login --provider <row> --browser` streams its sign-in lines live to the
//! invoking surface — the Login pane, and beside an auth-failed step —
//! verbatim (§5.3 instance-local RAM); on exit ONE outcome row lands in
//! `ops.jsonl` (§4.2, the stream never logged line-by-line), and a non-zero exit
//! carries the exact command as a run-by-hand fallback (§8.3). Credentials stay
//! bz's: yog renders the flow, never reads or writes a credential (§5.1 #22).
//!
//! **The flow follows the row** (§8.3 rule 1 as amended by bl-61bf; bl-b4e5 for
//! the rule it replaces). bz's default is the headless device flow and it
//! refuses one — exit 78, "this provider has no device endpoint; use
//! `--browser`" — on any row whose `oauth` block declares no `device` endpoint.
//! So the flag follows brazen's own `device` column
//! ([`ProviderRow::headless_login`](crate::config_edit::brazen::ProviderRow::headless_login)):
//! a row that declares one is fired headless — bz binds nothing and streams the
//! verification URL and user code down the lane, so the sign-in completes from
//! any seat, a phone's browser included — and every other row is fired
//! `--browser`, the loopback AuthCode flow (RFC 8252) whose
//! `authorize_url`/`token_url` are *required* of every `oauth` block and which
//! is therefore the floor. That branch is [`Flow`], and it is read ONCE at the
//! spawn ([`runs::Runs::start`]): still no flow selector on any surface, and
//! nothing yog guesses at.
//!
//! The selectable providers come from
//! [`BzRunner::providers`](crate::config_edit::brazen::BzRunner::providers) —
//! brazen's effective provider table, read in-process since §16.7 W10, so the
//! built-in rows (§5.1 #21) are listed rather than hinted at. Whether a row can
//! be signed in at all is that table's own `auth` column
//! ([`ProviderRow::login_blocked`](crate::config_edit::brazen::ProviderRow::login_blocked)):
//! a keyless or api-keyed row gets **no button at all**, only the reason there
//! is none (§8.3 rule 4, bl-402f) — never a verb that can only exit 78. [`LoginRun`] wraps the streamed child + the
//! pure [`LoginView`] the shell paints; [`auth`] classifies an auth-shaped step
//! failure so the Login affordance surfaces one click away.
//!
//! The spawn itself stays a process (§16.7: seams that are processes for a
//! reason stay processes) — only the binary on the far side is now yog's own
//! executable under the `bz` namespace, so the device flow is served by the
//! same linked brazen the config projection reads.

use std::path::{Path, PathBuf};

use crate::cli_outbound::{
    Cli, Streamed, StreamedLine, StreamedOutcome, StreamedPoll, stderr_text,
};
use crate::opslog::{self, OpEntry, Origin};

pub mod auth;
/// The engine's live-run holder (REMOTE §8.3, bl-c285): one `bz --login` child
/// per workspace × provider, read by lanes and settled by a thread of its own.
pub mod runs;
/// The standing's one JSON spelling, both directions — beside its own type.
pub mod wire;

#[cfg(test)]
mod tests;

/// bz's login subcommand flag, its provider selector (§8.2), and the one flag
/// that selects a flow — see the module note.
const LOGIN_FLAG: &str = "--login";
const PROVIDER_FLAG: &str = "--provider";
const BROWSER_FLAG: &str = "--browser";

/// **Which sign-in flow a row is fired with** (§8.3 rule 1 as amended by
/// bl-61bf). Not a choice any surface offers: the row's own `device` column
/// decides it, and [`runs::Runs::start`] is the one place that reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    /// The loopback browser flow (RFC 8252), selected by `--browser`. The floor
    /// every oauth row can serve, and the flow whose redirect only completes
    /// where a browser can reach the engine's own loopback.
    Browser,
    /// bz's default headless device-code flow — no flag at all, because bz
    /// selects it by the row's `device` block and the style declared there.
    Device,
}

/// The `bz` words one sign-in is fired with. **One spelling, three readers**:
/// the spawn's argv, the `ops.jsonl` row and the run-by-hand fallback all read
/// this, so what is logged and what is offered can never diverge from what ran.
fn login_args(provider: &str, flow: Flow) -> Vec<&str> {
    let mut args = vec![LOGIN_FLAG, PROVIDER_FLAG, provider];
    if flow == Flow::Browser {
        args.push(BROWSER_FLAG);
    }
    args
}

/// The pure view-model the shell paints for a login run (§8.3). All three fields
/// are derived facts of the streamed child; the shell holds a [`LoginRun`] as its
/// §5.3 instance-local RAM and reads this each frame.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoginView {
    /// The lines bz printed, **verbatim** and in order (§8.3), rendered live as
    /// they arrive — **both** streams, each tagged
    /// ([`StreamedLine::err`]). bz writes its whole human-facing flow to stderr:
    /// the authorize URL, and on failure its exact reason and remedy. Carrying
    /// only stdout left the pane blank and the operator with nothing but a
    /// fallback command (bl-b4e5 defect 3).
    pub lines: Vec<StreamedLine>,
    /// The terminal exit code once the run settles (`None` while streaming) — the
    /// outcome the shell paints and the S0-T5 story asserts.
    pub outcome: Option<i32>,
    /// The command to run by hand, set **only** on a non-zero exit (§8.3:
    /// "Showing the exact command stays as the fallback when the piped flow
    /// exits non-zero"). The workspace-bound spelling ([`by_hand`]), because an
    /// unbound one refuses in an ordinary shell (bl-b589).
    pub fallback: Option<String>,
}

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
    state_root: PathBuf,
    ts: String,
}

/// The §8.3 **run-by-hand spelling** of one sign-in (bl-b589): the supported
/// workspace-bound command, built from the very consts the spawn uses and the
/// hatch's own subcommand words, so what the pane offers cannot drift from what
/// yog runs or from what the hatch accepts.
///
/// It is not the spawn's argv, and deliberately so: yog fires `bz` with the
/// wall already standing in the child's environment, which a human's shell has
/// no way to inherit. The command shown therefore has to *ask* for that wall by
/// name — which is exactly what `yog exec --ws <workspace>` is for. Outside any
/// workspace there is no lawful spelling at all, so the fallback says what to
/// fix rather than offering a command that would refuse.
pub fn by_hand(workspace: Option<&Path>, provider: &str, flow: Flow) -> String {
    let args = login_args(provider, flow).join(" ");
    let Some(ws) = workspace else {
        return format!(
            "no workspace: a sign-in belongs to one, so there is nothing to run by hand — \
             focus a workspace, then yog {} {} <workspace> bz {args}",
            crate::world::hatch::EXEC_SUBCMD,
            crate::world::hatch::WS_FLAG,
        );
    };
    format!(
        "yog {} {} {} bz {args}",
        crate::world::hatch::EXEC_SUBCMD,
        crate::world::hatch::WS_FLAG,
        ws.display(),
    )
}

/// Spawn `bz --login --provider <provider>` in `flow` as the streamed-piped
/// class (§8): stdin null (this verb never reads TTY input — the browser flow
/// completes through the loopback redirect and the device flow through a code
/// the human types elsewhere), stdout/stderr piped for live line-buffering.
/// A spawn failure (bz absent) appends a synthetic `ops.jsonl` line (§4.2) and
/// returns the error, so no attempt is ever un-logged (§7.3). `ts` is the
/// wall-clock stamp minted at the shell boundary, kept clock-free here.
pub fn start(
    bz: &Cli,
    provider: &str,
    state_root: &Path,
    ts: &str,
    workspace: Option<&Path>,
    flow: Flow,
) -> std::io::Result<LoginRun> {
    let args = login_args(provider, flow);
    let mut argv = vec![bz.binary().display().to_string()];
    argv.extend(args.iter().map(|s| (*s).to_string()));
    match bz.run(&args) {
        Ok(stream) => Ok(LoginRun {
            streamed: Streamed::new(stream),
            view: LoginView::default(),
            argv,
            by_hand: by_hand(workspace, provider, flow),
            state_root: state_root.to_path_buf(),
            ts: ts.to_owned(),
        }),
        Err(spawn) => {
            let entry = OpEntry::synthetic_failure(
                ts.to_owned(),
                argv,
                String::new(),
                spawn.to_string(),
                Origin::World,
            );
            opslog::append(state_root, &entry)?;
            Err(std::io::Error::other(spawn))
        }
    }
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
        if self.view.outcome.is_some() {
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

    /// Fold the terminal outcome into the view and append the single `ops.jsonl`
    /// row (§4.2): exit + the run's stderr, `stdout` blank (the stream converged
    /// live, never re-logged line-by-line). The logged stderr is derived from the
    /// very lines the pane painted, so the log and the surface can never name
    /// different text. A non-zero exit sets the §8.3 fallback — the exact argv
    /// that was attempted, in the flow it was attempted in, so the command
    /// offered to run by hand is one that would actually succeed. An append io error has nowhere
    /// else to be recorded, so it is dropped (mirrors `Stream`'s best-effort
    /// cleanup) — the live view already showed the run.
    fn finalize(&mut self, outcome: StreamedOutcome) {
        self.view.lines.extend(outcome.lines);
        self.view.outcome = Some(outcome.exit);
        if outcome.exit != 0 {
            self.view.fallback = Some(self.by_hand.clone());
        }
        let entry = OpEntry {
            ts: self.ts.clone(),
            argv: self.argv.clone(),
            cwd: String::new(),
            exit: outcome.exit,
            stdout: String::new(),
            stderr: stderr_text(&self.view.lines),
            // The §8.3 login pane paints its own lines and its own
            // run-by-hand fallback (§7.3, bl-48f8) — the composer has no
            // business breaking that news.
            origin: Origin::World,
        };
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
            by_hand: by_hand(Some(Path::new("/ws")), "openai", Flow::Browser),
            argv,
            state_root: state_root.to_path_buf(),
            ts: "TS".to_owned(),
        }
    }
}
