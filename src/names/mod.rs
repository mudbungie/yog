//! Names, at the two altitudes §3.1/§3.3 put them at.
//!
//! **A workspace name is the operator's** (§3.1, bl-df65): typed at the
//! New-workspace affordance, or the fixed [`DEFAULT_NAME`] the empty-world
//! bootstrap uses without asking. Nothing mints one. [`validate`] is what the
//! wordlist used to guarantee by construction — the shape, the length, the
//! reserved literal, and the leaf collision — and it governs **creation only**:
//! enumeration classifies by path and never validates, so pre-reversal minted
//! leaves and foreign leaves stay lawful names.
//!
//! **A conversation name is minted** (§3.3, one word since bl-d12f): a single
//! word from an embedded wordlist — `gecko`. The mint is a **pure function over
//! an injected RNG and an occupied set** ([`mint`]): one RNG draw picks a start
//! index into the wordlist, then the scan walks forward with wraparound,
//! discarding each occupied word for the next, to the first unoccupied name.
//! Collision retry is that scan; its bound is the wordlist itself — exhaustion
//! is the scan running the whole pool out ([`MintError::Exhausted`]). No retry
//! budget, no probabilistic termination, no unbounded loop. The occupied set is
//! the caller's (§3.3: the stamped names of the target workspace's live roots),
//! assembled never stored.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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

/// The embedded pool (§3.1). Provenance and licence are recorded in the file's
/// own header; it is data, so it ships in the binary via `include_str!`.
const WORDS_TXT: &str = include_str!("words.txt");

/// The one way a mint fails: every word in the pool is already taken.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MintError {
    /// All `n` words of the list are occupied.
    #[error("name pool exhausted: all {0} words are occupied")]
    Exhausted(usize),
}

/// The injected randomness the mint is pure over. A trait rather than a
/// concrete generator so a test drives the mint with a scripted draw and the
/// production seeding stays out of the pure path.
pub trait Rng {
    /// The next 64 random bits.
    fn next_u64(&mut self) -> u64;
}

/// SplitMix64 — the production [`Rng`]. Chosen because it is ~6 lines of
/// wrapping arithmetic: the mint needs one draw per name, and a `rand`
/// dependency for that is not worth the supply-chain surface (AGENTS.md rule 6:
/// zero new dependencies).
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// A generator from an explicit seed — reproducible, and the seam the
    /// entropy path funnels through.
    pub fn from_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    /// A generator seeded from the wall clock and this process's id. Neither
    /// input is secret — the mint is a collision-avoidance device, not a
    /// security one, and the occupied-set check is what actually guarantees
    /// uniqueness.
    pub fn from_entropy() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self::from_seed(nanos ^ (u64::from(std::process::id()) << 32))
    }
}

impl Rng for SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// The embedded wordlist as words: non-blank, non-comment lines, trimmed.
fn wordlist() -> Vec<&'static str> {
    WORDS_TXT
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// The mint over an explicit wordlist — the whole algorithm, kept
/// list-injectable so tests exercise collision retry and exhaustion on a
/// tiny pool instead of the embedded one. The retry is bounded by the pool:
/// each occupied word is discarded for the next with wraparound, and one full
/// lap proves exhaustion exactly — no free name is ever missed, no loop runs
/// unbounded. Fallible reads (rule 4): an out-of-range index cannot occur, and
/// a missing word reads as empty rather than panicking.
fn mint_from(
    words: &[&str],
    rng: &mut dyn Rng,
    occupied: &HashSet<String>,
) -> Result<String, MintError> {
    let pool = words.len();
    let start = (rng.next_u64() % pool.max(1) as u64) as usize;
    for step in 0..pool {
        let name = words
            .get((start + step) % pool)
            .copied()
            .unwrap_or_default();
        if !occupied.contains(name) {
            return Ok(name.to_owned());
        }
    }
    Err(MintError::Exhausted(pool))
}

/// Mint a name from the embedded wordlist (§3.3): the first single word not in
/// `occupied`, scanning from an RNG-chosen start. Pure — same RNG and same
/// occupied set, same name.
pub fn mint(rng: &mut dyn Rng, occupied: &HashSet<String>) -> Result<String, MintError> {
    mint_from(&wordlist(), rng, occupied)
}

#[cfg(test)]
mod tests;
