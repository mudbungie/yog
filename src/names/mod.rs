//! **Workspace** names (§3.1) — the one name altitude yog still owns.
//!
//! **A workspace name is the operator's** (§3.1, bl-df65): typed at the
//! New-workspace affordance, or the fixed [`DEFAULT_NAME`] the empty-world
//! bootstrap uses without asking. Nothing mints one. [`validate`] is what the
//! wordlist used to guarantee by construction — the shape, the length, the
//! reserved literal, and the leaf collision — and it governs **creation only**:
//! enumeration classifies by path and never validates, so pre-reversal minted
//! leaves and foreign leaves stay lawful names.
//!
//! **A conversation name is minted, and the mint is not here** (§3.3, bl-aca4
//! consumed at bl-cd38): its one home is lernie, beside the
//! `require_available` uniqueness check it races, because *every* lernie
//! creation path mints on omission and none of them pass through yog. Yog is a
//! consumer — [`lernie::mint`]'s `mint` over the crate's `Rng`/`SplitMix64`,
//! drawn at preview and again at fire so the two cannot drift into two lists.
//! Yog's own wordlist and draw are deleted, not bypassed.

use std::path::PathBuf;

/// The bootstrap default (§3.1): the empty-world start (§3.4) creates its
/// workspace under this fixed name. **A constant, not a config** — there is
/// nothing to delete for severability — and not a mint. Zero workspaces is the
/// only state that takes it, so it cannot collide locally, and the first Enter
/// meets no name picker.
pub const DEFAULT_NAME: &str = "home";

/// bl's terminal `--as` fallback (§3.1). A workspace so named would false-join
/// every unstamped claim, so it is the one literal creation refuses outright.
const RESERVED: &str = "unknown";

/// The §3.1 length bound — path-safe on every §10 target.
const MAX_BYTES: usize = 32;

/// Why a typed workspace name is refused (§3.1). Each variant's message is the
/// **operator-facing** reason the §11 form renders inline: nothing has spawned,
/// so a refusal is a sentence beside the field, never an ops wound.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NameError {
    /// Not `^[a-z0-9]+(-[a-z0-9]+)*$` — including the empty name, which is that
    /// shape with no input rather than a case of its own.
    #[error(
        "a name is lowercase letters and digits in words joined by single hyphens — like `ops`"
    )]
    Shape,
    #[error("a name is at most {MAX_BYTES} bytes")]
    TooLong,
    #[error("`{RESERVED}` is reserved — it is what an unstamped claim already says")]
    Reserved,
    #[error("`{0}` already exists — pick another name")]
    Taken(String),
}

/// How a typed name is **read** (§3.1, §3.6): surrounding whitespace forgiven,
/// nothing else. One reading, shared by creation's [`validate`] and deletion's
/// typed-name arming ([`crate::delete::Confirmation::armed`]) — so what the
/// operator may type to raise a sphere wall is exactly what they must type to
/// take it down.
pub fn normalize(typed: &str) -> String {
    typed.trim().to_owned()
}

/// The §3.1 shape, spelled as a split rather than a regex dependency: every
/// hyphen-separated segment non-empty and lowercase-ASCII-alphanumeric. An
/// empty name splits to one empty segment and fails here — the general path
/// with no input, not a bootstrap branch.
fn shaped(name: &str) -> bool {
    name.split('-').all(|w| {
        !w.is_empty()
            && w.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    })
}

/// Validate an operator-typed workspace name (§3.1), returning the normalized
/// name to create under. `roots` are the three workspace roots
/// ([`crate::binding::roots`]): a name equal to an existing leaf under **any**
/// of them is refused outright — no suffixing, no prompt-loop, the operator
/// retypes. Equality with `$USER` is deliberately *not* refused (§3.1).
///
/// Collision asks only "does the leaf exist", which is wider than workspace
/// enumeration: a half-created dir still owns its name.
pub fn validate(typed: &str, roots: &[PathBuf]) -> Result<String, NameError> {
    let name = normalize(typed);
    if !shaped(&name) {
        return Err(NameError::Shape);
    }
    if name.len() > MAX_BYTES {
        return Err(NameError::TooLong);
    }
    if name == RESERVED {
        return Err(NameError::Reserved);
    }
    if roots.iter().any(|root| root.join(&name).exists()) {
        return Err(NameError::Taken(name));
    }
    Ok(name)
}

#[cfg(test)]
mod tests;
