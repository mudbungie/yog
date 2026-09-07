//! The door's sentences, one per [`Unready`] (DESIGN §8.1, bl-58e7/bl-21e9) —
//! split off [`super`] on the seam that module's own doc draws: it decides, and
//! this words the decision.
//!
//! **The three do not share a remedy, and that is the whole point.** A row that
//! is here and holds no credential is an act on that row; a row the wall does
//! not declare at all is an act on the *table*, and `/login` for it is a loop
//! with no exit (bl-21e9); and a wall whose lineage names nothing is the
//! bl-2291 sentence unchanged. Fact first, remedy last, in every one.
//!
//! **The remedy is per row, because the act is** ([`remedy`]). `bz --login`
//! serves oauth rows only — [`ProviderRow::login_blocked`] is brazen's own
//! answer to that — so `/login` is the move for exactly those, and every other
//! credential model's secret is a value in the wall's own `config.toml`, which
//! is `/config brazen`.

use crate::config_edit::brazen::ProviderRow;
use crate::model_pick::WORKER_ROLE;

use super::Unready;

/// The one sentence for each way a wall would reach no model.
pub(super) fn say(unready: &Unready, rows: &[ProviderRow]) -> String {
    match unready {
        Unready::Undeclared { provider } => undeclared(provider, rows),
        Unready::Uncredentialed { row } => uncredentialed(row),
        Unready::Wall => wall(rows),
    }
}

/// The act that would give `row` a credential, in the words of its own
/// credential model. Only ever asked of a row that holds none.
fn remedy(row: &ProviderRow) -> String {
    if row.login_blocked().is_none() {
        format!("/login {}", row.name)
    } else {
        "write its key into this wall with /config brazen".to_owned()
    }
}

/// A role resolves a row that is here and empty: the bl-1fd0 rung, said about
/// the role the fire will actually resolve rather than about the wall.
fn uncredentialed(row: &ProviderRow) -> String {
    let (provider, act) = (&row.name, remedy(row));
    format!(
        "sign in first: `{WORKER_ROLE}` resolves provider `{provider}`, which holds no credential in \
         this workspace's wall, so a conversation begun here would reach no model — {act}"
    )
}

/// A role resolves a row the wall does not carry (bl-21e9). Not a sign-in: the
/// row has to arrive before it can hold anything, and the operator's other move
/// is to point the role at one that is here.
fn undeclared(provider: &str, rows: &[ProviderRow]) -> String {
    format!(
        "`{WORKER_ROLE}` resolves provider `{provider}`, and this workspace's wall declares no \
         such row, so a conversation begun here would reach no model — a wall holds brazen's \
         built-in rows plus its own config.toml and nothing else, so add the row with \
         /config brazen, or point the role at one that is here with \
         /model {WORKER_ROLE} <provider> <model-id> (rows here: {})",
        names(rows)
    )
}

/// The lineage names no role at all, so the wall itself is the subject — the
/// bl-2291 sentence, unchanged.
fn wall(rows: &[ProviderRow]) -> String {
    format!(
        "sign in first: no provider in this workspace's wall holds a credential, so a \
         conversation begun here would reach no model — /login <provider> (rows: {})",
        names(rows)
    )
}

/// The wall's row names, in brazen's routing order.
fn names(rows: &[ProviderRow]) -> String {
    rows.iter()
        .map(|row| row.name.as_str())
        .collect::<Vec<&str>>()
        .join(", ")
}
