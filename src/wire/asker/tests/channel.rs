//! **What differs per channel** (REMOTE §8.2, bl-670c): the loopback asker
//! seats the window it is the window of, and an entry asker seats nothing and
//! answers its own sentence.

use super::*;

/// The whole read path in one test: the window seats its own leaf, asks over
/// loopback mTLS presenting that leaf, and the frame reads a decoded `Reply`
/// it never waited for.
#[test]
fn the_window_seats_itself_and_paints_a_decoded_reply() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut asker, mut link) = wired(
        &tmp,
        vec![json!({"ok": true, "kind": "clients", "rows": [
            {"client": "laptop", "present": true, "tools": []}]})],
        &["home", "away"],
    );
    // Nothing declared: a pass still seats the window, and asks nothing.
    assert_eq!(asker.pass(), 0);
    let window = crate::registry::window();
    assert_eq!(
        crate::registry::registered(tmp.path(), &window),
        ["away".to_owned(), "home".to_owned()].into_iter().collect(),
        "the engine seats its own window in every workspace it enumerates"
    );

    let question = json!({"op": "clients", "workspace": "home"});
    assert!(frame(&mut link, &question).is_none(), "nothing landed yet");
    frame(&mut link, &question);
    assert_eq!(asker.pass(), 1);
    let Some(Ok(Reply::Clients(rows))) = frame(&mut link, &question) else {
        panic!("the reply decodes to the roster");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].client, "laptop");
    assert!(rows[0].present);
}

/// A registration already written is one directory read, not a second write —
/// which is what makes the seating free on every pass.
#[test]
fn a_seating_that_is_already_there_writes_nothing() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut asker, _link) = wired(&tmp, Vec::new(), &["home"]);
    asker.pass();
    let seat = crate::registry::registrations(tmp.path(), &crate::registry::window()).join("home");
    let before = std::fs::metadata(&seat).expect("seated").modified().ok();
    asker.pass();
    assert_eq!(
        std::fs::metadata(&seat).expect("seated").modified().ok(),
        before
    );
}

/// **An entry channel's asker seats nothing** (REMOTE §4.1, §1.4) and answers
/// its own sentence in place of every question standing on it (§8.2).
///
/// A registration is a file on the *host's* disk, written by the operator who
/// owns that box; nothing on this side of the wire may write one, so the entry
/// asker holds no enumeration and no registry root to write into — which is
/// what the contrast below states: the same pass, on the loopback channel,
/// seats the window in everything this engine enumerates.
#[test]
fn an_entry_channels_asker_seats_nothing_and_answers_its_own_sentence() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut local, _link) = wired(&tmp, Vec::new(), &["home"]);
    let window = crate::registry::window();

    let (mut entry_link, end) = pair();
    let mut entry = Asker::entry(
        Err("cobalt is an empty entry".to_owned()),
        end,
        Arc::new(NoRepaint),
    );
    let question = json!({"op": "clients", "workspace": "cobalt"});
    frame(&mut entry_link, &question);
    frame(&mut entry_link, &question);
    assert_eq!(entry.pass(), 1, "the question was answered, not skipped");
    assert_eq!(
        frame(&mut entry_link, &question),
        Some(Err("cobalt is an empty entry".to_owned())),
        "an entry that exists is the answer to its name even when it cannot \
         be dialled"
    );
    assert!(
        crate::registry::registered(tmp.path(), &window).is_empty(),
        "and the pass wrote no registration anywhere"
    );

    local.pass();
    assert_eq!(
        crate::registry::registered(tmp.path(), &window),
        ["home".to_owned()].into_iter().collect(),
        "while the loopback channel's asker seats the window it is the window of"
    );
}
