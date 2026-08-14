//! The shell's reads of the world clock and its entropy (§7.2, §4.2): the
//! three mintings that can only happen at the process boundary, in one place
//! because each is a fact two seats must agree on. The frame takes them as
//! parameters and stays testable; nothing here is derived, cached, or held.

/// An entropy seed for the conversation-mint RNG (§3.3), held stable in
/// [`StartState::mint_seed`] so the composer's preview and its fire agree. Read
/// **once per session** (bl-dd3d): the seed still lives only as long as the
/// prediction it backs, but a landed fire takes its successor off the spent
/// seed's own stream ([`StartState::spend_mint`], bl-28ba) rather than reading
/// this clock again — one entropy read per run is what lets a test pin a whole
/// session's names by pinning its first seed. Not
/// secret — the mint is collision-avoidance, and the occupied-set check is what
/// guarantees uniqueness. Yog's own seat in the mint seam since bl-cd38: the
/// draw is `lernie::mint`'s, but "now" has one home and it is here, so the seed
/// is minted at this boundary and handed to `SplitMix64::from_seed` (never the
/// crate's own `from_entropy`, which would be a second reading of the clock).
pub(crate) fn entropy_seed() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64);
    nanos ^ (u64::from(std::process::id()) << 32)
}

/// The wall-clock `ops.jsonl` timestamp (§4.2) for the verb dispatchers, which
/// stay clock-free and testable by taking it as a parameter. Delegated to the
/// crate's one time seam ([`Clock::stamp`](crate::ui_state::Clock::stamp)) —
/// the §7.2 worker writes its own ops lines from the *injected* clock, and two
/// spellings of "now, as a string" would be two facts.
pub fn now_ts() -> String {
    crate::ui_state::Clock::stamp(&crate::ui_state::SystemClock)
}

/// Wall-clock unix seconds for every seat that dates something (§11): the
/// conversation list's ages and the in-flight strip's elapsed (§5.1 #28). Minted
/// here at the shell boundary like [`now_ts`], in **one** place — two spellings
/// of "now, in seconds" would be two facts, and the two seats must agree on what
/// `42s` means.
pub(crate) fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}
