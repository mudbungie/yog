//! **Busy means queue** (REMOTE §5.7; bl-3b4b, bl-d9ad): a machine that is
//! working, or between two jobs, is still there — its rows stay, and a read
//! aimed at it waits in its mailbox instead of being refused.

use std::time::Instant;

use serde_json::json;
use tempfile::tempdir;

use super::{LISTING, at, models, providers, world};
use crate::bz_host::far::TOOL;
use crate::config_edit::brazen::TABLE;
use crate::registry::mailbox::{Call, Mailbox, Post};
use crate::test_support::foot::{parked, printed, serve};

/// **A machine mid-job stays in sight, and busy means queue** (bl-3b4b,
/// bl-d9ad). It holds no read while it works. The listing keeps its rows and a
/// read naming it is answered, both from what it last said and with no wait;
/// a read it has never answered is **queued behind the job, once**, and said
/// to be. When the job ends the machine is handed every queued read at its
/// next read, and the seat that asks again is answered.
#[test]
fn a_busy_machine_is_asked_behind_its_job() {
    let root = tempdir().unwrap();
    let (deps, mail) = world(root.path());
    let listed = printed(r#"{"models":[{"id":"big","default":true}]}"#);
    let script = vec![Some(printed(LISTING)), Some(listed.clone()), None];
    let (foot, _asked) = serve(&mail, "laptop", script);

    parked(&mail, "laptop");
    assert!(providers(&deps).iter().any(|row| row.name == "sub@laptop"));
    parked(&mail, "laptop");
    let fresh = models(&deps, "sub@laptop");
    assert!(fresh.is_ok(), "{fresh:?}");

    parked(&mail, "laptop");
    let job = Call {
        client: "laptop".to_owned(),
        tool: TOOL.to_owned(),
        input: json!({ "stdin": "a model call" }),
        cwd: None,
    };
    let job = mail.post(0, "driver", &job);
    foot.join().unwrap();
    assert!(!mail.serving("laptop"), "taken, nothing parked behind it");

    let before = Instant::now();
    let rows = providers(&deps);
    assert!(
        before.elapsed().as_secs() < 2,
        "a busy machine costs no wait"
    );
    assert!(
        rows.iter()
            .any(|row| row.name == "sub@laptop" && row.at == Some(at("laptop", "sub")))
    );
    assert_eq!(models(&deps, "sub@laptop"), fresh);
    for _ in 0..2 {
        let queued = models(&deps, "other@laptop").unwrap_err();
        assert!(queued.contains("queued, not refused"), "{queued}");
    }

    mail.complete(0, "laptop", &job, &Post::Capture(printed("done")))
        .expect("the job's own machine");
    let work = mail.take("laptop").expect("one reader").work();
    assert_eq!(work.len(), 3, "three reads asked, each queued once");
    for handed in &work {
        mail.complete(1, "laptop", &handed.id, &Post::Capture(listed.clone()))
            .expect("posted");
    }
    let laptop = crate::registry::Client::parse("laptop").unwrap();
    crate::registry::seen::mark(&deps.state_root, &laptop, 0);
    let (other, _again) = serve(&mail, "laptop", vec![Some(listed)]);
    assert_eq!(models(&deps, "other@laptop"), fresh);
    other.join().unwrap();
}

/// **A machine heard from a moment ago is there** (bl-d9ad): between its last
/// post and its next read a foot is neither parked nor working, and a listing
/// made in that instant still asks it — the ask waits in its mailbox for the
/// read that is coming.
#[test]
fn a_machine_between_jobs_is_asked() {
    let root = tempdir().unwrap();
    let (deps, mail) = world(root.path());
    let laptop = crate::registry::Client::parse("laptop").unwrap();
    crate::registry::seen::mark(&deps.state_root, &laptop, 0);
    assert!(!mail.serving("laptop"));
    let asker = std::thread::spawn({
        let mail = mail.clone();
        move || {
            parked_out(&mail);
            serve(&mail, "laptop", vec![Some(printed(LISTING))])
                .0
                .join()
        }
    });
    assert!(providers(&deps).iter().any(|row| row.name == "sub@laptop"));
    asker.join().unwrap().unwrap();
}

/// Wait until the listing's ask is in the mailbox — a fact, not a sleep.
fn parked_out(mail: &Mailbox) {
    let ask = Call {
        client: "laptop".to_owned(),
        tool: TOOL.to_owned(),
        input: json!({ "args": TABLE }),
        cwd: None,
    };
    while !mail.queued(0, &ask) {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
