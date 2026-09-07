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

use std::collections::BTreeMap;

use crate::config_edit::brazen::{NOT_REQUIRED, ProviderRow};

use super::Unready;

/// The one sentence for each way a wall would reach no model. `role` is the
/// role this fire will be born on (bl-9ced) — `worker` for every start that
/// named none, and the named one otherwise, because a remedy that pointed at
/// `worker` would send the operator to fix a row the start never resolves.
pub(super) fn say(unready: &Unready, rows: &[ProviderRow], role: &str) -> String {
    match unready {
        Unready::Undeclared { provider } => undeclared(provider, rows, role),
        Unready::Uncredentialed { row } => uncredentialed(row, role),
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
fn uncredentialed(row: &ProviderRow, role: &str) -> String {
    let (provider, act) = (&row.name, remedy(row));
    format!(
        "sign in first: `{role}` resolves provider `{provider}`, which holds no credential in \
         this workspace's wall, so a conversation begun here would reach no model — {act}"
    )
}

/// A role resolves a row the wall does not carry (bl-21e9). Not a sign-in: the
/// row has to arrive before it can hold anything, and the operator's other move
/// is to point the role at one that is here.
fn undeclared(provider: &str, rows: &[ProviderRow], role: &str) -> String {
    format!(
        "`{role}` resolves provider `{provider}`, and this workspace's wall declares no \
         such row, so a conversation begun here would reach no model — a wall holds brazen's \
         built-in rows plus its own config.toml and nothing else, so add the row with \
         /config brazen, or point the role at one that is here with \
         /model {role} <provider> <model-id> (rows here: {})",
        names(rows)
    )
}

/// **What act would ready one row** (bl-8523) — the partition the wall's own
/// refusal prints, over the columns that decide it and nothing else.
///
/// It is a statement about a row's **credential model**, true whatever the row
/// currently holds: an oauth row is the row you sign in to, signed in or not.
/// That is why it is total, and why the wall sentence — printed only when no
/// row here holds a credential at all — can print it flat. The declaration
/// order is the reading order the refusal takes, most actionable first.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Act {
    /// `bz --login` serves oauth rows and only oauth rows (§8.3), which is what
    /// [`ProviderRow::login_blocked`] answers.
    SignIn,
    /// Every other keyed model's secret is a value in the wall's own
    /// `config.toml`, so the act is an edit and never a sign-in.
    WriteKey,
    /// A keyless row needs no credential, so nothing readies it *by itself* —
    /// brazen merges its built-in table under every config. What readies it is
    /// the operator naming it in a role, which is their own hand and not the
    /// merge ([`super`]'s keyless clause).
    NameInRole,
    /// A keyless row whose dialect carries no tool declaration can serve no
    /// role either (bl-3d22): every yog turn declares at least the `clients`
    /// tool. It can ready a wall by **no act at all**, and saying so is the
    /// whole of bl-8523.
    Never,
}

/// The act, read off the row's own columns.
fn act(row: &ProviderRow) -> Act {
    if row.credential == NOT_REQUIRED {
        if row.tools_blocked().is_none() {
            Act::NameInRole
        } else {
            Act::Never
        }
    } else if row.login_blocked().is_none() {
        Act::SignIn
    } else {
        Act::WriteKey
    }
}

/// Each act in the imperative the operator can type.
fn phrase(act: &Act) -> &'static str {
    match act {
        Act::SignIn => "sign in with /login",
        Act::WriteKey => "write a key with /config brazen",
        Act::NameInRole => "name in a role with /model <role> <provider> <model-id>",
        Act::Never => "can serve no role",
    }
}

/// The lineage declares no worker at all, so the wall itself is the subject —
/// bl-2291's sentence, with its rows **partitioned by the act each one takes**
/// (bl-8523).
///
/// The flat list it used to print offered `/login <provider>` over every row,
/// the two brazen ships keyless included: a reader who typed `/login
/// claude-code` was told there is *nothing to log in*, and `/model` refuses that
/// same row because `claude_code` declares no tools — so it was offered, could
/// not be signed in to, and could ready a wall by no act at all. A loop with no
/// exit, at the last gate before a first reply, which is the one place the
/// product tells a stranger what to do next. Every fact was already at the site:
/// `credential`, `auth` and `tools` are columns of the rows this is handed.
fn wall(rows: &[ProviderRow]) -> String {
    let mut by_act: BTreeMap<Act, Vec<&str>> = BTreeMap::new();
    for row in rows {
        by_act.entry(act(row)).or_default().push(&row.name);
    }
    let offers: Vec<String> = by_act
        .iter()
        .map(|(act, names)| format!("{}: {}", phrase(act), names.join(", ")))
        .collect();
    format!(
        "sign in first: no provider in this workspace's wall holds a credential, so a \
         conversation begun here would reach no model — {}",
        offers.join("; ")
    )
}

/// The wall's row names, in brazen's routing order.
fn names(rows: &[ProviderRow]) -> String {
    rows.iter()
        .map(|row| row.name.as_str())
        .collect::<Vec<&str>>()
        .join(", ")
}
