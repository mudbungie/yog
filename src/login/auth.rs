//! Auth-shaped step-failure classification (DESIGN §8.3 detection, §15 M6 Z8): a
//! pure predicate over the already-derived step facts (§5.1 #10/#13 — framing
//! Failed + the response/error text) deciding whether a failed step *looks* like a
//! credential / authorization problem, so the Login affordance surfaces one click
//! away (beside the failed step and in the Login pane). Shell paints; this is
//! covered logic — the steps view-model reads [`classify`] into its Login flag.
//!
//! **The affordance names its row (bl-8e34).** An auth-shaped failure is only
//! half an answer: `bz --login` takes a provider row, and brazen's error line
//! carries none (`provider_detail` is null on the borrowed-credential decline
//! that motivated this). So the classification is a three-state [`AuthFailure`],
//! not a flag — no failure / a failure with no derivable row / a failure **on
//! this row** — and [`row_of_model`] is where the row comes from: the failing
//! step's own `request.json` model id, matched against the roles the agent's
//! governing config declares (§9.4's `roles:` grammar, read through
//! [`crate::fork::roles_at`]). Both halves are already-on-disk facts, so nothing
//! is stored and no new read is invented; the routing is a join, not a lookup
//! table.

use crate::git_tree::error_text;
use crate::model_pick::grammar::RoleModel;

/// Case-insensitive substrings that mark an **auth-shaped** failure — the
/// credential / auth / 401 / 403 / permission-denied class (§8.3). Matched against
/// the raw error event line ([`error_text`]), so both an HTTP status code and its
/// reason phrase catch. Deliberately narrow so a transport reset or a 500 never
/// fires this: bare `auth` is **excluded** (it hits `author`); the tokens are the
/// full `authenticat` / `authoriz` / `authoris` stems, and every other marker
/// names a genuinely auth-related concept.
const AUTH_MARKERS: &[&str] = &[
    "401",
    "403",
    "unauthorized", // the 401 reason phrase
    "unauthenticated",
    "forbidden", // the 403 reason phrase
    "permission denied",
    "permission-denied",
    "credential",  // credential / credentials
    "authenticat", // authentication / authenticate / not authenticated
    "authoriz",    // authorization / not authorized (US spelling)
    "authoris",    // authorisation (UK spelling)
    "api key",
    "api_key",
    "apikey",
];

/// Does `error_line` (a raw JSONL error event) look auth-shaped? Pure,
/// case-insensitive substring match against [`AUTH_MARKERS`].
pub fn looks_auth(error_line: &str) -> bool {
    let lower = error_line.to_ascii_lowercase();
    AUTH_MARKERS.iter().any(|marker| lower.contains(marker))
}

/// A step's auth-shaped-failure state and where its remedy points (§8.3).
/// Three states because three exist: the affordance is offered or it is not,
/// and when it is offered the row it should run is derivable or it is not.
/// Absent data is a value, never a branch — [`Unrouted`](Self::Unrouted) is the
/// honest middle, not an error.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AuthFailure {
    /// Not an auth-shaped failure. No affordance.
    #[default]
    No,
    /// Auth-shaped, but no provider row is derivable — the step's request names
    /// a model the governing config's roles do not bind, or binds under two
    /// rows at once. The affordance still paints; it routes to the Login pane,
    /// which is where a row is chosen by hand.
    Unrouted,
    /// Auth-shaped **on this provider row**: Login runs `bz --login --provider
    /// <row>` directly, with nothing left to pick.
    Row(String),
}

impl AuthFailure {
    /// Is the affordance offered at all? True for both failing states.
    pub fn offered(&self) -> bool {
        !matches!(self, Self::No)
    }

    /// The provider row to log in to, when one was derived.
    pub fn row(&self) -> Option<&str> {
        match self {
            Self::Row(row) => Some(row),
            _ => None,
        }
    }

    /// The conversation banner's sentence — the one authoritative home for this
    /// wording, beside the classification that decides it (the `frozen_label`
    /// discipline of `config_edit::branch`). A routed failure states the row,
    /// so the operator reads the remedy instead of inferring it; an unrouted one
    /// keeps the sentence that was there before, which is exactly as much as is
    /// known. `No` has no banner and the caller does not paint one; a value is
    /// still returned rather than an `Option` the caller must unwrap in a branch
    /// it has already taken.
    pub fn banner(&self) -> String {
        match self {
            Self::No => String::new(),
            Self::Unrouted => "⚠ the last step failed on credentials — log in below".to_string(),
            Self::Row(row) => {
                format!("⚠ the last step failed on {row}'s credentials — log in below")
            }
        }
    }

    /// The Steps-row mark beside an auth-failed step (§11): the same fact at
    /// list altitude, where only a few characters fit.
    pub fn step_mark(&self) -> String {
        match self {
            Self::No => String::new(),
            Self::Unrouted => "⚠ auth — Login ↙".to_string(),
            Self::Row(row) => format!("⚠ auth: {row} — Login ↙"),
        }
    }
}

/// Classify a step's settled `response.json` (§8.3): its framing is Failed (an
/// error event settled the last segment) **and** that error text matches
/// [`looks_auth`]. [`error_text`] returning `Some` already means framing Failed
/// (they share the last-segment traversal), so one call answers both facts —
/// the single source the steps view-model reads for its Login flag.
///
/// Bytes in, no row out: routing needs the governing config, which is a git
/// read this pure classifier does not do. The caller upgrades
/// [`Unrouted`](AuthFailure::Unrouted) through [`row_of_model`] — so the git
/// call is paid once per agent, and only when some step is actually failing.
pub fn classify(response_bytes: &[u8]) -> AuthFailure {
    match error_text(response_bytes) {
        Some(line) if looks_auth(&line) => AuthFailure::Unrouted,
        _ => AuthFailure::No,
    }
}

/// The provider row that serves `model`, per the roles a governing config
/// declares (`providers.yaml` `roles:`, read through [`crate::fork::roles_at`]).
///
/// The join is on the model id because that is the fact the failing step
/// records: `request.json` carries the model, never the row. A model no role
/// binds yields `None`, and so does one bound under **two different rows** —
/// naming either would be a guess, and a guessed row sends the operator through
/// a browser sign-in for a credential that was never the problem.
pub fn row_of_model(model: &str, roles: &[RoleModel]) -> Option<String> {
    let mut rows = roles
        .iter()
        .filter(|role| role.model == model)
        .map(|role| role.provider.as_str());
    let first = rows.next()?;
    rows.all(|row| row == first).then(|| first.to_string())
}

/// The **latest** step's auth-shaped-failure state (§11 center): the
/// conversation-view detection that banners Login inline. Reads off an
/// already-built steps view-model ([`crate::steps_view::StepsView`] — the one
/// owner of the per-step Login flag), so the banner, the Steps tab, and the
/// Login pane all derive from the same classification. It takes the view, not
/// the disk: the shell declares one standing `Query::Steps` that this banner
/// and the Steps tab share (REMOTE §9.7, bl-13f9), and a predicate that re-read
/// the whole steps tree per frame was the chat pane's frame-time cost.
pub fn latest_step_auth_failed(steps: &crate::steps_view::StepsView) -> AuthFailure {
    steps
        .steps
        .last()
        .map(|s| s.auth_failed.clone())
        .unwrap_or_default()
}
