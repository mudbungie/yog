//! The conversation's **identity** (DESIGN §3.3): the mint that draws the name
//! and the **legacy** stamp parse. The composer's pre-mint name *preview* left
//! with bl-7cc8 — the wording of a prediction is a seat's, and no reply carried
//! it.
//!
//! The `You are <name>.` stamp no longer composes anywhere (bl-6920): the goal reaches the model exactly as the operator edited it, and
//! identity rides `--name` alone — litany states the stored name fact in its
//! assembled context (litany bl-d55f, released 0.0.4). What remains here is
//! pure: the mint and the two inverses of the retired
//! compose ([`parse_identity_stamp`] / [`strip_identity_stamp`]), kept only to
//! read pre-0.0.4 roots until litany's 30-day retention ages them out. Parse
//! and the shape it reads live together (PRINCIPLES "single source of truth"):
//! [`IDENTITY_LEAD`] is the one record of what the compose used to write.

use litany::mint::{MintError, Rng};

/// The retired stamp's fixed lead — what the pre-bl-6920 compose wrote as the
/// goal's first line (`You are <name>.`), kept as the one shape
/// [`parse_identity_stamp`] recognizes. No live path composes it.
const IDENTITY_LEAD: &str = "You are ";

/// The **legacy** name parse (§3.3, demoted by bl-08f2, orphaned by bl-6920):
/// reads the `You are <x>.` first line the retired compose used to stamp, as
/// the fallback rung of [`crate::git_tree::Agent::name_fact`] — the
/// litany-stored `name` blob is the name's home, and this covers pre-0.0.4
/// roots (no blob) until litany's 30-day retention ages them out, after which
/// the rung is deleted. New roots never match: nothing composes the shape
/// anymore. The stamp was always line one, so the read is that line and no
/// scan. `None` for a foreign or hand-typed root, and for the pre-bl-df65
/// goals whose `<name>` was a workspace — which parse identically and are the
/// one accepted cosmetic misread, bounded by the same retention (§3.3).
pub fn parse_identity_stamp(goal: &str) -> Option<String> {
    let name = goal
        .lines()
        .next()?
        .strip_prefix(IDENTITY_LEAD)?
        .strip_suffix('.')?;
    (!name.is_empty() && !name.contains(char::is_whitespace)).then(|| name.to_owned())
}

/// The goal's **payload** — the retired stamp's other inverse (§3.3): the very
/// text the pre-bl-6920 compose prepended to, or the goal verbatim when no
/// stamp leads it — which is every post-bl-6920 root, every foreign root, and
/// every hand-typed one: the general path. Line-wise, exactly as the compose
/// was: the first line leaves with the blank line that separated it. This is
/// what keeps the display ladder's rungs mutually exclusive for legacy roots —
/// rung two is drawn from the payload, so a stamped conversation never
/// previews as its own identity line.
pub fn strip_identity_stamp(goal: &str) -> String {
    if parse_identity_stamp(goal).is_none() {
        return goal.to_owned();
    }
    goal.split_once('\n')
        .map_or_else(String::new, |(_stamp, payload)| {
            payload.trim_start_matches('\n').to_owned()
        })
}

/// Mint the conversation's own name (§3.3, bl-df65): **litany's** mint
/// ([`litany::mint::mint`], the crate's one home for it since bl-aca4), drawn
/// through the crate yog already links so preview and spawn cannot drift into
/// two wordlists. `occupied` is the **per-workspace** set — the names the
/// target workspace's living agents wear, each agent's
/// [`crate::git_tree::Agent::name_fact`] (the litany-stored blob, with this
/// module's legacy stamp parse as fallback while pre-0.0.4 roots live). No
/// cross-workspace enumeration: workspaces are isolation walls, so global
/// uniqueness would buy nothing.
pub(super) fn mint_conversation(occupied: &[String], rng: &dyn Rng) -> Result<String, MintError> {
    litany::mint::mint(rng, &occupied.iter().cloned().collect())
}
