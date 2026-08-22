//! **The send is on the glass before the driver writes anything** (bl-915e),
//! driven through the real window.
//!
//! Operator: *"you send the message, but before it goes into the inbox, it's
//! just missing for a minute, then it responds. The UI should be pretty much
//! immediate: you write a message, it goes into inbox, pushing the inbox line
//! up; right now, it looks like it's waiting for the send."* Nothing was
//! blocked — the frame renders a completed derivation (§7.2) and the operator's
//! text had no home in one until the detached driver wrote it.
//!
//! Both beats are shaped the same way, because the fix is one mechanism (§7.2):
//! type, press Enter, and read the very next frame **with no derivation having
//! run** — `World::converge` is deliberately not called, and the assertions
//! below prove the derivation really is still empty rather than merely assuming
//! it. Then land the message on disk, converge, and prove the echo gave its seat
//! up instead of doubling it.

/// **The drive's shared fixtures** — the world, the gestures and the paint
/// reads, in their own file at §12's budget (bl-56c6, when a third beat file
/// wanted them too).
mod harness;
/// **One message is one queue row** (§7.2, bl-78d8) — the seat where the echo
/// must *give way* rather than appear, split off at §12's budget on that seam.
mod queue;
/// **The §3.4 start window** (bl-56c6) — Enter to the driver's first write, the
/// stretch in which the conversation has no address at all.
mod window;

use super::fixture::{fake_lernie, seed_world};
use super::screen::Screen;
use harness::{
    MINTED_FIRST, SAID, branches, converge_ws, faded_user, fills, quick, rows_named, say, shot,
    world,
};
use tempfile::tempdir;

/// **A start.** Enter mints a name and detaches `lernie prompt`; the driver has
/// written nothing. The very next frame must carry the goal anyway — as a §11
/// row of its own, faded, in the operator's own words.
#[test]
fn a_start_is_a_row_in_the_operators_own_words_on_the_next_frame() {
    let bin = tempdir().unwrap();
    let mut world = quick(world());
    seed_world(&world);
    let screen = Screen::with_lernie(fake_lernie(bin.path()));
    assert!(screen.idle(&mut world), "the cursor starts in the composer");
    let worktrees = branches(&world);

    let before = screen.text(&mut world);
    assert!(
        !before.contains(SAID),
        "nothing says it before the operator does:\n{before}"
    );
    say(&screen, &mut world);

    // No `converge`, and the substrate says so: the detached driver has written
    // no worktree, so there is nothing for any derivation to have found.
    // Asserted rather than assumed — without this the beat would pass just as
    // green on a derivation that had quietly caught up (bl-70b8 shape 2).
    assert_eq!(
        branches(&world),
        worktrees,
        "the driver has written nothing: no branch for a derivation to carry"
    );

    let painted = screen.text(&mut world);
    assert!(
        painted.contains(SAID),
        "the frame immediately after Enter carries what was typed:\n{painted}"
    );
    // And it is the conversation the operator is now *in* (bl-2e8f). The claim
    // used to wait for the branch, so the one row the operator had just made was
    // the one nothing highlighted, with the birth placeholder still filling the
    // centre — operator: "you start a new chat, start typing, and the new chat
    // isn't immediately selected."
    assert_eq!(
        world.model.focus().agent.as_deref(),
        Some(MINTED_FIRST),
        "the start selected what it started, by the minted name its row wears"
    );
    assert!(
        !painted.contains("select a conversation"),
        "so the centre is that conversation, not the placeholder:\n{painted}"
    );
    let pending = rows_named(&world, MINTED_FIRST);
    assert_eq!(
        pending.len(),
        1,
        "exactly one row for the started conversation"
    );
    assert_eq!(
        pending[0].root_id, MINTED_FIRST,
        "keyed by the minted §3.3 name — the only identity a start has yet"
    );
    assert_eq!(
        pending[0].subtitle(),
        SAID,
        "and its preview is the operator's text verbatim"
    );
    assert_eq!(
        pending[0].tone,
        crate::transcript::Tone::Weak,
        "faded: yog's own word for it, not yet a statement (§11)"
    );

    // The driver lands: the branch, its name blob, and the first user message.
    world.add_root("c-2", MINTED_FIRST);
    let messages = world.ws.join("agents/c-2/messages");
    std::fs::create_dir_all(&messages).unwrap();
    std::fs::write(messages.join("001-user.md"), SAID).unwrap();
    // And what the driver wrote is what the driver writes: the operator's
    // payload in `goal.md`, while the context it assembled for the model opens
    // with the §3.7 pinned-instruction frame. The two part here, so the flip
    // below is a claim about which one the row reads (bl-368d).
    std::fs::write(world.ws.join("agents/c-2/goal.md"), SAID).unwrap();
    std::fs::write(
        world.ws.join("steps/c-2/001/request.json"),
        br#"{"model":"m","messages":[{"role":"user","content":[{"type":"text","text":"<file path=\"instructions/00/AGENTS.md\">\nhouse rules\n</file>"},{"type":"text","text":"---\nfrom: operator\n---\n\nunbar it"}]}]}"#,
    )
    .unwrap();
    converge_ws(&mut world);

    let landed = rows_named(&world, MINTED_FIRST);
    assert_eq!(
        landed.len(),
        1,
        "still one row — the echo gave its seat up rather than doubling it"
    );
    assert_eq!(
        landed[0].tone,
        crate::transcript::Tone::Plain,
        "and it brightened: the derivation made it a statement"
    );
    assert_eq!(
        landed[0].root_id, "c-2",
        "the row is now the real branch, keyed by its agent id"
    );
    assert_eq!(
        landed[0].subtitle(),
        SAID,
        "and the flip changed nothing about what it says: still the operator's \
         text, not the assembled context's head"
    );
}

/// **A follow-up.** The `message` verb is piped and its deposit becomes
/// `NNN-user.md` only at the driver's next step boundary — the identical hole,
/// closed by the identical mechanism (§7.2). Here the echo's seat is the §11
/// inbox-composer queue, which is the seat the operator named.
#[test]
fn a_message_joins_the_inbox_queue_faded_and_brightens_when_it_lands() {
    let bin = tempdir().unwrap();
    let mut world = quick(world());
    seed_world(&world);
    let screen = Screen::with_lernie(fake_lernie(bin.path()));
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    // The Transcript tab, so the landed half of the beat has a seat to land in:
    // the echo's seat is the queue above the box, the message's is the chat.
    world
        .model
        .select_tab(crate::keymap::InspectorTab::Transcript);
    screen.text(&mut world);

    let before = world
        .model
        .tree(&ws)
        .and_then(|t| t.agents.iter().find(|a| a.agent_id == "c-1").cloned())
        .expect("the fixture's conversation");
    assert_eq!(before.pending.len(), 1, "the fixture's one landed deposit");
    let landed_messages = before.messages;

    say(&screen, &mut world);

    // Again: no converge, and the substrate is pinned to prove it. The driver
    // has flushed nothing into `messages/`, so anything on the glass is the echo.
    assert_eq!(
        std::fs::read_dir(world.ws.join("agents/c-1/messages")).map_or(0, |d| d.flatten().count()),
        landed_messages,
        "no message has landed: the transcript directory is exactly where it was"
    );

    let out = shot(&screen, &mut world);
    let text = crate::paint_probe::text_of(&out);
    assert!(
        text.contains(SAID),
        "the frame immediately after Enter carries what was typed:\n{text}"
    );
    assert!(
        fills(&out).contains(&faded_user()),
        "and it is faded — yog's own word for it, not yet a statement (§11)"
    );

    // The driver flushes the deposit into the transcript.
    let messages = world.ws.join("agents/c-1/messages");
    std::fs::write(messages.join("003-user.md"), SAID).unwrap();
    converge_ws(&mut world);

    let after = shot(&screen, &mut world);
    assert!(
        !fills(&after).contains(&faded_user()),
        "the faded row is gone: the echo retired onto the landed message"
    );
    assert!(
        crate::paint_probe::text_of(&after).contains(SAID),
        "and the words are still on screen — now the derivation's, not yog's:\n{}",
        crate::paint_probe::text_of(&after)
    );
}
