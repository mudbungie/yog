//! Where a **config-kind** failure is fixed (§9.1, bl-dd7f) — §8.3 rule 5's
//! sibling on the other kind of fault.
//!
//! An auth-shaped step failure has offered its remedy since bl-8e34: the
//! sentence names the row and Login is one click away
//! ([`crate::login::auth`]). A **config**-shaped one offered nothing. A
//! dispatch through a provider row brazen does not resolve dies with lernie's
//! own words —
//!
//! ```text
//! lernie prompt: provider error (Config): unknown provider `openai-chatgpt`
//! ```
//!
//! — and the §7.3 banner painted exactly that, with a Dismiss beside it and no
//! way out at all. Dismiss puts the sentence down; it does not fix the file.
//! The one thing that does is the §9.1 raw-TOML editor, which is where a
//! provider row is authored, and it was never named.
//!
//! **The judgement happens at the first dispatch, against a wall that exists.**
//! §9.2 once gated this at *birth* and the gate was retired (bl-00ee) for
//! judging a workspace's providers against a wall that did not exist yet. This
//! reads a failure that already happened, so there is nothing to pre-judge and
//! nothing to resurrect: the row's existence is brazen's fact, resolved at call
//! time (lernie ARCH §4.1), and this is that answer arriving.
//!
//! **It classifies, it never re-words.** brazen's and lernie's sentences stay
//! verbatim on the banner (INV-2 / §7.3); what is added is one sentence saying
//! which file holds the fault, and the control that opens it.

/// Case-insensitive markers of a **config**-shaped failure — the class whose
/// remedy is a file, not a credential and not a retry.
///
/// Deliberately narrow, the discipline [`looks_auth`](crate::login::auth::looks_auth)
/// keeps: each token is a phrase brazen or lernie writes for a configuration
/// fault and for nothing else. `provider error (config)` is lernie's own
/// wrapper around brazen's `ErrorKind::Config`; `unknown provider` is brazen's
/// `ConfigError::UnknownProvider`, the one an operator actually meets. A bare
/// `config` is **excluded** — it appears in every path yog prints.
const CONFIG_MARKERS: &[&str] = &["provider error (config)", "unknown provider"];

/// Does `text` look config-shaped? Pure, case-insensitive substring match
/// against [`CONFIG_MARKERS`] — the same shape, and the same reasons, as
/// [`looks_auth`](crate::login::auth::looks_auth).
pub fn looks_config(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    CONFIG_MARKERS.iter().any(|marker| lower.contains(marker))
}

/// The sentence painted beside a config-kind failure, or `None` when the
/// failure is not one. It names the row when the failing text names one, for
/// the reason bl-8e34 gave the auth affordance its row: a remedy that says
/// "edit the file" without saying *what to look for in it* is half an answer.
///
/// The row is **read out of the failure's own words**, never joined from the
/// tree: brazen quotes the name it could not resolve, so the fact is already
/// in the sentence being classified, and a second derivation could only
/// disagree with it.
pub fn config_remedy(text: &str) -> Option<String> {
    if !looks_config(text) {
        return None;
    }
    Some(match failing_row(text) {
        Some(row) => format!(
            "no provider row named {row} — add or rename it in brazen's config.toml, \
             then start the conversation again"
        ),
        None => "this is a configuration fault, not a credential one — the row a role \
                 names has to exist in brazen's config.toml"
            .to_owned(),
    })
}

/// The provider row a config-kind failure names, when it names one: the word
/// brazen quotes in backticks (`unknown provider `openai-chatgpt``), or the
/// word lernie quotes after `on provider row` when it wrapped the decline with
/// the row it routed to. `None` when neither shape is present, or when the
/// quote never closes — a half-parsed name is worse than none.
fn failing_row(text: &str) -> Option<String> {
    let quoted = |open: &str, close: char| -> Option<String> {
        let rest = text.split_once(open)?.1;
        let (row, _) = rest.split_once(close)?;
        (!row.is_empty()).then(|| row.to_owned())
    };
    quoted("unknown provider `", '`').or_else(|| quoted("on provider row \"", '"'))
}

#[cfg(test)]
mod tests;
