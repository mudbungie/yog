//! **The first thing to say about a wall with no credential is sign in**
//! (DESIGN §8.1, bl-1fd0) — the rung, at the one door every fire passes
//! (bl-2291).
//!
//! The ruling was that a goal typed into an unsigned wall works zero percent of
//! the time: the conversation is born, dies on no-models, and the operator
//! learns it from a dead row. bl-7cc8 deleted the callerless fold that judged
//! it and left the predicate in the doc, honourable by any seat that asked
//! `/providers` first — and no seat did, because a seat that re-judged the rows
//! would be a second implementation of a settled decision, and there are two
//! seats. So the decision has one home now, and it is the `Prompt` door
//! ([`super::doors::prompt`]): a click, a line and a deposit all pass it, the
//! refusal is the envelope every act already answers with, and a refused fire
//! spends nothing — the caller still holds its goal.
//!
//! **The predicate is the wall's `credential` column, plus one clause the
//! door's placement forces.** A wall is ready when any row is
//! [`credentialed`](ProviderRow::credentialed) — every spelling but `missing`
//! and `not required`, brazen's own answer to whether the row could answer at
//! all. A keyless row does not ready the wall by itself: brazen merges its
//! built-in table under every config, so `ollama` and `claude-code` read `not
//! required` on every wall there can be and a predicate they satisfied would
//! be one nothing ever fails. **Unless the lineage the fire forks off names it
//! in a role.** A role pointing at a keyless row is the operator's own hand,
//! not the merge — and a door has no read-past where a seat's sentence had one,
//! so the conservatism §8.1 tolerated (*told to sign in when their setup needs
//! no sign-in*) would have become a keyless-only setup that can never start a
//! conversation. The clause is one read of the file §9.4's pick gate already
//! reads at its own fire.
//!
//! **An unanswerable table refuses nothing.** brazen unable to answer is an
//! empty table, never an error (the `Providers` read's own contract), and no
//! surface refuses on the strength of a question that went unanswered.

use std::path::Path;

use crate::config_edit::branch::config_file;
use crate::config_edit::brazen::{BzRunner, NOT_REQUIRED, ProviderRow, RealBzRunner};
use crate::model_pick::{BRANCH, PROVIDERS};

use super::super::config::wall_env;
use super::Deps;

/// The rung: `Ok` when the fire would reach a model, the refusal otherwise.
/// `lineage` is the `Prepared`'s §8.7 birth lineage — `None` is
/// `config/default`, exactly as the fire itself reads it.
pub(super) fn gate(deps: &Deps, workspace: &Path, lineage: Option<&str>) -> Result<(), String> {
    let rows = RealBzRunner::resolve(&wall_env(deps, workspace)).providers();
    if rows.is_empty() || ready(&rows, &named_rows(workspace, lineage)) {
        return Ok(());
    }
    Err(refusal(&rows))
}

/// The provider names the lineage's role assignments point at (§9.4's
/// `roles.<r>.provider`). A lineage that cannot be read names nothing, which
/// is the `Roles` read's own answer rather than a refusal: the wall predicate
/// alone then decides.
fn named_rows(workspace: &Path, lineage: Option<&str>) -> Vec<String> {
    let refspec = format!("config/{}", lineage.unwrap_or(BRANCH));
    let text = config_file(workspace, &refspec, PROVIDERS)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    crate::model_pick::grammar::roles(&text)
        .into_iter()
        .map(|role| role.provider)
        .collect()
}

/// The fold itself, pure over the table and the names the config points at.
pub(crate) fn ready(rows: &[ProviderRow], named: &[String]) -> bool {
    rows.iter().any(|row| {
        row.credentialed() || (row.credential == NOT_REQUIRED && named.contains(&row.name))
    })
}

/// The one sentence, fact first and remedy last: what is wrong with the wall,
/// what the fire would have cost, and the act that comes first.
fn refusal(rows: &[ProviderRow]) -> String {
    let names: Vec<&str> = rows.iter().map(|row| row.name.as_str()).collect();
    format!(
        "sign in first: no provider in this workspace's wall holds a credential, so a \
         conversation begun here would reach no model — /login <provider> (rows: {})",
        names.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, credential: &str) -> ProviderRow {
        ProviderRow {
            name: name.to_owned(),
            protocol: String::new(),
            auth: String::new(),
            credential: credential.to_owned(),
            effort: false,
            priority: false,
            device: String::new(),
        }
    }

    /// The three clauses of the fold, each on its own row: a credential of any
    /// spelling readies the wall, a keyless row does not, and a keyless row a
    /// role names does.
    #[test]
    fn a_credential_readies_a_keyless_row_only_when_a_role_names_it() {
        let keyless = [row("ollama", NOT_REQUIRED), row("acme", "missing")];
        assert!(!ready(&keyless, &[]));
        assert!(ready(&keyless, &["ollama".to_owned()]));
        assert!(
            !ready(&keyless, &["acme".to_owned()]),
            "missing is the one refusal"
        );
        assert!(ready(&[row("acme", "stored")], &[]));
        assert!(ready(
            &[row("acme", "a-spelling-this-build-never-heard-of")],
            &[]
        ));
    }

    #[test]
    fn the_refusal_names_the_act_and_the_rows() {
        let said = refusal(&[row("ollama", NOT_REQUIRED), row("acme", "missing")]);
        assert!(said.starts_with("sign in first"), "{said}");
        assert!(said.contains("/login"), "{said}");
        assert!(said.ends_with("(rows: ollama, acme)"), "{said}");
    }
}
