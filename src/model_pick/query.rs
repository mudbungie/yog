//! The live model roster (DESIGN §9.4, §5.1 #26): `bz --list-models --provider
//! <row> --json`, run as the **streamed-piped** spawn class (§8) — the same
//! machinery §8.3's login pane runs, not a second async-command surface.
//!
//! Fired on **every** picker open and never stored: the roster is the
//! provider's fact, it changes without yog's involvement, and a cached
//! candidate set would be a second representation of it (AGENTS.md — make it a
//! query, not a field). What the picker holds is the *answer to the question it
//! just asked*, for as long as the surface is open (§5.3).
//!
//! Failure renders as itself (INV-2 / §7.3): a spawn error, a non-zero exit, or
//! an empty roster all settle into [`RosterView::error`] with the exact command
//! to run by hand — never an empty list that reads as "your provider has no
//! models". The query is a *read* and appends no `ops.jsonl` row; the writes it
//! leads to do, through the surfaces that already log them (§9.2/§9.3).

use crate::cli_outbound::{
    Cli, Streamed, StreamedLine, StreamedOutcome, StreamedPoll, stderr_text, stdout_text,
};

const LIST_FLAG: &str = "--list-models";
const PROVIDER_FLAG: &str = "--provider";
const JSON_FLAG: &str = "--json";

/// The sentence an exit-0 run with an empty roster settles to. A provider that
/// answers "nothing" is a fact worth naming, not a picker with no rows.
pub const EMPTY_ROSTER: &str = "the provider offered no models";

/// The pure view-model the picker paints each frame.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RosterView {
    /// The query is still running — the surface paints the §11 pulse.
    pub in_flight: bool,
    /// The model ids the provider offered, in the order it listed them.
    pub models: Vec<String>,
    /// Why the roster is unusable, rendered in ichor (§7.3).
    pub error: Option<String>,
    /// The exact command to run by hand, set alongside any `error` (§8.3's
    /// fallback grammar). Single-sourced from the resolved argv, so the shown
    /// command and the attempted one can never diverge.
    pub fallback: Option<String>,
}

/// One `bz --list-models` run: the streamed child (`None` once settled, or from
/// the start when the spawn itself failed) plus the view the picker paints.
pub struct Roster {
    streamed: Option<Streamed>,
    view: RosterView,
    lines: Vec<StreamedLine>,
    provider: String,
    argv: Vec<String>,
}

/// Spawn the roster query for `provider`. **Infallible by construction**: a
/// spawn failure returns a `Roster` already settled into its error view, so no
/// caller can drop the failure on the floor (INV-2).
pub fn start(bz: &Cli, provider: &str) -> Roster {
    let args = [LIST_FLAG, PROVIDER_FLAG, provider, JSON_FLAG];
    let mut argv = vec![bz.binary().display().to_string()];
    argv.extend(args.iter().map(|s| (*s).to_string()));
    let mut roster = Roster {
        streamed: None,
        view: RosterView::default(),
        lines: Vec::new(),
        provider: provider.to_owned(),
        argv,
    };
    match bz.run(&args) {
        Ok(stream) => {
            roster.streamed = Some(Streamed::new(stream));
            roster.view.in_flight = true;
        }
        Err(spawn) => roster.fail(spawn.to_string()),
    }
    roster
}

impl Roster {
    /// The provider row this roster was asked for — the picker re-fires when
    /// the selected role's row differs from it.
    pub fn provider(&self) -> String {
        self.provider.clone()
    }

    /// The view-model this frame (owned per AGENTS.md rule 2).
    pub fn view(&self) -> RosterView {
        self.view.clone()
    }

    /// Non-blocking: drain what the child produced since the last frame and,
    /// on exit, settle the view. Returns whether the run is **still live**.
    /// Idempotent after settle — the guard short-circuits, so a stray poll
    /// never re-settles.
    pub fn poll(&mut self) -> bool {
        let Some(streamed) = self.streamed.as_mut() else {
            return false;
        };
        match streamed.poll() {
            StreamedPoll::Lines(lines) => {
                self.lines.extend(lines);
                true
            }
            StreamedPoll::Pending => true,
            StreamedPoll::Done(outcome) => {
                self.settle(outcome);
                false
            }
        }
    }

    /// Fold the terminal outcome into the view: exit 0 with ids ⇒ the roster;
    /// exit 0 with none ⇒ [`EMPTY_ROSTER`]; any other exit ⇒ the child's stderr
    /// lines (or the bare exit when it said nothing). The payload is parsed from
    /// the **stdout**-tagged lines alone, so a chatty stderr can never corrupt it.
    fn settle(&mut self, outcome: StreamedOutcome) {
        self.lines.extend(outcome.lines);
        self.streamed = None;
        self.view.in_flight = false;
        if outcome.exit != 0 {
            let stderr = stderr_text(&self.lines).trim().to_string();
            let why = if stderr.is_empty() {
                format!("bz {LIST_FLAG} exited {}", outcome.exit)
            } else {
                stderr
            };
            return self.fail(why);
        }
        let models = model_ids(&stdout_text(&self.lines));
        if models.is_empty() {
            return self.fail(EMPTY_ROSTER.to_string());
        }
        self.view.models = models;
    }

    /// Settle into the failure view: the reason plus the run-by-hand command.
    fn fail(&mut self, why: String) {
        self.streamed = None;
        self.view.in_flight = false;
        self.view.error = Some(why);
        self.view.fallback = Some(self.argv.join(" "));
    }

    /// Build a roster over an already-wired [`Streamed`] — the seam the unit
    /// tests drive `poll`/`settle` through deterministically.
    #[cfg(test)]
    pub(crate) fn from_streamed(streamed: Streamed, provider: &str) -> Self {
        Self {
            streamed: Some(streamed),
            view: RosterView {
                in_flight: true,
                ..RosterView::default()
            },
            lines: Vec::new(),
            provider: provider.to_owned(),
            argv: vec!["bz".to_string(), LIST_FLAG.to_string()],
        }
    }
}

/// The `models:` array of a `{"models":[{"id":…,"default":…},…]}` document —
/// `bz --list-models --json`'s payload, and the per-provider cache that verb
/// wholesale-writes from it, which is the same array under the same key. A
/// shapeless or unparseable document folds to no rows, never an error.
fn rows(document: &str) -> Vec<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(document.trim())
        .ok()
        .and_then(|listing| {
            listing
                .get("models")
                .and_then(serde_json::Value::as_array)
                .cloned()
        })
        .unwrap_or_default()
}

/// The `id` column of `bz --list-models --json`, in order. Every row carries
/// `id` and `default`; the three metadata keys beside them
/// (`context_window`, `max_output_tokens`, `display_name`) are OPTIONAL and
/// per-provider — brazen carries what the provider's list GET served and omits
/// the rest, so a payload with none of them is the ordinary case and not a
/// short one. The empty case is named by [`EMPTY_ROSTER`], never by an error.
pub fn model_ids(stdout: &str) -> Vec<String> {
    rows(stdout)
        .iter()
        .filter_map(|row| row.get("id").and_then(serde_json::Value::as_str))
        .map(str::to_owned)
        .collect()
}

/// The `context_window` brazen **served** for `model`, out of a roster document
/// — the number the picker seeds its `models.yaml` declaration from (§9.4,
/// bl-848f).
///
/// `None` where the provider published none (Anthropic, OpenAI and Ollama serve
/// no window on their list GET; Google does), where the document names no such
/// model, or where it is unreadable. Absent stays absent — brazen's own
/// zero-vs-unknown rule — because the fallback is a *declared default* the
/// entry's note says is one, and a fabricated number smuggled in here would be
/// indistinguishable from a served one.
pub fn served_window(document: &str, model: &str) -> Option<u32> {
    rows(document)
        .into_iter()
        .find(|row| row.get("id").and_then(serde_json::Value::as_str) == Some(model))?
        .get("context_window")?
        .as_u64()?
        .try_into()
        .ok()
}
