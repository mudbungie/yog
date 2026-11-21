//! The login flow (DESIGN §8.3 as amended, §15 M6 Z8): bz's one interactive
//! surface, run as the **streamed-piped** spawn class (§8's third class).
//! `bz --login --provider <row>` streams its sign-in lines live to the
//! invoking surface — the Login pane, and beside an auth-failed step —
//! verbatim (§5.3 instance-local RAM); on exit ONE outcome row lands in
//! `ops.jsonl` per command line (§4.2, the stream never logged line-by-line),
//! and a non-zero exit carries the command that starts the sign-in again as a
//! run-by-hand fallback (§8.3). Credentials stay bz's: yog renders the flow,
//! never reads or writes a credential (§5.1 #22).
//!
//! **One command line, and the row's brazen picks the flow** (§8.3 rule 1 as
//! amended by bl-0127; REMOTE §8.3). yog fires the bare `bz --login --provider
//! <row>` for every row and never `--browser`: brazen's default is the sign-in
//! a human finishes somewhere else, which is the only kind a server can offer
//! a seat. A row declaring a device endpoint streams a verification URL and a
//! user code and the child stays up polling. A row declaring none prints its
//! **authorize URL on stdout and exits 0 with nothing stored yet** (brazen's
//! pasted-code sign-in) — and that is the one thing this module reads off a child
//! beyond its lines and its exit: a zero exit that wrote to stdout is a run
//! **waiting for a code** ([`LoginView::authorize`]), not a sign-in. The code
//! a human copies at that URL comes back as a second act, and
//! [`LoginRun::redeem`] fires the second command line into the same run, so
//! one lane tells the whole sign-in. There is no flow selector on any surface
//! and nothing yog guesses at: it never learns which flow a row serves until
//! the child says so.
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
//! executable under the `bz` namespace, so the flow is served by the same
//! linked brazen the config projection reads.

use std::path::Path;

use crate::cli_outbound::StreamedLine;

pub mod auth;
/// One sign-in's child processes and the view they fold into.
mod run;
/// The server's live-run holder (REMOTE §8.3, bl-c285): one sign-in per
/// workspace × provider, read by lanes and settled by a thread of its own.
pub mod runs;
/// The standing's one JSON spelling, both directions — beside its own type.
pub mod wire;

pub use run::{LoginRun, start};

#[cfg(test)]
mod tests;

/// bz's login subcommand flag, its provider selector (§8.2), and the flag a
/// pasted code rides in on (brazen's pasted-code sign-in).
const LOGIN_FLAG: &str = "--login";
const PROVIDER_FLAG: &str = "--provider";
const CODE_FLAG: &str = "--code";

/// The `bz` words one command line of a sign-in is fired with: the start, or
/// — with what the human copied — the finish. **One spelling, two readers**:
/// the spawn's argv and the `ops.jsonl` row both read this, so what is logged
/// can never diverge from what ran. The code rides as `--code=<pasted>`, one
/// word, so a paste that begins with a dash is still the flag's value.
fn login_args(provider: &str, code: Option<&str>) -> Vec<String> {
    let mut args = vec![
        LOGIN_FLAG.to_owned(),
        PROVIDER_FLAG.to_owned(),
        provider.to_owned(),
    ];
    args.extend(code.map(|pasted| format!("{CODE_FLAG}={pasted}")));
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
    /// **The URL to open, set exactly while the run waits for a code** (REMOTE
    /// §8.3, bl-0127): the row's brazen printed its authorize URL and exited
    /// clean with nothing stored. A human signs in there from any browser and
    /// what they copy comes back as `Action::Login`'s `code`; redeeming it
    /// clears this, and the finishing command's exit is the `outcome`. Never
    /// set beside an `outcome` — a run is waiting or it is settled.
    pub authorize: Option<String>,
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
/// fix rather than offering a command that would refuse. **It is always the
/// command that STARTS the sign-in**, a failed finish included: a code that
/// was refused is spent or foreign, and running it again by hand cannot help.
pub fn by_hand(workspace: Option<&Path>, provider: &str) -> String {
    let args = login_args(provider, None).join(" ");
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
