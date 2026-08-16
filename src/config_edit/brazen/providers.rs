//! brazen's effective provider table, projected (DESIGN §5.1 #20/#21, §8.3).
//!
//! One row per provider `bz` would route to, read from `bz --list-providers
//! --json` — the **linked** brazen's own serde projection (§16.7 W10), not a
//! scan of the config text. yog re-implements none of it: the `name` column
//! names the row, the `auth` column names its credential model and the
//! `protocol` column names its wire dialect — all three spelled by brazen's own
//! serde renames.
//!
//! **`protocol` is the TOOL capability** (bl-3d22). The column carries brazen's
//! own `ProtocolId` spelling, and `ProtocolId` is a public, closed enum on
//! brazen's library surface — so [`ProviderRow::tools_blocked`] parses the
//! column back into that enum through brazen's own serde rename and answers
//! over a **total match**, which a new upstream dialect fails to compile until
//! its arm is added. That is the whole reason the judgement is here rather than
//! a table of row NAMES: `claude-code` is a name, `claude_code` is a protocol,
//! and only the second is a fact about request shape.
//!
//! It exists because row EXISTENCE never established request-shape
//! compatibility, which §9.4 used to claim it did: `/model worker claude-code
//! <id>` passed the row gate, advanced both config halves, and the next worker
//! start died at brazen's encoder before any network call — *"claude_code
//! carries no tool declarations; use the `anthropic` row for tools"*. Every yog
//! turn declares at least the `clients` tool
//! ([`crate::tool_host::Injection::tools`] returns it unconditionally, and
//! lernie splices the injection into every canonical request), so a
//! tool-less dialect can serve no lernie ROLE at all.
//!
//! **The upstream ask is already filed, and is brazen bl-5053** ("publish
//! per-row capability declines (tools, multi-turn) on the read surface"): brazen
//! projects no capability column — `--list-providers` serves `name`/`protocol`/
//! `auth`/`credential`, and the `Protocol` trait that owns the rejection is
//! crate-private, so the authoritative answer cannot be asked for. That ball was
//! filed off THIS defect seen once before, and its own words name the cost:
//! *"yog's model picker validated provider=claude-code for a tool-bearing worker
//! role (it only checks the row exists in `bz --list-providers`), and every
//! session call then died at encode"*. It was never gated on yog's side, so it
//! recurred. When brazen serves the column, the match below is deleted and the
//! column read in its place — that is the whole migration.
//!
//! **The refusal is per-ROLE, and removes nothing from the catalog.** The VISION
//! §4.9 alignment monitor's check is structurally tool-less
//! ([`crate::monitor::check`]: *"`bz` takes no tool flag, so tool-lessness here
//! is structural rather than a promise"*) and pins a MODEL rather than a provider
//! row, so a tool-less dialect is a legitimate target for it. What cannot be a
//! lernie role's row is not thereby useless.
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
//! is not a listed column at the `brazen = "=0.0.5"` pin), and it does not need
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
    /// (`anthropic_messages`, `claude_code`, …) — the tool-capability fact
    /// [`Self::tools_blocked`] reads.
    pub protocol: String,
    /// brazen's credential model for the row: `oauth2` / `api_key` / `bearer` /
    /// `none`.
    pub auth: String,
}

impl ProviderRow {
    /// Why this row can carry no lernie **role**, or `None` when it can
    /// (bl-3d22). Read by the §9.4 pick gate
    /// ([`plan`](crate::model_pick::plan)) and by the picker's provider control,
    /// which offers the row unselectably with this sentence beside it.
    ///
    /// Three answers, and the third is the load-bearing one. A dialect that
    /// declines tools is refused, naming its protocol. A dialect that carries
    /// them is fine. A spelling **this build's brazen does not know** is *no
    /// answer* rather than a refusal — the same discipline
    /// [`is_unknown_row`](crate::model_pick::grammar::is_unknown_row) applies to
    /// an unanswerable table: no surface may refuse on the strength of a
    /// question that went unanswered.
    pub fn tools_blocked(&self) -> Option<String> {
        match self.protocol_id()? {
            brazen::ProtocolId::ClaudeCode => Some(format!(
                "{} declares no tools — every yog turn carries at least the \
                 `clients` tool, so this row can serve no role",
                self.protocol
            )),
            brazen::ProtocolId::AnthropicMessages
            | brazen::ProtocolId::OpenAiChat
            | brazen::ProtocolId::OpenAiResponses
            | brazen::ProtocolId::GoogleGenAi
            | brazen::ProtocolId::OllamaChat => None,
        }
    }

    /// The `protocol` column parsed back into brazen's own registry key, through
    /// brazen's own serde rename — never a second table of spellings beside it.
    /// `None` for a spelling this build cannot name, which is a *newer* brazen
    /// or a degraded column, and in both cases an unanswered question.
    fn protocol_id(&self) -> Option<brazen::ProtocolId> {
        serde_json::from_value(serde_json::Value::String(self.protocol.clone())).ok()
    }

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
/// only asks "which rows exist" needs (§9.2's provider gate). Kept here so the
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

#[cfg(test)]
mod tests;
