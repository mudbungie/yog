//! The §3.3 naming invariant, machine-held (bl-df72): **no seat formats an
//! agent id as a display name**. The operator met the violation as "the agent
//! list at the top is named just some incoherent timestamp" — the descent-tree
//! member row painting `agent_id` raw. The one naming rule is the display
//! ladder (`crate::nav::convs::display_name` and its seats); its floor may be
//! the id — an id is a fact — but only the ladder spells it, and the id's
//! other seat is the hover.
//!
//! **Held on values, not on field names** (bl-45c7). The scan this replaces
//! read the tree's source and flagged a paint call whose argument span
//! mentioned one of a hand-listed set of identifiers — `agent_id`, `root_id`,
//! and, after bl-3aa1, `sender`. That list was the defect. bl-63a1 recorded the
//! lesson verbatim — *"the first scan forbade only `agent_id`, so a seat
//! painting the same fact under its other name would have passed"* — and the
//! lesson still recurred: a deposit's `from` deserializes into `sender`, a
//! third spelling, and the scan sat silent while a seat painted a four-token
//! ancestry chain in plain sight. A scan whose strength is a vocabulary decays
//! every time anyone renames the fact, and **nothing fails when it does** — it
//! goes on passing, on the subset it happens to know. Vacuous by omission.
//!
//! So the question moved from the name to the value: an agent id has a shape
//! ([`is_stamp`], the one stamp grammar the ladder itself reads), and the
//! window is asked what it *painted*. A seat that leaks an id under
//! `originator`, `author`, `who` or any name nobody has thought of yet is
//! caught by the same sentence as one that leaks it under `agent_id`, because
//! the scan never learns a name at all.
//!
//! What it costs: the source scan indicted a seat whether a fixture reached it
//! or not, and this one sees only what the window paints. That is the trade the
//! ball took deliberately — a guard over every seat that knows three of the
//! fact's names is worth less than a guard over the reachable seats that knows
//! the fact itself, and §11's fixture reaches the whole window.
//!
//! **Which rung a seat may take is [`rungs`]** — the paint half, split off at
//! §12's budget: this file holds what no seat may spell, that one holds what a
//! seat must spell instead when the ladder has a name to give it.

/// The seats whose defect was the wrong rung rather than an unlawful spelling.
mod rungs;

use super::fixture::World;
use crate::cli_outbound::Cli;
use crate::nav::convs::{id_floor, is_stamp};

/// A **named** root wearing a real litany id — the shape every seat that wants
/// a title must resolve past.
const ROOT: &str = "20260803T045643Z-1e5f99d4";
/// A nameless descent child of [`ROOT`], its id carrying the whole ancestry
/// chain — one `<stamp>-<hash>` pair per generation (§2.3). This is the value
/// the operator called unparseable: the ladder's floor may spell its **terminal
/// generation**, and no seat may spell more.
const CHILD: &str = "20260803T045643Z-1e5f99d4-20260804T101112Z-abcdef01";
/// A chained id **no branch carries** — a reaped subagent whose mail outlived
/// it. Nothing above the floor exists for it, which is what keeps an id on
/// screen at all now that a *resolvable* sender wears its name (bl-b6d0).
const GONE: &str = "20260803T045643Z-1e5f99d4-20260805T220100Z-deadbeef";
/// A **named peer** — a second root, and the sender bl-b6d0 was filed on. One
/// generation, so its id and its floor spelling are the same string: anything
/// the row paints in place of the name is the whole id.
pub(super) const PEER: &str = "20260731T101112Z-abcdef01";

/// The separators a seat puts between an id and whatever sits beside it, so a
/// run like `✉ <id> · t0` yields the id as its own token. Splitting on these is
/// the whole of the scan's knowledge about layout; it knows nothing about which
/// seat painted what.
const BREAKS: &[char] = &[
    ' ', '\n', '\t', '·', ':', ',', '(', ')', '[', ']', '"', '\'', '—', '→', '⟩', '⟨',
];

/// Every id-shaped run in `painted`: a token one of whose `-` segments is a
/// litany stamp. **This is the derivation** — the scan asks the value what it
/// is, never asks a field what it is called.
fn id_runs(painted: &str) -> Vec<String> {
    painted
        .split(BREAKS)
        .filter(|token| token.split('-').any(is_stamp))
        .map(str::to_owned)
        .collect()
}

/// The whole window over a world built out of real litany ids: a named root, a
/// chained child of it unfolded into the list, and three inbox deposits — the
/// §2.11 `from:` field, which is the carrier the vocabulary scan had never heard
/// of until the defect had already shipped.
///
/// **Three senders, because bl-b6d0 moved that header onto the ladder's first
/// rung.** A deposit from an agent the roster carries paints that agent's name
/// now, so [`CHILD`] and [`PEER`] spell no id at all and [`GONE`] — mail whose
/// sender was reaped — is where the ladder still falls to the floor. Without it
/// this scan would enumerate nothing, which its own guard calls a failure.
pub(super) fn painted_over_ids(tab: crate::keymap::InspectorTab) -> String {
    let (litany, bl) = (Cli::new("/yog-absent-litany"), Cli::new("/yog-absent-bl"));
    let mut world = super::inbox_composer::quick(super::fixture::world());
    let ws = world.ws.clone();
    world.add_root(ROOT, "cormorant");
    world.add_root(PEER, "peregrine");
    world.add_child(ROOT, CHILD);
    deposit_from(&world, ROOT, CHILD);
    deposit_from(&world, ROOT, GONE);
    deposit_from(&world, ROOT, PEER);
    world.model.mark_dirty([ws.clone()]);
    world.converge();
    world.model.focus_agent(&ws, ROOT);
    world.model.select_tab(tab);
    // Unfold the root: since bl-fa82 a member is a row of the conversation
    // list, and since bl-8905 that is the only place a child paints.
    world.state.expanded.insert(ROOT.to_owned());
    super::painted(&mut world, &litany, &bl)
}

/// Land a deposit in `agent`'s inbox whose `from:` is another **agent id**
/// (§2.11) — what one agent's `litany message` to another leaves behind. The
/// sibling helper in [`super::inbox_composer`] deposits from `user`, which is
/// precisely the sender that carries no id and so proves nothing here.
fn deposit_from(world: &World, agent: &str, sender: &str) {
    let dir = world.ws.join("inbox").join(agent);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(format!("{sender}-001.md")),
        format!("---\nfrom: {sender}\ndeposited_at: t0\n---\nreporting back"),
    )
    .unwrap();
}

/// **The invariant.** Every id-shaped run the window paints is one the display
/// ladder put there — [`id_floor`]'s terminal generation, and never a syllable
/// more. A seat that hands an id straight to a label is caught whatever the
/// field it read it out of was called, because the sentence names no field.
///
/// Asserted on both inspector tabs that seat a deposit, since §5.1 #11's
/// `✉ from · at` header has three seats and the `from` in it is an agent id.
#[test]
fn no_painted_run_spells_more_of_an_agent_id_than_the_ladder_does() {
    let lawful = [ROOT, CHILD, GONE].map(|id| id_floor(id).to_owned());
    for tab in [
        crate::keymap::InspectorTab::Transcript,
        crate::keymap::InspectorTab::Inbox,
    ] {
        let painted = painted_over_ids(tab);
        let runs = id_runs(&painted);
        // Enumerating nothing is itself a failure: a world whose ids never
        // reached the paint layer would let any seat pass, which is the exact
        // shape of rot the vocabulary scan died of.
        assert!(
            !runs.is_empty(),
            "the {tab:?} drive painted no id at all — the fixture no longer \
             reaches a seat that spells one, so this scan is asserting nothing:\n{painted}"
        );
        for run in &runs {
            assert!(
                lawful.contains(run),
                "a seat spells an agent id past the ladder's floor: {run:?} on \
                 {tab:?}, where §3.3 allows only {lawful:?} — every title rides \
                 the display ladder, and the id's seats are the ladder's floor \
                 and the hover:\n{painted}"
            );
        }
    }
}

/// **The scan cannot pass vacuously.** The bands drive's idiom (bl-58e4: show
/// the reader a frame laid out the retired way), applied to a scan rather than
/// a layout — the defect bl-3aa1 shipped is handed to [`id_runs`] verbatim and
/// must come back indicted. Without this, a reduction that quietly stopped
/// recognizing a stamp would turn the invariant above green and silent, which
/// is precisely how its predecessor failed.
#[test]
fn the_scan_indicts_the_defect_that_shipped() {
    let leaked = format!("✉ {CHILD} · t0");
    let runs = id_runs(&leaked);
    assert_eq!(
        runs,
        vec![CHILD.to_owned()],
        "the whole chain is read out of the row as one run"
    );
    assert!(
        !runs.contains(&id_floor(CHILD).to_owned()),
        "and it is not the floor's spelling, so the invariant above fails on it"
    );
    // The floor's own spelling is lawful, so the scan indicts the leak rather
    // than the seat that behaved: a check that flagged both would be no check.
    assert_eq!(
        id_runs(&format!("✉ {} · t0", id_floor(CHILD))),
        vec![id_floor(CHILD).to_owned()]
    );
    // And a sender that is not an agent id at all carries no stamp grammar, so
    // the operator's own deposits are invisible to the scan — no branch.
    assert!(id_runs("✉ user · t0").is_empty());
}
