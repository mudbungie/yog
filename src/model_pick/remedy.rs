//! Where the operator goes when the §9.4 roster query comes back on
//! credentials — §8.3 rule 5's sibling seat (bl-91f1).
//!
//! The picker asks a provider row for its models; when that row has no
//! credential, `bz` answers in its own words, which yog forwards verbatim
//! (INV-2 / §7.3): *"no credential for this provider: set BRAZEN_API_KEY (or
//! the provider API-key env var / --api-key) or run `bz --login --provider
//! <id>`"*. That sentence is brazen's `resolved_secret` decline, reached from
//! `StaticSecretAuth` — so it fires for `api_key` and `bearer` rows and no
//! others. Left as the whole answer it is a shell remedy in a desktop surface,
//! and for exactly the rows it fires on it leads with an env var and offers a
//! sign-in yog's own capability read says can only exit 78.
//!
//! **yog already knows the row's real remedy, and says it in two other seats.**
//! [`ProviderRow::login_blocked`] is the one home for "what this row needs",
//! rendered by the §8.3 Login rows and the §9.5 config rows alike. This is a
//! third seat at that same derivation and it invents no wording of its own: it
//! pairs that sentence with the §11 tab that acts on it, so the picker's fault
//! ends in a control like every other auth seat in the tree.
//!
//! **No `Unrouted` state.** §8.3's [`AuthFailure`](crate::login::auth::AuthFailure)
//! needs one because a failed step's error carries no row and the join that
//! recovers it can come back ambiguous. The picker has nothing to join: the row
//! is the question it just asked, by name, one frame ago.

use crate::config_edit::brazen::ProviderRow;
use crate::keymap::CenterTab;
use crate::login::auth::looks_auth;

/// The way out of an auth-shaped roster failure: what the row needs, the
/// control that goes there, and the §11 tab it focuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remedy {
    /// Why this row cannot answer, in the words the Login and §9.5 config rows
    /// already use, under its own name. `None` for an oauth2 row, where
    /// [`ProviderRow::login_blocked`] deliberately has no sentence because the
    /// verb is the entire answer.
    pub reason: Option<String>,
    /// The control's label.
    pub verb: String,
    /// The tab the control focuses.
    pub tab: CenterTab,
}

/// The remedy for `error`, or `None` when it is not a credential problem at
/// all — a spawn failure, a non-zero exit on something else, an empty roster.
/// Gated on [`looks_auth`], §8.3's own classifier, rather than a second list of
/// markers: routing a transport reset to the Config tab would be a guess with a
/// button on it.
pub fn remedy(row: &ProviderRow, error: &str) -> Option<Remedy> {
    if !looks_auth(error) {
        return None;
    }
    Some(match row.login_blocked() {
        // The row signs in. Here the verb can name its object outright — the
        // routing bl-8e34 had to derive from a git join, this seat was handed.
        None => Remedy {
            reason: None,
            verb: format!("Login: {}", row.name),
            tab: CenterTab::Login,
        },
        // Every other credential model — `api_key`, `bearer`, keyless, and an
        // `auth` spelling this build does not know — is authored in brazen's
        // own `config.toml`, which is the §9.1 editor in the Config tab. One
        // arm and no per-spelling branch: the *sentence* already differs per
        // row because `login_blocked` differs, and the destination does not.
        Some(why) => Remedy {
            reason: Some(format!("{}: {why}", row.name)),
            verb: CenterTab::Config.label().to_owned(),
            tab: CenterTab::Config,
        },
    })
}
