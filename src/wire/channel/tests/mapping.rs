//! The leaf↔host-name mapping, spent in both directions at this one boundary
//! (REMOTE §8.2) — a question crossing under the host's name, a row landing
//! back under the leaf, and the three shapes that rewrite nothing.

use super::*;

#[test]
fn a_gesture_crosses_carrying_the_name_its_host_knows() {
    let (mut channel, mut end) = wired(entry("cobalt", "home"));
    channel.ask(&about("cobalt"));
    channel.settle();
    let carried = end.standing();
    assert_eq!(
        carried,
        vec![about("home")],
        "the leaf is the client's word; the host is asked in its own"
    );
}

#[test]
fn an_entry_that_renames_nothing_sends_the_envelope_byte_for_byte() {
    let (mut channel, mut end) = wired(entry("cobalt", "cobalt"));
    channel.ask(&about("cobalt"));
    channel.settle();
    assert_eq!(end.standing(), vec![about("cobalt")]);
}

#[test]
fn a_name_the_entry_does_not_hold_crosses_unrewritten() {
    let (mut channel, mut end) = wired(entry("cobalt", "home"));
    // The host may hold more than one workspace this leaf is registered in
    // (§8.2: lawful, and the operator's); only the entry's own name is mapped.
    channel.ask(&about("second"));
    channel.settle();
    assert_eq!(end.standing(), vec![about("second")]);
}

#[test]
fn a_question_the_codec_cannot_read_crosses_unchanged() {
    let (mut channel, mut end) = wired(entry("cobalt", "home"));
    let nonsense = Value::String("not a gesture".to_owned());
    channel.ask(&nonsense);
    channel.settle();
    assert_eq!(end.standing(), vec![nonsense]);
}

#[test]
fn a_landed_row_is_labelled_with_the_leaf_not_the_host() {
    let (mut channel, mut end) = wired(entry("cobalt", "home"));
    answer(&mut channel, &mut end, &[], &listing(&["home", "other"]));
    let rows = channel.rows();
    assert_eq!(
        rows.iter()
            .map(|r| r.row.workspace.clone())
            .collect::<Vec<_>>(),
        vec!["cobalt".to_owned(), "other".to_owned()],
        "the entry's own name comes back in this box's spelling; a workspace \
         the entry does not name is the host's word, unchanged"
    );
    assert!(
        rows.iter().all(|r| r.origin
            == Origin::Entry {
                leaf: "cobalt".to_owned(),
                host: "home".to_owned()
            }),
        "every row wears the channel it came from"
    );
}

#[test]
fn a_reply_that_names_no_workspace_lands_untouched() {
    let (mut channel, mut end) = wired(entry("cobalt", "home"));
    let models = Reply::Models(vec!["home".to_owned()]);
    answer(&mut channel, &mut end, &[about("cobalt")], &models);
    assert_eq!(channel.ask(&about("cobalt")), Some(Ok(models)));
}

/// **The §8.1 `Prepared` is renamed back too, and it has to be** (bl-e349). The
/// name it carries is handed straight out again as the next act's address —
/// `Action::Prompt` names `prepared.workspace` — so a `Prepared` left in the
/// host's spelling would route its own `Prompt` to a name no entry claims, back
/// to this window's own engine, which is the local misfire the mapping exists
/// to prevent.
#[test]
fn a_landed_prepare_is_renamed_back_so_its_prompt_routes_home_again() {
    let (mut channel, mut end) = wired(entry("cobalt", "home"));
    answer(
        &mut channel,
        &mut end,
        &[about("cobalt")],
        &prepared("home"),
    );
    assert_eq!(channel.ask(&about("cobalt")), Some(Ok(prepared("cobalt"))));
}

/// The other half of the same rule: an entry that renames nothing lands the
/// reply byte for byte, the general path with the two names agreeing.
#[test]
fn an_entry_that_renames_nothing_lands_the_prepare_unchanged() {
    let (mut channel, mut end) = wired(entry("cobalt", "cobalt"));
    answer(
        &mut channel,
        &mut end,
        &[about("cobalt")],
        &prepared("cobalt"),
    );
    assert_eq!(channel.ask(&about("cobalt")), Some(Ok(prepared("cobalt"))));
}

/// A §8.1 prepare answered for the workspace `workspace`.
fn prepared(workspace: &str) -> Reply {
    Reply::Prepared(crate::start::Prepared {
        workspace: workspace.to_owned(),
        binding: None,
        lineage: None,
        goal: "do it".to_owned(),
        origin: crate::opslog::Origin::Conversation,
    })
}
