//! **Fixture worlds** (bl-8741): named, deterministic world states an external
//! client harness can dial and render.
//!
//! A seat's snapshot harness and an emulator screencap loop both need the same
//! thing and neither can reach for it: a yog serving a *known* world, at an
//! address they can dial, laid the same way every run. The suite has fabricated
//! substrate for a long time — `test_support`, and the integration crate's
//! `AgentFixture` — but both are `#[cfg(test)]` and crate-internal, so nothing
//! outside this repository has ever been able to spend them. **This module is
//! that machinery, reached by a verb**, for the reason `wire-certs.sh` became
//! `yog wire-certs` (REMOTE §8): *"an installed binary has no repository to find
//! a script in"*, and every consumer of this one is in another repository.
//!
//! # The contract
//!
//! ```text
//! root=$(yog fixture busy | jq -r .root)     # lay the state, take the root
//! XDG_DATA_HOME="$root" yog &                # boot an engine on it
//! # …dial the address, render, compare…
//! kill %1 && rm -rf "$root"                  # tear it down
//! ```
//!
//! **It lays and prints; it does not boot.** The consumer owns the engine
//! process because the consumer is the one that has to kill it — a verb that
//! parked would hand a harness a live child to parse stdout from, which is
//! worse at exactly the moment it matters. Booting is `XDG_DATA_HOME=<root>
//! yog` and nothing else: §16.2's anchor is the whole of the nesting, so
//! pointing it at a scratch root is what keeps a fixture off the operator's own
//! world. `make fixture STATE=<name>` is the one-command door over the pair.
//!
//! # What makes it deterministic
//!
//! Every byte a state contains is a `&'static str` in [`roster`]; every commit,
//! message and step is dated from the recipe's own offsets rather than from the
//! laying machine's clock; and the address is **stated** before the engine
//! binds, because self-provisioning writes `127.0.0.1:0` and only the listener
//! ever learns what that became ([`crate::wire::provision`]).
//!
//! The residual is named rather than hidden. yog serves derived ages
//! (`age_secs` on a conversation row is `now - last_action_unix`), and the
//! engine's clock is the system's — there is no environment seam that fakes it
//! and this module does not add one, because a product that can be told to lie
//! about the time is worse than a harness that normalises. So [`Laid::origin`]
//! reports the second every offset was measured from, and a harness that wants
//! an exact age computes it.
//!
//! # Two premises the tree corrects
//!
//! - **A *speaking* conversation is not a file.** `AgentState::InFlight` is
//!   derived from an open `response.json` write fd and a held executor lock
//!   (§3.5), so no static tree can be one. A `Streaming` step lays the bytes
//!   and [`Laid::hold`] names the two paths a harness opens to complete it —
//!   one line of shell (`exec 9<dir`), in the process that already owns the
//!   engine.
//! - **`wound_grace` is the server's, and a laid wound must outlive it.** The
//!   §7.2 catch-up window is spent on the engine (bl-776a): a no-response wound
//!   is stated only once the step's own call start is older than it, so a
//!   fixture that laid its step at *now* would answer no wound at all. The
//!   `wound` state dates its step from the recipe's own offsets like everything
//!   else here, which is why it reads as one.

/// How a byte gets written — the four primitives every writer here spends.
pub mod disk;
/// The disk writer: a recipe onto a scratch root, through the production folds.
pub mod lay;
/// Where a laid state's pieces go — the path arithmetic, and no effect.
pub mod places;
/// What a named state IS — the declarative vocabulary, and nothing more.
pub mod recipe;
/// The named states themselves, and the whole of what a consumer may ask for.
pub mod roster;
/// `yog fixture` — the verb, its two environment readings and its refusals.
pub mod verb;

use std::path::PathBuf;

/// What one `yog fixture <state>` laid — the whole consumer contract, printed
/// as one JSON object because a harness in another language should need no
/// parser of its own and no second document to look a path up in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Laid {
    /// The state's name, echoed so a log line is self-describing.
    pub state: String,
    /// The data root — the `XDG_DATA_HOME` an engine is booted with, and the
    /// one path a teardown removes.
    pub root: PathBuf,
    /// `host:port` the engine will bind and a seat dials. Stated, not
    /// discovered: it is written into the material before anything binds.
    pub address: String,
    /// The operator CA both ends verify against.
    pub anchors: PathBuf,
    /// The client leaf and its key — what a harness presents to be admitted.
    pub chain: PathBuf,
    pub key: PathBuf,
    /// The second every `age_secs` in this state was measured back from.
    pub origin: i64,
    /// Paths a harness holds open for the run to make a `Streaming` step read
    /// as a live model call: the conversation's `inbox/<id>` directory (the
    /// §2.11 executor lock) and its `response.json` (the §4.4 writer fd).
    /// Empty for every state that needs no live process.
    pub hold: Vec<PathBuf>,
}

impl Laid {
    /// The one line printed on stdout.
    pub fn json(&self) -> String {
        let path = |p: &PathBuf| p.display().to_string();
        serde_json::json!({
            "state": self.state,
            "root": path(&self.root),
            "address": self.address,
            "anchors": path(&self.anchors),
            "chain": path(&self.chain),
            "key": path(&self.key),
            "origin": self.origin,
            "hold": self.hold.iter().map(path).collect::<Vec<String>>(),
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests;
