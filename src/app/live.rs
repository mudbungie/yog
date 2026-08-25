//! **The live tail at the seat** (DESIGN §7.2, REMOTE §3; bl-54f7, bl-73e7):
//! the model's side of the follow lane, and the wire read and act paths beside
//! it.
//!
//! Text did not stream back in while the model was thinking or writing, and the
//! fix has been rebuilt once. bl-54f7 put a **follower thread in the window**,
//! reading the open `response.json` off disk at 16 ms and folding the result
//! onto the snapshot the frame painted. That was right while the window derived
//! its own content; the remote split (REMOTE §9.7) then moved every §11 read to
//! the wire, and the follower kept running — publishing a fold onto
//! `AppModel::snap` that no seat read any more, and asking for up to sixty
//! repaints a second on behalf of nobody. bl-73e7 deleted it and moved the
//! **mechanism** to the engine (`boundary::follow`), where the same incremental
//! read now answers a held [`Query::Follow`](crate::boundary::Query::Follow).
//!
//! What is left here is the accessor pair a frame needs and nothing else:
//! declare the followed conversation, read whatever fold has landed. The
//! transport is [`wire::lane`](crate::wire::lane); the splice onto the committed
//! transcript is `shell::inspector::reads`, at
//! [`Transcript::with_live`](crate::transcript::Transcript::with_live) — the
//! seam that was built for exactly this split and, until now, had only ever
//! been used on one side of it.
//!
//! **The in-memory carve-out is gone with the follower.** §7.2's live tail used
//! to be RAM the window held and could not re-derive; now it is an answer, like
//! every other thing the frame paints — arriving faster than the derivation
//! does, but arriving over the same wire, from the same fold, described by the
//! same function. There is nothing left to keep a dead end away from, so the
//! rule that guarded it (no accessor from the model to the tail) is retired
//! rather than weakened.

use std::sync::Arc;

use super::Snapshot;

impl super::AppModel {
    /// The worker's derivation — the **memo** key (§7.2 `SnapMemo`) and the
    /// address table a receipt resolves against: read-only and `pub(crate)`,
    /// this is not a second render source.
    pub(crate) fn derivation(&self) -> &Arc<Snapshot> {
        &self.derived
    }

    /// Take the engine's end of the wire read path (REMOTE §1.2, bl-ae05) —
    /// **the whole channel set** since bl-028a, the engine being what knows
    /// which entries this box holds (REMOTE §8.2). Handed over rather than
    /// taken at [`boot`](Self::boot) because the model owns no thread and mints
    /// no handle the engine is the one owner of.
    pub fn adopt_wire(&mut self, channels: crate::wire::channels::Channels) {
        self.wire = channels;
    }

    /// Take the frame's end of the **follow lane** (REMOTE §3, bl-73e7), on
    /// [`adopt_wire`](Self::adopt_wire)'s own terms: minted by the engine
    /// beside the lane's other end, because both ends of a loopback wire belong
    /// to one assembly.
    pub fn adopt_tail(&mut self, tail: crate::wire::lane::Tail) {
        self.lane = tail;
    }

    /// Record why this window's wire is absent (bl-dc14), keeping the FIRST
    /// reason: the engine's own bind refusal outranks the "no seat" that
    /// follows from it, and a wired window never records one at all.
    pub fn refuse_wire(&mut self, reason: String) {
        self.wire_refusal.get_or_insert(reason);
    }

    /// Why this window has no wire — `None` on a wired window. The frame
    /// paints this INSTEAD of the shell (`shell::refusal`): every read and act
    /// crosses the wire (REMOTE §1.2), so controls painted without one only
    /// look actionable, which is the inert window bl-dc14 refuses.
    pub fn wire_refusal(&self) -> Option<String> {
        self.wire_refusal.clone()
    }

    /// **Ask the wire** (REMOTE §1.2, §3): declare `question` standing and read
    /// whatever answer has landed for it. Never blocks and never dials — the
    /// [`Asker`](crate::wire::asker::Asker) does both, off-frame, at human
    /// cadence — so a surface built on this paints one cadence period behind
    /// the world and the frame stays at its rate no matter what the engine is
    /// doing.
    pub fn wire_ask(&mut self, question: &serde_json::Value) -> Option<crate::wire::link::Landed> {
        self.wire.ask(question)
    }

    /// **The union roster** (REMOTE §8.2, bl-028a): every channel's workspace
    /// slice, each row carrying the channel it came from — the window's own
    /// engine, or the entry leaf. [`wire_ask`](Self::wire_ask)'s shape for the
    /// one read that is a union rather than a routed question: a workspace is a
    /// workspace, and which engine hosts it is a fact painted on it.
    pub fn wire_roster(&mut self) -> Vec<crate::wire::channel::RosterRow> {
        self.wire.roster()
    }

    /// **Which §8.2 entry hosts `name`**, or `None` for this window's own
    /// engine — a selection out of the union roster the frame already holds, so
    /// it costs no ask of its own.
    ///
    /// [`wire_roster`](Self::wire_roster) drops the origin for every seat that
    /// paints a workspace *row*, which is right: a row is a row wherever it
    /// came from. This is the one question for which the channel IS the answer
    /// — the §8.1 provider gate judges a wall on the far side of it, and
    /// [`start_path`](Self::start_path) below asks it to find out whether this
    /// box has a directory for the workspace at all.
    pub fn hosting_entry(&mut self, name: &str) -> Option<String> {
        self.wire_roster()
            .into_iter()
            .find(|r| r.row.workspace == name)
            .and_then(|r| r.origin.label())
    }

    /// **Where the workspace `name` lives on THIS box, if anywhere** (REMOTE
    /// §8.2, bl-e349) — one rule with three answers and no branch on a mode:
    ///
    /// - an enumerated name is its own path
    ///   ([`workspace_path`](Self::workspace_path));
    /// - a name **no channel holds** is one a `Prepare` is about to found
    ///   *here*, so it is spelled the §3.1 way, under yog's flat names root.
    ///   That is the frame's side of the chokepoint's own rule
    ///   (`dispatch::resolve_workspace`: *"a `Prepare` naming an unenumerated
    ///   workspace resolves to yog's flat names root"*), and the shape the §11
    ///   raise and the bootstrap `home` have always taken;
    /// - a name a §8.2 **entry** holds answers `None`. REMOTE §8.2, verbatim:
    ///   *"A remote name still has no local PATH, on purpose. `Snapshot::ws_path`
    ///   resolves the painted enumeration, whose members are directories on this
    ///   box; a workspace hosted elsewhere has none."*
    ///
    /// `None` is the fact the start's frame-side folds need in order to **skip**
    /// rather than invent. The §3.4 raise claim, the §3.4 start claim and the
    /// §3.4 pending echo are every one of them keyed by path, and every one of
    /// them is local optimism standing in for a wire read that will arrive on
    /// the entry's own slice a moment later.
    ///
    /// **Inventing a path was the defect** (bl-e349). `focused_workspace()`
    /// answers `None` for a workspace an entry hosts exactly as it does for no
    /// focus at all, so the start flow read the two states as one and
    /// substituted the §3.1 bootstrap default: a start fired at a workspace held
    /// elsewhere founded a LOCAL workspace named `home`, focused it, and ran the
    /// operator's goal in it.
    pub fn start_path(&mut self, name: &str) -> Option<std::path::PathBuf> {
        if self.hosting_entry(name).is_some() {
            return None;
        }
        Some(
            self.workspace_path(name)
                .unwrap_or_else(|| self.roots.names().join(name)),
        )
    }

    /// **Follow the tail** (REMOTE §3, bl-73e7): declare `question` the lane's
    /// subject and read whatever fold has landed for it. The
    /// [`wire_ask`](Self::wire_ask) shape one lane over, and deliberately so —
    /// a seat cannot tell from here that one of the two arrives at write
    /// cadence and the other at ask cadence, which is what keeps the fallback
    /// invisible when the lane is down.
    pub fn tail_ask(&mut self, question: &serde_json::Value) -> Option<crate::git_tree::Stream> {
        self.lane.ask(question)
    }

    /// Whether any standing question is still unanswered — a **driven** frame's
    /// settle condition and nothing else's (bl-44e9); the window itself never
    /// asks, because a surface paints what it has.
    #[cfg(test)]
    pub fn awaiting(&self) -> bool {
        self.wire.awaiting()
    }
}

#[cfg(test)]
mod tests;
