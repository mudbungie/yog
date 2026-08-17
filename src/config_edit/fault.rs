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
//!
//! **The class is wider than the wrapper says, and a marker cannot find all of
//! it** (bl-5252). A dispatch through a row whose *dialect* cannot carry a yog
//! turn dies at brazen's encoder, not at its config resolution, and the whole
//! `claude_code` decline family — no tools, no `tool_choice`, no multi-turn
//! transcript, no non-text block — is stamped `ErrorKind::ParseInput` by one
//! `reject` helper. So lernie wraps it `provider error (ParseInput) …`: no
//! config-kind word appears anywhere in it, and the banner offered Dismiss for a
//! failure whose only remedy is a config file. The second way in
//! ([`dialect_remedy`]) is therefore keyed on the dialect the decline NAMES, not
//! on a phrase and not on the error kind — the kind cannot tell that family from
//! a malformed image block, and the widest marker that could (`parseinput`)
//! would claim every one of them.

/// Case-insensitive markers of a **config**-shaped failure — the class whose
/// remedy is a file, not a credential and not a retry.
///
/// Deliberately narrow, the discipline [`looks_auth`](crate::login::auth::looks_auth)
/// keeps: each token is a phrase brazen or lernie writes for a configuration
/// fault and for nothing else — which is why they are not the whole class, and
/// [`dialect_remedy`] is the other way in. `provider error (config)` is lernie's own
/// wrapper around brazen's `ErrorKind::Config`; `unknown provider` is brazen's
/// `ConfigError::UnknownProvider`, the one an operator actually meets. A bare
/// `config` is **excluded** — it appears in every path yog prints.
const CONFIG_MARKERS: &[&str] = &["provider error (config)", "unknown provider"];

/// Does `text` look config-shaped? Either a [`CONFIG_MARKERS`] hit or a
/// **request-shape decline** ([`dialect_remedy`]) — two ways in, because brazen
/// spells the second one with no config-kind word in it at all.
pub fn looks_config(text: &str) -> bool {
    marker_hit(text) || dialect_remedy(text).is_some()
}

/// A [`CONFIG_MARKERS`] hit. Pure, case-insensitive substring match — the same
/// shape, and the same reasons, as
/// [`looks_auth`](crate::login::auth::looks_auth).
fn marker_hit(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    CONFIG_MARKERS.iter().any(|marker| lower.contains(marker))
}

/// The remedy for the **other** config-shaped failure: a step that died because
/// its row's *dialect* cannot carry a yog turn at all (bl-5252).
///
/// No marker can reach this family. brazen's four `claude_code` declines come
/// from one `reject` helper stamping `ErrorKind::ParseInput`, so lernie wraps
/// them `provider error (ParseInput) …` and the words above match nothing —
/// while the only remedy there IS a config file. The judgement is not
/// re-derived here either: `dialect_decline` reads the dialect out of the
/// failure's own words and answers with the sentence
/// [`tools_blocked`](crate::config_edit::brazen::ProviderRow::tools_blocked)
/// already gives the picker, so the banner and the picker cannot disagree about
/// why the row is unusable. What this adds is the operator's next move, and the
/// route is the same one an `unknown provider` gets: the §9.1 editor authors a
/// row, §9.4's picker chooses between them.
fn dialect_remedy(text: &str) -> Option<String> {
    let why = crate::config_edit::brazen::dialect_decline(text)?;
    Some(format!(
        "{why} — the fix is the row and not a retry: give this role a \
         tool-carrying row in the model picker, or author one in brazen's config.toml"
    ))
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
    if let Some(remedy) = dialect_remedy(text) {
        return Some(remedy);
    }
    marker_hit(text).then(|| match failing_row(text) {
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
