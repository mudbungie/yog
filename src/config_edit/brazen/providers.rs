//! brazen's effective provider table, projected (DESIGN §5.1 #20/#21, §8.3).
//!
//! One row per provider `bz` would route to, read from `bz --list-providers
//! --json` — the **linked** brazen's own serde projection (§16.7 W10), not a
//! scan of the config text. yog re-implements none of it: the `name` column
//! names the row, the `auth` column names its credential model and the
//! `protocol` column names its wire dialect — all three spelled by brazen's own
//! serde renames.
//!
//! **What the `protocol` column judges lives beside it** in
//! [`capability`]: the tool-capability refusal (bl-3d22), the context caveat
//! (bl-671d) and the dead step's route back to the first of them (bl-5252) — all
//! keyed on brazen's own public `ProtocolId` over one total match. This module
//! owns the columns; that one owns what a dialect does to a yog turn.
//!
//! **`auth` is the login capability.** brazen's `Provider::oauth` is documented
//! `present exactly when auth = "oauth2"` — resolution pairs the two or fails
//! (→78), so `auth == "oauth2"` *is* "this row has an `oauth` block", answered by
//! the crate that owns the invariant. `bz --login` serves oauth rows only, so
//! every other spelling is a row it can only refuse. That is the whole of the
//! capability question the Login surface asks (§8.3), which is why nothing here
//! reclassifies a row: it reads a column.
//!
//! **The rendered row is derived here too** ([`ProviderRowView`], bl-402f), not
//! in the pane: `auth` plus the §5.1 #22 presence read is the whole of what a
//! surface can say about a provider, so the words that say it live beside the
//! columns they read. The §8.3 Login pane is the one **painted** seat at it
//! (bl-20cb retired the §9.5 config copy); the boundary's `Providers` reply and
//! §9.4's remedy sentence read the same derivation without repainting the row.
//!
//! The projection carries no *device-endpoint* fact (`OAuthConfig::device_url`
//! is not a listed column at the brazen pin — `Cargo.toml` is the pin
//! authority and no version is restated here), and it does not need
//! to: `authorize_url`/`token_url` are required fields of every `oauth` block
//! while `device_url` is `Option`, so the **browser** flow is the one flow every
//! oauth row can serve — see [`crate::login`] for the flow rule.

/// brazen's `auth` spelling for an OAuth row (`AuthId::OAuth2`'s serde rename).
const OAUTH2: &str = "oauth2";

/// The keyless spelling: the row needs no credential at all (a local runtime,
/// or a host CLI that carries its own).
const NONE: &str = "none";

/// The two keyed spellings: the secret is a *config* value, not a bz-stored
/// credential, so the operator's move is an edit and never a sign-in.
const API_KEY: &str = "api_key";
const BEARER: &str = "bearer";

/// One row of the effective provider table (§5.1 #20/#21) — the three columns
/// yog consumes, verbatim from brazen's `--list-providers --json` object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRow {
    /// The row's name — the `--provider <name>` selector and the credential
    /// file's stem.
    pub name: String,
    /// The wire dialect the row speaks, in brazen's own `ProtocolId` spelling
    /// (`anthropic_messages`, `claude_code`, …) — the column
    /// [`capability`]'s two reads judge.
    pub protocol: String,
    /// brazen's credential model for the row: `oauth2` / `api_key` / `bearer` /
    /// `none`.
    pub auth: String,
}

impl ProviderRow {
    /// Why `bz --login` cannot serve this row, or `None` when it can (§8.3).
    /// The Login surface renders the button exactly when this is `None`, and
    /// **this sentence instead of a button** otherwise (bl-402f): a row that
    /// could only exit 78 gets no verb at all, and the sentence is the
    /// operator's actual next move, per credential model. An `auth` spelling
    /// this build does not know is quoted rather than guessed at.
    pub fn login_blocked(&self) -> Option<String> {
        match self.auth.as_str() {
            OAUTH2 => None,
            NONE => Some("keyless — nothing to log in".to_owned()),
            API_KEY => Some("api-key provider — set the key in Config".to_owned()),
            BEARER => Some("bearer-token provider — set the token in Config".to_owned()),
            other => Some(format!(
                "auth \"{other}\" — bz --login signs in oauth2 rows only"
            )),
        }
    }

    /// The credential fact in the words this row's credential model makes true
    /// (bl-402f, STORIES S5 point 4 "presence renders"): a login-capable row is
    /// signed in or not, a keyless row needs nothing, and a keyed row's file is
    /// stored or absent — "signed in" is a sentence only a login can earn.
    /// `present` is the §5.1 #22 existence read, the only credential fact yog
    /// ever holds.
    fn credential_words(&self, present: bool) -> &'static str {
        match (self.auth.as_str(), present) {
            (OAUTH2, true) => "signed in",
            (OAUTH2, false) => "not signed in",
            (NONE, _) => "no credential needed",
            (_, true) => "credential stored",
            (_, false) => "no credential stored",
        }
    }

    /// This row as every surface renders it (§8.3, §9.5).
    fn view(&self, present: bool) -> ProviderRowView {
        ProviderRowView {
            name: self.name.clone(),
            fact: format!("auth {} · {}", self.auth, self.credential_words(present)),
            blocked: self.login_blocked(),
        }
    }
}

/// One provider row as the operator reads it: what it is called, its credential
/// fact in words, and either the Login verb (`blocked == None`) or the reason
/// there is none. **One derivation, one painted seat** — the §8.3 Login pane
/// renders this struct and, since bl-20cb, nothing else does: a second surface
/// painting the same row was two renderings of one fact, and the seat that keeps
/// it is the one whose verb the row is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRowView {
    pub name: String,
    pub fact: String,
    pub blocked: Option<String>,
}

/// Pair the effective table with the credential-presence answer (§5.1 #22) into
/// the rendered rows, in listing order. The **table** names the rows; a row the
/// presence list does not answer about is credential-less, not a hole (absence
/// is the answer, so no branch and no error case).
pub fn row_views(rows: &[ProviderRow], creds: &[(String, bool)]) -> Vec<ProviderRowView> {
    rows.iter()
        .map(|row| {
            let present = creds.iter().any(|(name, yes)| *name == row.name && *yes);
            row.view(present)
        })
        .collect()
}

/// Parse `bz --list-providers --json`
/// (`{"providers":[{"name":…,"protocol":…,"auth":…},…]}`) into rows, in listing
/// order — which IS brazen's routing order. A non-object or shapeless payload
/// folds to no rows, never an error; a row missing any column folds to an empty
/// string for it (an unnamed row is unroutable, an unknown auth model is not
/// `oauth2`, and an unreadable protocol answers nothing about tools, so all
/// three degrade without a branch).
pub fn provider_rows(listing_json: &str) -> Vec<ProviderRow> {
    let Ok(listing) = serde_json::from_str::<serde_json::Value>(listing_json) else {
        return Vec::new();
    };
    let Some(rows) = listing
        .get("providers")
        .and_then(serde_json::Value::as_array)
    else {
        return Vec::new();
    };
    rows.iter()
        .map(|row| ProviderRow {
            name: column(row, "name"),
            protocol: column(row, "protocol"),
            auth: column(row, "auth"),
        })
        .collect()
}

/// The `name` column alone, in listing order — the whole answer a caller that
/// only asks "which rows exist" needs (§9.4's pick gate, §9.5's provider
/// control). Kept here so the
/// projection has one home: a caller that mapped the rows itself would be a
/// second place that knows which column names a row.
pub fn row_names(rows: &[ProviderRow]) -> Vec<String> {
    rows.iter().map(|r| r.name.clone()).collect()
}

/// One string column of a listing row, or `""` when absent or not a string.
fn column(row: &serde_json::Value, key: &str) -> String {
    row.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

mod capability;

pub use capability::{CONTEXT_REMEDY, dialect_decline};

#[cfg(test)]
mod tests;
