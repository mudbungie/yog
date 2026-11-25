//! **A pushing host is never busy** (REMOTE §5.8, bl-f5cd): a listing asked
//! while it runs a job is pushed to it, runs beside the job, and is answered
//! fresh — no remembered copy, no *queued, not refused*.

use serde_json::json;
use tempfile::tempdir;

use super::{LISTING, at, models, providers, world};
use crate::bz_host::far::TOOL;
use crate::config_edit::brazen::TABLE;
use crate::registry::mailbox::{Call, Post};
use crate::test_support::foot::{parked, printed};

#[test]
fn a_listing_is_answered_while_a_pushing_host_runs_a_job() {
    let root = tempdir().unwrap();
    let (deps, mail) = world(root.path());
    let job = Call {
        client: "laptop".to_owned(),
        tool: TOOL.to_owned(),
        input: json!({ "stdin": "a model call" }),
        cwd: None,
    };
    let job = mail.post(0, "driver", &job);
    let listed = printed(r#"{"models":[{"id":"big","default":true}]}"#);
    let foot = std::thread::spawn({
        let (mail, job) = (mail.clone(), job.clone());
        move || {
            let mut answered = 0;
            for frame in mail.push("laptop", &[], Box::new(|| 0)) {
                for handed in frame.work() {
                    let post = if handed.id == job {
                        Post::Partial(crate::registry::mailbox::Piece::default())
                    } else if handed.input["args"] == json!(TABLE) {
                        Post::Capture(printed(LISTING))
                    } else {
                        Post::Capture(listed.clone())
                    };
                    mail.complete(0, "laptop", &handed.id, &post)
                        .expect("posted");
                    answered += usize::from(handed.id != job);
                }
                if answered == 2 {
                    return;
                }
            }
        }
    });
    parked(&mail, "laptop");

    let rows = providers(&deps);
    assert!(
        rows.iter()
            .any(|row| row.name == "sub@laptop" && row.at == Some(at("laptop", "sub"))),
        "{rows:?}"
    );
    let fresh = models(&deps, "sub@laptop");
    assert!(fresh.is_ok(), "answered while the job runs: {fresh:?}");
    foot.join().unwrap();
    let standing = mail.collect(0, "driver", &job, None).expect("still held");
    assert_eq!(standing.capture, None, "and the job still runs");
}
